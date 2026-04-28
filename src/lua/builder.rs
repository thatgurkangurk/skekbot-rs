use std::collections::BTreeMap;
use std::fmt::Write;

use crate::lua::LuauTypeExt;

pub struct ModuleBuilder {
    name: String,
    functions: Vec<(String, String)>,
    custom_types: Vec<String>,

    imports: BTreeMap<String, String>,
    table: mlua::Table,
}

impl ModuleBuilder {
    pub fn new(lua: &mlua::Lua, name: &str) -> mlua::Result<Self> {
        Ok(Self {
            name: name.to_string(),
            functions: Vec::new(),
            custom_types: Vec::new(),
            imports: BTreeMap::new(),
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

        let to_io_err = |e| std::io::Error::other(e);

        // emit imports first
        for (alias, path) in &self.imports {
            writeln!(content, r#"local {alias} = require("{path}")"#).map_err(to_io_err)?;
        }

        if !self.imports.is_empty() {
            content.push('\n');
        }

        for ty in &self.custom_types {
            writeln!(content, "{ty}").map_err(to_io_err)?;
        }

        writeln!(content, "\nexport type {}Module = {{", self.name).map_err(to_io_err)?;

        for (fn_name, fn_type) in &self.functions {
            writeln!(content, "    {fn_name}: {fn_type},").map_err(to_io_err)?;
        }

        content.push_str("}\n\n");
        writeln!(content, "return {{}} :: {}Module", self.name).map_err(to_io_err)?;

        let final_content = {
            #[cfg(feature = "formatting")]
            {
                crate::lua::format::format_code(&content)
                    .map_err(|e| std::io::Error::other(e.to_string()))?
            }
            #[cfg(not(feature = "formatting"))]
            {
                content
            }
        };

        let file_path = base_path.join(format!("{}.luau", self.name.to_lowercase()));
        std::fs::write(&file_path, final_content)?;

        Ok(())
    }

    pub fn declare_struct_as<T: LuauTypeExt>(&mut self, alias: &str) {
        let def = T::luau_definition();
        self.custom_types
            .push(format!("export type {alias} = {def}"));
    }

    pub fn declare_struct<T: LuauTypeExt>(&mut self) {
        let name = T::luau_name();
        let def = T::luau_definition();
        self.custom_types
            .push(format!("export type {name} = {def}"));
    }

    pub fn use_module(&mut self, module: &str, namespace: &str) {
        let full_path = format!("@skekbot/{}", module.to_lowercase());
        self.imports.insert(namespace.to_string(), full_path);
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
