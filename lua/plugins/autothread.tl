local del = require("delamain")

local record context
    say: function(string)
end

local record user
    id: string
    name: string

    mention: function(): string
end

local record channel
    id: string

    mention: function(): string
end

local record message
    channel: channel
    author: user
    attachments: {attachment}
end

local record attachment
    filename: string
end

local function toggle_autothreading(chan: channel): boolean
    if del.kv_exists(chan.id) then
        del.kv_delete(chan.id)
        return false
    else
        del.kv_set(chan.id, "true")
        return true
    end
end

del.command("autothread", function(ctx: context, chan: channel)
    local on = toggle_autothreading(chan)
    local state = on and "enabled" or "disabled"
    ctx.say("Autothreading is now " .. state .. " for channel " .. chan.mention())
end, { permissions = "ADMINISTRATOR" })

del.on("message_create", function(msg: message)
    if not del.kv_exists(msg.channel.id) then return end

    if #msg.attachments > 0 then
        local subject = msg.attachments[1].filename or msg.author.name .. "'s submission"
        local thread = del.thread_create(msg, {archive_duration = 1440, subject = subject})
        del.create_message(thread, "Discuss " .. msg.author.mention() .. "'s submission here!")
    else
        del.delete_message(msg)
    end
end)
