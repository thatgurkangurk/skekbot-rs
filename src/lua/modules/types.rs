use mlua::{FromLua, Lua, Value};
use mlua::LuaSerdeExt;

use crate::lua::builder::ModuleBuilder;
use crate::models::server;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LuaMessage {
    pub id: String,
    pub content: String,
    pub author: LuaUser,
    pub channel_id: String,
    pub guild_id: Option<String>,
}

impl FromLua for LuaMessage {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct LuaUser {
    pub id: String,
    pub bot: bool,
    pub username: String,
    pub global_name: Option<String>,
}

impl FromLua for LuaUser {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

pub fn setup(lua: &Lua) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Types")?;

    builder.declare_struct_as::<server::Model>("ServerSettings");
    builder.declare_struct_as::<LuaMessage>("Message");

    Ok(builder)
}
