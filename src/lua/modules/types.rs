use mlua::Lua;

use crate::lua::builder::ModuleBuilder;
use crate::models::server;

pub fn setup(lua: &Lua) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Types")?;

    builder.declare_struct_as::<server::Model>("ServerSettings");

    Ok(builder)
}
