use mlua::Lua;

use crate::lua::builder::ModuleBuilder;

pub fn setup(lua: &Lua) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Hidden")?;

    builder.add_function(lua, "warmup", "() -> ()", move |_lua, ()| {
        let _ = crate::features::hidden::NLP.pipe("warmup");

        Ok(())
    })?;

    builder.add_function(
        lua,
        "getHiddenNoun",
        "(message: string) -> string?",
        move |_lua, string: String| {
            let result = crate::features::hidden::extract_nouns_with_correct_verb(string.as_str());

            Ok(result)
        },
    )?;

    Ok(builder)
}
