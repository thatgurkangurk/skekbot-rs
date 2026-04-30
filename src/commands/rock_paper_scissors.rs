use crate::{Context, Error};
use ::serenity::{
    all::{CreateInteractionResponseMessage, EditMessage},
    futures::StreamExt,
};
use poise::{CreateReply, serenity_prelude as serenity};
use rand::seq::IndexedRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RockPaperScissorsOption {
    Rock,
    Paper,
    Scissors,
}

impl RockPaperScissorsOption {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Rock => "Rock",
            Self::Paper => "Paper",
            Self::Scissors => "Scissors",
        }
    }

    fn as_emoji(self) -> serenity::ReactionType {
        match self {
            Self::Paper => serenity::ReactionType::Unicode("📄".to_string()),
            Self::Rock => serenity::ReactionType::Unicode("🪨".to_string()),
            Self::Scissors => serenity::ReactionType::Unicode("✂️".to_string()),
        }
    }

    fn as_button(self, message_id: u64) -> serenity::CreateButton {
        // use the message ID to ensure the custom_id is unique to this specific game instance
        serenity::CreateButton::new(format!("{message_id}.{}", self.as_str()))
            .style(serenity::ButtonStyle::Primary)
            .emoji(self.as_emoji())
            .label(self.as_str())
    }

    const fn beats(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Rock, Self::Scissors)
                | (Self::Scissors, Self::Paper)
                | (Self::Paper, Self::Rock)
        )
    }

    fn get_random_option() -> Self {
        let options = [Self::Rock, Self::Paper, Self::Scissors];
        let mut rng = rand::rng();
        // choose() only returns None if the slice is empty; since this is static, this is safe.
        *options.choose(&mut rng).unwrap_or(&Self::Rock)
    }
}

async fn announce(
    ctx: Context<'_>,
    interaction: &serenity::ComponentInteraction,
    user1: &serenity::User,
    user2: &serenity::User,
    user1_choice: RockPaperScissorsOption,
    user2_choice: RockPaperScissorsOption,
) -> Result<(), Error> {
    let mut losers: Vec<serenity::User> = vec![];

    // remove buttons from the original message immediately
    let mut msg = interaction.message.clone();
    let _ = msg
        .edit(&ctx.http(), EditMessage::new().components(vec![]))
        .await;

    let outcome = if user1_choice == user2_choice {
        losers.push(user1.clone());
        if user1 != user2 {
            losers.push(user2.clone());
        }
        if user1 == user2 {
            "You played yourself and tied!"
        } else {
            "It's a tie!"
        }
        .to_string()
    } else if user1_choice.beats(user2_choice) {
        if user1 != user2 {
            losers.push(user2.clone());
        }
        if user1 == user2 {
            "You played yourself and won!".into()
        } else {
            format!("{} wins!", user1.display_name())
        }
    } else {
        if user1 != user2 {
            losers.push(user1.clone());
        }
        if user1 == user2 {
            "You played yourself and lost!".into()
        } else {
            format!("{} wins!", user2.display_name())
        }
    };

    // timeout logic
    if let Some(guild_id) = interaction.guild_id {
        for loser in losers {
            // don't try to timeout the bot itself (cuz that's silly)
            if loser.id == ctx.cache().current_user().id {
                continue;
            }

            if let Ok(mut member) = guild_id.member(&ctx.http(), loser.id).await {
                let timeout_until = std::time::SystemTime::now()
                    .checked_add(std::time::Duration::from_secs(60))
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs().cast_signed());

                if let Some(timestamp_secs) = timeout_until
                    && let Ok(timestamp) = serenity::Timestamp::from_unix_timestamp(timestamp_secs)
                {
                    let _ = member
                        .disable_communication_until_datetime(&ctx.http(), timestamp)
                        .await;
                }
            }
        }
    }

    interaction
        .create_response(
            &ctx.http(),
            serenity::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(format!(
                    "{} chose {}.\n{} chose {}.\n\n{}",
                    user1,
                    user1_choice.as_str(),
                    user2,
                    user2_choice.as_str(),
                    outcome
                )),
            ),
        )
        .await?;

    Ok(())
}

#[poise::command(slash_command, rename = "rock-paper-scissors")]
/// Play a game of rock paper scissors and timeout the loser!
pub async fn rock_paper_scissors(
    ctx: Context<'_>,
    #[description = "The user to play against."] against: serenity::User,
) -> Result<(), Error> {
    // send initial message and get the message object
    let handle = ctx
        .send(
            CreateReply::default()
                .content(format!(
                    "{} has challenged {} to Rock Paper Scissors!",
                    ctx.author(),
                    against
                ))
                .components(vec![serenity::CreateActionRow::Buttons(vec![
                    RockPaperScissorsOption::Rock.as_button(ctx.id()),
                    RockPaperScissorsOption::Paper.as_button(ctx.id()),
                    RockPaperScissorsOption::Scissors.as_button(ctx.id()),
                ])]),
        )
        .await?;

    let mut msg = handle.into_message().await?;
    let user_1_id = ctx.author().id;
    let user_2_id = against.id;

    let mut user_1_choice: Option<RockPaperScissorsOption> = None;
    let mut user_2_choice: Option<RockPaperScissorsOption> = None;

    // start collector filtered by THIS message id
    let mut collector = serenity::ComponentInteractionCollector::new(ctx)
        .message_id(msg.id) // this prevents "stealing" clicks from other games
        .timeout(std::time::Duration::from_secs(120))
        .stream();

    while let Some(mci) = collector.next().await {
        let choice = match mci.data.custom_id.as_str() {
            id if id.ends_with(".Rock") => RockPaperScissorsOption::Rock,
            id if id.ends_with(".Paper") => RockPaperScissorsOption::Paper,
            id if id.ends_with(".Scissors") => RockPaperScissorsOption::Scissors,
            _ => continue,
        };

        if mci.user.id == user_1_id {
            user_1_choice = Some(choice);
        } else if mci.user.id == user_2_id {
            user_2_choice = Some(choice);
        } else {
            // ignore people who aren't in the game
            let _ = mci
                .create_response(
                    &ctx.http(),
                    serenity::CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("You aren't a part of this battle!")
                            .ephemeral(true),
                    ),
                )
                .await;
            continue;
        }

        // handle bot or self-play logic
        if against.bot || against.id == ctx.author().id {
            user_2_choice = Some(RockPaperScissorsOption::get_random_option());
        }

        // if both have chosen, finish the game
        if let (Some(c1), Some(c2)) = (user_1_choice, user_2_choice) {
            announce(ctx, &mci, ctx.author(), &against, c1, c2).await?;
            return Ok(());
        }

        // acknowledge the click so it doesnt say that the interaction failed
        let _ = mci
            .create_response(
                &ctx.http(),
                serenity::CreateInteractionResponse::Acknowledge,
            )
            .await;
    }

    // clean up if the game times out
    let _ = msg
        .edit(
            &ctx.http(),
            EditMessage::new()
                .content("game timed out!")
                .components(vec![]),
        )
        .await;

    Ok(())
}
