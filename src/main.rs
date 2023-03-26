mod autothread;
mod config;

use color_eyre::Result;
use config::Config;
use persy::Persy;
use poise::serenity_prelude as serenity;
use tracing::info;

pub struct Bot {
    kv: Persy,
}

impl Bot {
    fn kv_declare_segment(&self, name: &str) -> Result<()> {
        // TODO: allow plugins to declare their kv segment up-front.
        if !self.kv.exists_segment(name)? {
            let mut tx = self.kv.begin()?;
            tx.create_segment(name)?;
            tx.commit()?;
        }
        Ok(())
    }
}

type Error = color_eyre::eyre::ErrReport;
type CommandContext<'a> = poise::Context<'a, Bot, Error>;

async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Bot, Error>,
    bot: &Bot,
) -> Result<()> {
    match event {
        poise::Event::Ready { .. } => {
            info!("Leave your problems at the door.");
        }
        poise::Event::Message { new_message } => {
            autothread::handle_message(new_message, ctx, bot).await?;
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let config = Config::from_env()?;

    let bot =
        Bot {
            kv: Persy::open_or_create_with(config.kv_store_name, persy::Config::new(), |_persy| {
                Ok(())
            })?,
        };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![autothread::autothread()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .token(config.discord_token)
        .intents(intents)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                match config.testing_guild {
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

    framework.run().await?;

    Ok(())
}
