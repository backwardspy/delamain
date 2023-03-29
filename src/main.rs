mod autothread;
mod bot;
mod config;

use std::sync::Mutex;

use color_eyre::Result;
use config::Config;
use mlua::Lua;
use persy::Persy;

fn create_lua_runtime() -> Result<Lua> {
    let loader = format!(
        "local tl = (function()\n{}\nend)()\ntl.loader()",
        include_str!("../tl/tl.lua")
    );

    let lua = Lua::new();
    lua.load(&loader).exec()?;
    lua.load("require('init')").exec()?;

    Ok(lua)
}

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
        lua: Mutex::new(create_lua_runtime()?),
    };

    bot::run(bot, &config).await
}
