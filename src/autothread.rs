use crate::{Bot, CommandContext};
use color_eyre::{eyre::eyre, Result};
use poise::serenity_prelude as serenity;

const SEGMENT: &'static str = "autothread";

#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn autothread(
    ctx: CommandContext<'_>,
    #[description = "The channel to toggle autothread on"] channel: serenity::Channel,
) -> Result<()> {
    let channel_id_bytes = channel.id().0.to_le_bytes();

    ctx.data().kv_declare_segment(SEGMENT)?;
    let kv = &ctx.data().kv;

    // check if channel is already autothreaded
    let mut tx = kv.begin()?;
    let on = if let Some((id, _)) = kv.scan(SEGMENT)?.find(|(_, v)| v == &channel_id_bytes) {
        tx.delete(SEGMENT, &id)?;
        false
    } else {
        tx.insert(SEGMENT, &channel_id_bytes)?;
        true
    };
    tx.commit()?;

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
    bot.kv_declare_segment(SEGMENT)?;
    if !bot
        .kv
        .scan(SEGMENT)?
        .any(|(_, c)| c == message.channel_id.0.to_le_bytes())
    {
        return Ok(());
    }

    // we are. if the message has no embeds, delete it.
    // TODO: decide between embeds or attachments or both
    if message.embeds.is_empty() && message.attachments.is_empty() {
        message
            .delete(&ctx.http)
            .await
            .map_err(|e| eyre!("failed to delete message: {}", e))?;
    }
    // if it does, create a new thread under it.
    else {
        // attempt to get the first attachment's filename, or the first embed's title, or finally the author's name.
        let thread_subject = message
            .attachments
            .first()
            .map(|a| a.filename.clone())
            .or_else(|| message.embeds.first().and_then(|embed| embed.title.clone()))
            .unwrap_or(format!("{}'s submission", message.author.name));

        let thread = message
            .channel_id
            .create_public_thread(ctx, message, |thread| {
                thread
                    .name(format!("Discussion of {}", thread_subject))
                    .auto_archive_duration(1440)
                    .kind(serenity::ChannelType::PublicThread)
            })
            .await?;

        thread
            .say(
                ctx,
                format!(
                    "Discuss {}'s submission here!",
                    serenity::Mention::User(message.author.id)
                ),
            )
            .await?;
    }

    Ok(())
}
