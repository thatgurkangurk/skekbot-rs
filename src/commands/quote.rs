use crate::{Context, Error};
use ::serenity::all::CreateAttachment;
use image::EncodableLayout;
use poise::{CreateReply, serenity_prelude as serenity};

#[poise::command(context_menu_command = "Quote")]
pub async fn quote(
    ctx: Context<'_>,
    #[description = "message to quote"] msg: serenity::Message,
) -> Result<(), Error> {
    let user = msg.author;
    let content = msg.content;

    let avatar_url = user.static_face();

    ctx.defer().await?;

    let image = match crate::features::quote::generate_quote_image(
        &avatar_url,
        &content,
        user.display_name(),
        &user.name,
    )
    .await
    {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to generate quote image: {e:?}");
            ctx.say("uh oh something went wrong").await?;
            return Ok(());
        }
    };

    let mut bytes: Vec<u8> = Vec::new();
    image::DynamicImage::ImageRgba8(image).write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )?;

    let attachment = CreateAttachment::bytes(bytes.as_bytes(), "quote.png");

    let message_builder = CreateReply::default().attachment(attachment);

    ctx.send(message_builder).await?;

    Ok(())
}
