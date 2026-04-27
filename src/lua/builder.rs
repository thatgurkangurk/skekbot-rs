use std::path::Path;
use stylua_lib::{Config as StyluaConfig, OutputVerification};

pub struct ModuleBuilder {
    name: String,
    functions: Vec<(String, String)>,
    custom_types: Vec<String>,
    table: mlua::Table,
}

impl ModuleBuilder {
    fn get_stylua_config(&self) -> StyluaConfig {
        let config_path = Path::new(".stylua.toml");

        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(config_path)
                && let Ok(config) = toml::from_str(&content)
            {
                return config;
            }
            tracing::warn!("found .stylua.toml but failed to parse it. using defaults.");
        }

        StyluaConfig::default()
    }

    pub fn new(lua: &mlua::Lua, name: &str) -> mlua::Result<Self> {
        Ok(Self {
            name: name.to_string(),
            functions: Vec::new(),
            custom_types: Vec::new(),
            table: lua.create_table()?,
        })
    }

    pub fn add_type_declaration(&mut self, declaration: &str) {
        self.custom_types.push(declaration.to_string());
    }

    pub fn add_function<A, R, F>(
        &mut self,
        lua: &mlua::Lua,
        name: &str,
        luau_type: &str,
        func: F,
    ) -> mlua::Result<()>
    where
        A: mlua::FromLuaMulti,
        R: mlua::IntoLuaMulti,
        F: Fn(&mlua::Lua, A) -> mlua::Result<R> + 'static + Send + Sync,
    {
        self.functions
            .push((name.to_string(), luau_type.to_string()));
        self.table.set(name, lua.create_function(func)?)?;
        Ok(())
    }

    pub fn add_async_function<A, R, F, Fut>(
        &mut self,
        lua: &mlua::Lua,
        name: &str,
        luau_type: &str,
        func: F,
    ) -> mlua::Result<()>
    where
        A: mlua::FromLuaMulti + 'static,
        R: mlua::IntoLuaMulti + 'static,
        F: Fn(mlua::Lua, A) -> Fut + 'static + Send + Sync,
        Fut: std::future::Future<Output = mlua::Result<R>> + Send + 'static,
    {
        self.functions
            .push((name.to_string(), luau_type.to_string()));
        self.table.set(name, lua.create_async_function(func)?)?;
        Ok(())
    }

    pub fn emit_type_file(&self, base_path: &std::path::Path) -> std::io::Result<()> {
        tracing::info!("trying to emit type file for {}", self.name);

        let mut content = String::new();

        for ty in &self.custom_types {
            content.push_str(ty);
            content.push_str("\n");
        }

        content.push_str(&format!("\nexport type {}Module = {{\n", self.name));
        for (fn_name, fn_type) in &self.functions {
            content.push_str(&format!("    {fn_name}: {fn_type},\n"));
        }
        content.push_str("}\n\n");
        content.push_str(&format!("return {{}} :: {}Module\n", self.name));

        let config = self.get_stylua_config();

        let formatted_content =
            stylua_lib::format_code(&content, config, None, OutputVerification::None)
                .map_err(|e| std::io::Error::other(e.to_string()))?;

        let file_path = base_path.join(format!("{}.luau", self.name.to_lowercase()));
        std::fs::write(&file_path, formatted_content)?;

        Ok(())
    }

    // For attaching pre-computed values, tables, or signals
    pub fn add_value<V: mlua::IntoLua>(
        &mut self,
        name: &str,
        luau_type: &str,
        value: V,
    ) -> mlua::Result<()> {
        self.functions
            .push((name.to_string(), luau_type.to_string()));
        self.table.set(name, value)?;
        Ok(())
    }

    pub fn register(&self, registry: &mlua::Table) -> mlua::Result<()> {
        registry.set(
            format!("@skekbot/{}", self.name.to_lowercase()),
            self.table.clone(),
        )
    }
}
