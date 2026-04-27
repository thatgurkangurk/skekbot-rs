use mlua::{Lua, Table, Value};
use std::{collections::HashSet, time::Instant};

fn stringify_value(
    value: &Value,
    visited: &mut HashSet<usize>,
    indent: usize,
) -> mlua::Result<String> {
    let pad = "  ".repeat(indent);

    match value {
        Value::Nil => Ok("nil".to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(format!("{:?}", s.to_str()?)),
        Value::Table(t) => {
            // Get the raw memory address of the table to check for cycles
            let ptr = t.to_pointer() as usize;
            if visited.contains(&ptr) {
                return Ok("\"<cycle>\"".to_string());
            }

            visited.insert(ptr);

            let mut items = Vec::new();
            for pair in t.pairs::<Value, Value>() {
                let (k, v) = pair?;

                let key_str = match k {
                    Value::String(s) => format!("{:?}", s.to_str()?),
                    Value::Integer(i) => i.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    _ => format!("\"<{}>\"", k.type_name()),
                };

                let val_str = stringify_value(&v, visited, indent + 1)?;
                items.push(format!("  {pad}[{key_str}] = {val_str}"));
            }

            visited.remove(&ptr);

            if items.is_empty() {
                Ok("{}".to_string())
            } else {
                let inner = items.join(",\n");
                Ok(format!("{{\n{inner}\n{pad}}}"))
            }
        }
        Value::Function(_) => Ok("\"<function>\"".to_string()),
        Value::UserData(_) | Value::LightUserData(_) => Ok("\"<userdata>\"".to_string()),
        Value::Thread(_) => Ok("\"<thread>\"".to_string()),
        Value::Error(e) => Ok(format!("\"<error: {e}>\"")),

        // catch all for luau's Vector, Buffer, Other, or future mlua updates
        _ => Ok(format!("\"<{}>\"", value.type_name())),
    }
}

pub fn setup(lua: &Lua, registry: &Table) -> anyhow::Result<()> {
    let utils_table = lua.create_table()?;

    let start_time = Instant::now();
    let uptime_helper = lua.create_function(move |_, ()| Ok(start_time.elapsed().as_secs()))?;
    utils_table.set("getUptime", uptime_helper)?;

    let sleep_helper = lua.create_async_function(|_, seconds: f64| async move {
        tokio::time::sleep(std::time::Duration::from_secs_f64(seconds)).await;
        Ok(())
    })?;
    utils_table.set("sleep", sleep_helper)?;

    let stringify = lua.create_function(|_, value: Value| {
        let mut visited = HashSet::new();
        let result = stringify_value(&value, &mut visited, 0)?;
        Ok(result)
    })?;
    utils_table.set("stringify", stringify)?;

    registry.set("@skekbot/utils", utils_table)?;
    Ok(())
}
