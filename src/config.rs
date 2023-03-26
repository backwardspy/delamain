use color_eyre::Result;
use std::env;

use poise::serenity_prelude as serenity;

pub struct Config {
    pub discord_token: String,
    pub testing_guild: Option<serenity::GuildId>,
    pub kv_store_name: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            discord_token: env::var("DISCORD_TOKEN")?,
            testing_guild: env::var("DISCORD_TESTING_GUILD")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(serenity::GuildId),
            kv_store_name: env::var("KV_STORE_NAME").unwrap_or("delamain.persy".to_owned()),
        })
    }
}
