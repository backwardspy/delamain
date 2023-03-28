use color_eyre::{eyre::eyre, Result};
use poise::serenity_prelude as serenity;

use crate::bot::{Bot, CommandContext};

pub const SEGMENT: &str = "autothread";

#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn autothread(
    ctx: CommandContext<'_>,
    #[description = "The channel to toggle autothread on"] channel: serenity::Channel,
) -> Result<()> {
    let channel_id_bytes = make_channel_id_bytes(channel.id());

    let on = toggle_autothreading(&ctx.data().kv, &channel_id_bytes)?;

    ctx.say(format!(
        "Autothreading {} for {}.",
        if on { "enabled" } else { "disabled" },
        serenity::Mention::Channel(channel.id())
    ))
    .await?;

    Ok(())
}

pub async fn handle_message(
    message: &serenity::Message,
    ctx: &serenity::Context,
    bot: &Bot,
) -> Result<()> {
    // check if we're meant to autothread this channel
    if !bot
        .kv
        .scan(SEGMENT)?
        .any(|(_, c)| c == make_channel_id_bytes(message.channel_id))
    {
        return Ok(());
    }

    // we are. if the message has no attachments, delete it.
    if message.attachments.is_empty() {
        delete_message(message, ctx).await?;
    }
    // if it does, create a new thread under it.
    else {
        create_autothread(message, ctx).await?;
    }

    Ok(())
}

fn make_channel_id_bytes(id: serenity::ChannelId) -> [u8; 8] {
    id.0.to_le_bytes()
}

fn toggle_autothreading(kv: &persy::Persy, channel_id_bytes: &[u8]) -> Result<bool> {
    let mut tx = kv.begin()?;

    let on = if let Some((id, _)) = kv.scan(SEGMENT)?.find(|(_, v)| v == channel_id_bytes) {
        tx.delete(SEGMENT, &id)?;
        false
    } else {
        tx.insert(SEGMENT, channel_id_bytes)?;
        true
    };

    tx.commit()?;

    Ok(on)
}

async fn create_autothread(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    let subject = get_thread_subject(message);
    let thread = create_thread(message, ctx, subject).await?;
    post_thread_message(&thread, message, ctx).await?;
    Ok(())
}

async fn post_thread_message(
    thread: &serenity::GuildChannel,
    message: &serenity::Message,
    ctx: &serenity::Context,
) -> Result<serenity::Message> {
    Ok(thread
        .say(
            ctx,
            format!(
                "Discuss {}'s submission here!",
                serenity::Mention::User(message.author.id)
            ),
        )
        .await?)
}

async fn create_thread(
    message: &serenity::Message,
    ctx: &serenity::Context,
    subject: String,
) -> Result<serenity::GuildChannel, color_eyre::Report> {
    Ok(message
        .channel_id
        .create_public_thread(ctx, message, |thread| {
            thread
                .name(format!("Discussion of {}", subject))
                .auto_archive_duration(1440)
                .kind(serenity::ChannelType::PublicThread)
        })
        .await?)
}

fn get_thread_subject(message: &serenity::Message) -> String {
    // attempt to get the first attachment's filename, fall back on author's name otherwise.
    message
        .attachments
        .first()
        .map(|a| a.filename.clone())
        .unwrap_or(format!("{}'s submission", message.author.name))
}

async fn delete_message(message: &serenity::Message, ctx: &serenity::Context) -> Result<()> {
    message
        .delete(&ctx.http)
        .await
        .map_err(|e| eyre!("failed to delete message: {}", e))
}
