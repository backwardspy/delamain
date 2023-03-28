mod autothread;
mod bot;
mod config;

use color_eyre::Result;
use config::Config;
use persy::Persy;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Config::from_env()?;

    let bot = bot::Bot {
        kv: Persy::open_or_create_with(&config.kv_store_name, persy::Config::new(), |_persy| {
            Ok(())
        })?,
    };

    bot::run(bot, &config).await
}
