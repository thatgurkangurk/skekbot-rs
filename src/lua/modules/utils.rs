use mlua::{Lua, Table};
use std::time::Instant;

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

    registry.set("@skekbot/utils", utils_table)?;
    Ok(())
}
