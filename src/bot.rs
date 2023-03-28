use color_eyre::{eyre::eyre, Result};
use persy::Persy;
use poise::serenity_prelude as serenity;
use tracing::info;

use crate::{autothread, config::Config};

pub struct Bot {
    pub kv: Persy,
}

impl Bot {
    fn kv_declare_segment(&self, name: &str) -> Result<()> {
        if !self.kv.exists_segment(name)? {
            let mut tx = self.kv.begin()?;
            tx.create_segment(name)?;
            tx.commit()?;
        }
        Ok(())
    }
}

type Error = color_eyre::eyre::ErrReport;
pub type CommandContext<'a> = poise::Context<'a, Bot, Error>;

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Bot, Error>,
    bot: &Bot,
) -> Result<()> {
    match event {
        poise::Event::Ready { .. } => {
            info!("Declaring plugin KV segments.");
            bot.kv_declare_segment(autothread::SEGMENT)?;

            info!("Leave your problems at the door.");
        }
        poise::Event::Message { new_message } => {
            autothread::handle_message(new_message, ctx, bot).await?;
        }
        _ => {}
    }

    Ok(())
}

pub async fn run(bot: Bot, config: &Config) -> Result<()> {
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let testing_guild = config.testing_guild;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![autothread::autothread()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .token(&config.discord_token)
        .intents(intents)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                match testing_guild {
                    Some(guild) => {
                        info!("Registering slash commands in guild {}.", guild);
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            guild,
                        )
                        .await?
                    }
                    None => {
                        info!("Registering slash commands globally.");
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?
                    }
                }
                Ok(bot)
            })
        });
    framework
        .run()
        .await
        .map_err(|e| eyre!("Error running framework: {}", e))
}
