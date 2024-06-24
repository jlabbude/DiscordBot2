use std::env;
use std::sync::Arc;
use std::time::SystemTime;

use serenity::all::{ChannelId, GuildId, Presence, Ready, UserId, VoiceState};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::SerenityInit;

mod commands;

include!(concat!(env!("OUT_DIR"), "/env.rs"));

macro_rules! try_join_channel {
    ($manager:expr, $guild_id:expr, $channel:expr) => {
        if let Err(why) = $manager.join(*&$guild_id, $channel).await {
            println!("Error joining channel: {:?}", why);
        } else {
            return;
        }
    };
}

macro_rules! send_message {
    ($msg_ch_id:expr, $context:expr, $message_content:expr) => {
        if let Err(why) = $msg_ch_id.say(&$context, $message_content).await {
            println!("Error sending message: {why:?}");
        }
    };
}

struct Handler {
    voice_time_start: Arc<Mutex<SystemTime>>,
    old_vc: Arc<Mutex<ChannelId>>,
    old_activity_name: Arc<Mutex<String>>,
    activity_time_start: Arc<Mutex<SystemTime>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "teste" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "teste").await {
                    println!("Error sending message: {why:?}");
                }
            }
            _ => {
                println!("Author: {:?} \n Message: {}", msg.author.name, msg.content);
            }
        }
    }

    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        let msgch: ChannelId = ChannelId::from(GENERAL.parse::<u64>().unwrap());
        let mut old_activity_name = self.old_activity_name.lock().await;
        let mut activity_time = self.activity_time_start.lock().await;

        if new_data.user.id.to_string().eq(DISCORD_ID_LH)
            && new_data.guild_id.unwrap().to_string().eq(GUILD_ID)
        {
            if let Some(activity) = new_data.activities.first() {
                match (
                    activity.name.as_str(),
                    old_activity_name.as_str(),
                    *activity_time,
                ) {
                    // Avoid useless data
                    ("Spotify", _, _) | ("Hang Status", _, _) => return,
                    (activity_now, _, _) if activity_now == old_activity_name.as_str() => return,
                    // If stopped playing game, and it took more than 30 seconds to do so
                    (empty, old, now)
                        if now.duration_since(*activity_time).unwrap().as_secs() >= 30
                            && empty.is_empty() =>
                    {
                        let duration = now.duration_since(*activity_time).unwrap().as_secs();
                        let msg = format!(
                            "{} jogou {} por {} horas, {} minutos e {} segundos.",
                            &mut new_data.user.id.mention(),
                            old,
                            duration / 3600,
                            (duration % 3600) / 60,
                            duration % 60
                        );
                        *old_activity_name = activity.name.clone();
                        *activity_time = SystemTime::now();
                        send_message!(msgch, &ctx, msg);
                    }
                    // Changed games, and it took more than 30 seconds to do so
                    (current, old, now)
                        if now.duration_since(*activity_time).unwrap().as_secs() >= 30
                            && !current.is_empty() =>
                    {
                        let duration = now.duration_since(*activity_time).unwrap().as_secs();
                        let msg = format!(
                            "{} começou a jogar {} ap�s {} horas, {} minutos e {} segundos jogando {}",
                            &mut new_data.user.id.mention(),
                            current, duration / 3600, (duration % 3600) / 60, duration % 60, old
                        );

                        *old_activity_name = activity.name.clone();
                        *activity_time = SystemTime::now();

                        send_message!(msgch, &ctx, msg);
                    }
                    // Started playing game from scratch
                    (name, empty, now)
                        if now.duration_since(*activity_time).unwrap().as_secs() >= 30
                            && empty.is_empty() =>
                    {
                        let msg = format!(
                            "{} começou a jogar {}",
                            &mut new_data.user.id.mention(),
                            name,
                        );

                        *old_activity_name = activity.name.clone();

                        send_message!(msgch, &ctx, msg);
                    }
                    _ => println!("124"),
                }

                /*

                For some reason there's no normalization for what should be what, so each activity
                displays a different thing on different fields, so I'm just going to leave this here
                until I figure out what to do with it

                 if let Some(a) = &activity.state {
                     // doesn't work
                     msg = format!(
                         "{:?}",
                         a
                         a.large_text,
                         a.small_text
                     );
                 }

                 */
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ctx.cache.guilds() {
            guild
                .set_commands(
                    &ctx.http,
                    vec![commands::ping::register(), commands::join::register()],
                )
                .await
                .unwrap();
        }

        let manager = songbird::get(&ctx).await.expect("Songbird");
        let guild: GuildId = GuildId::from(GUILD_ID.parse::<u64>().unwrap());
        let jo: UserId = UserId::from(DISCORD_ID_LH.parse::<u64>().unwrap());
        let guild_data = ctx.cache.guild(guild).unwrap().clone();
        if let Some(vs) = guild_data.voice_states.get(&jo) {
            if let Some(ch) = vs.channel_id {
                try_join_channel!(manager, guild, ch);
                *self.old_vc.lock().await = ch.clone();
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let guildid = new.guild_id.unwrap();
        let jo: UserId = UserId::from(DISCORD_ID_LH.parse::<u64>().unwrap());
        let bot: UserId = UserId::from(DISCORD_ID_BOT.parse::<u64>().unwrap());
        let manager = songbird::get(&ctx).await.expect("Songbird");
        let jo_ch = &mut self.old_vc.lock().await.clone();
        if let Some(cached_ch) = ctx.cache.guild(guildid).unwrap().voice_states.get(&jo) {
            *jo_ch = cached_ch.channel_id.unwrap();
        }

        match new.user_id {
            jot if jot.eq(&jo) => {
                if let Some(new_channel) = new.channel_id {
                    match (old, new.self_stream) {
                        (None, _) => {
                            try_join_channel!(manager, *&guildid, new_channel);
                            *self.old_vc.lock().await = new_channel;
                        }
                        (Some(old), _) if !new_channel.eq(&old.channel_id.unwrap()) => {
                            try_join_channel!(manager, *&guildid, new_channel);
                            *self.old_vc.lock().await = new_channel;
                            *self.voice_time_start.lock().await = SystemTime::now();
                        }
                        (_, Some(_)) => {
                            let now = SystemTime::now();
                            let start = *self.voice_time_start.lock().await;
                            let duration = now.duration_since(start).unwrap().as_secs();

                            let msg_ch_id: ChannelId =
                                ChannelId::from(GENERAL.parse::<u64>().unwrap());

                            let diff = format!("{} demorou {} horas, {} minutos, {} segundos para compartilhar a tela", jo.mention(), duration / 3600, (duration % 3600) / 60, duration % 60);

                            send_message!(msg_ch_id, &ctx, diff);
                        }
                        _ => {}
                    }
                    return;
                } else {
                    manager.leave(*&guildid).await.expect("Failed to leave");
                    return;
                }
            }
            bo if bo.eq(&bot) => {
                if let Some(ch) = new.channel_id {
                    if !ch.eq(jo_ch) {
                        try_join_channel!(manager, *&new.guild_id.unwrap(), *jo_ch);
                    }
                } else {
                    try_join_channel!(manager, *&new.guild_id.unwrap(), *jo_ch);
                }
            }
            _ => {}
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "join" => match commands::join::run(&ctx, &command.data.options()).await {
                    Ok(_) => Some("Joined.".to_string()),
                    Err(e) => Some(format!("Error: {}", e)),
                },
                _ => Some("Not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond: {why}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(DISCORD_TOKEN, intents)
        .event_handler(Handler {
            activity_time_start: Arc::new(Mutex::new(SystemTime::now())),
            voice_time_start: Arc::new(Mutex::new(SystemTime::now())),
            old_vc: Arc::new(Default::default()),
            old_activity_name: Arc::new(Default::default()),
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
