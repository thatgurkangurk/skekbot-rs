use crate::config::Config;
use crate::lua::builder::ModuleBuilder;
use mlua::Lua;

pub fn setup(lua: &Lua, config: &Config) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Env")?;

    let env_access_enabled = config.lua.as_ref().is_some_and(|cfg| cfg.allow_env_access);

    builder.add_function(
        lua,
        "getEnvironmentVariable",
        "(key: string) -> string?",
        move |lua, key: String| {
            if !env_access_enabled {
                let (source, line) = lua
                    .inspect_stack(1, |debug| {
                        let src = debug.source().short_src.as_ref().map_or_else(
                            || "<unknown>".to_string(),
                            std::string::ToString::to_string,
                        );

                        (src, debug.current_line())
                    })
                    .unwrap_or_else(|| ("<unknown>".to_string(), Some(0)));

                tracing::error!(
                    lua_source = %source,
                    lua_line = line,
                    "blocked Lua script attempt to read environment variable: '{}'",
                    key
                );

                return Err(mlua::Error::RuntimeError(format!(
                    "environment access is disabled (attempted to read '{key}')"
                )));
            }

            Ok(std::env::var(key).ok())
        },
    )?;

    Ok(builder)
}
