use crate::lua::builder::ModuleBuilder;
use mlua::{Function, Lua, Table};

pub fn setup(lua: &Lua) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Log")?;

    let log_backend =
        lua.create_function(|_, (level, location, message): (String, String, String)| {
            match level.to_uppercase().as_str() {
                "ERROR" => tracing::error!("({}) {}", location, message),
                "WARN" => tracing::warn!("({}) {}", location, message),
                _ => tracing::info!("({}) {}", location, message),
            }
            Ok(())
        })?;

    let log_table: Table = lua
        .load(
            r##"
        local log_backend = ...
        local function get_caller_info(stack_level)
            local source, line = debug.info(stack_level, "sl")
            if not source then return "unknown" end
            
            -- strip [string "name"] wrapper
            local clean_name = string.match(source, '^%[string "(.-)"%]$')
            if clean_name then
                source = clean_name
            end
            
            if string.sub(source, 1, 1) == "=" or string.sub(source, 1, 1) == "@" then
                source = string.sub(source, 2)
            end
            return source .. ":" .. tostring(line)
        end

        print = function(...)
            local num_args = select("#", ...)
            local str = {}
            for i = 1, num_args do
                table.insert(str, tostring(select(i, ...)))
            end
            log_backend("INFO", get_caller_info(3), table.concat(str, "\t"))
        end

        return {
            log = function(level, message)
                log_backend(level, get_caller_info(3), tostring(message))
            end
        }
        "##,
        )
        .call(log_backend)?;

    // Extract the generated log function and bind it to our builder
    let log_func: Function = log_table.get("log")?;
    builder.add_value("log", "(level: string, message: string) -> ()", log_func)?;

    Ok(builder)
}
