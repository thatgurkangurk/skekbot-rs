use poise::serenity_prelude as serenity;

use crate::features;
use crate::{Data, Error};

pub async fn event_handler_root(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    event_handler(ctx, event, framework, _data).await?;
    features::dad::event_handler(ctx, event, framework, _data).await?;
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
        println!("Logged in as {}", data_about_bot.user.name);
        let mut data = ctx.data.write().await;

        let mut fallback_name = data_about_bot.user.name.clone();
        let name = data.get_mut::<Data>().unwrap_or(&mut fallback_name);

        *name = data_about_bot.user.name.clone();
    }
    Ok(())
}
