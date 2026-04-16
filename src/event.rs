use poise::serenity_prelude as serenity;
use tracing::info;

use crate::features;
use crate::{Data, Error};

pub async fn event_handler_root(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    event_handler(ctx, event, framework, data).await?;
    features::dad::event_handler(ctx, event, framework, data).await?;
    features::hidden::event_handler(ctx, event, framework, data).await?;
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
        info!("Logged in as {}", data_about_bot.user.name);
        let mut data = ctx.data.write().await;

        if let Some(name) = data.get_mut::<Data>() {
            name.clone_from(&data_about_bot.user.name);
        }
    }
    Ok(())
}
