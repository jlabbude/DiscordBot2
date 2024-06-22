use std::env;
use std::num::NonZero;
use std::sync::Arc;
use std::time::SystemTime;

use serenity::all::{ChannelId, GuildId, Ready, UserId, VoiceState};
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


struct Handler {
    start_time_stamp_voice: Arc<Mutex<u64>>,
    old_vc: Arc<Mutex<ChannelId>>,
    start_time_stamp_activity: Arc<Mutex<u64>>,
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

    /* TODO: fix bug where reconnecting channel makes the bot send the message again

        async fn presence_update(&self, ctx: Context, new_data: Presence) {
            let msgch: ChannelId = GENERAL
                .unwrap()
                .parse()
                .expect("Error parsing channel id");

            if
            /*new_data
            .user
            .id
            .to_string()
            .eq(&DISCORD_ID_lh.unwrap())
            &&*/
            new_data
                .guild_id
                .unwrap()
                .to_string()
                .eq(&GUILD_ID.unwrap())
            {
                if let Some(activity) = new_data.activities.first() {
                    {
                        print!("{:?}", &new_data.activities);

                        let msg = format!(
                            "{} começou a jogar {}",
                            &mut new_data.user.name.unwrap(),
                            activity.name
                        );

                        // For some reason there's no normalization for what should be what, so each activity
                        // displays a different thing on different fields, so I'm just going to leave this here for now

                        // if let Some(a) = &activity.state {
                        //     // doesn't work
                        //     msg = format!(
                        //         "{:?}",
                        //         a
                        //         a.large_text,
                        //         a.small_text
                        //     );
                        // }

                        msgch
                            .say(&ctx.http, msg)
                            .await
                            .expect("Error sending message");
                    }
                }
            }
        }
    */
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
        let guild: GuildId = GuildId::from(
            GUILD_ID
                .parse::<NonZero<u64>>()
                .unwrap(),
        );
        let jo: UserId = UserId::from(
            DISCORD_ID_LH
                .parse::<u64>()
                .unwrap(),
        );
        let guild_data = ctx.cache.guild(guild).unwrap().clone();
        if let Some(vs) = guild_data.voice_states.get(&jo) {
            if let Some(ch) = vs.channel_id {
                try_join_channel!(manager, guild, ch);
                *self.old_vc.lock().await = ch.clone();
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let guildid = new.guild_id.unwrap();
        let jo: UserId = UserId::from(
            DISCORD_ID_LH
                .parse::<u64>()
                .unwrap(),
        );
        let bot: UserId = UserId::from(
            DISCORD_ID_BOT
                .parse::<u64>()
                .unwrap(),
        );
        let manager = songbird::get(&ctx).await.expect("Songbird");
        let jo_ch = &mut self.old_vc.lock().await.clone();
        if let Some(cached_ch) = ctx.cache.guild(guildid).unwrap().voice_states.get(&jo) {
            *jo_ch = cached_ch.channel_id.unwrap();
        }

        match new.user_id {
            jot if jot.eq(&jo) => {
                if let Some(new_channel) = new.channel_id {
                    try_join_channel!(manager, *&guildid, new_channel);

                    *self.old_vc.lock().await = new_channel;

                    *self.start_time_stamp_voice.lock().await = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .into();
                    return;
                } else {
                    manager.leave(*&guildid).await.expect("TODO: panic message");
                    return;
                }
            }
            bo if bo.eq(&bot) => {
                if let Some(ch) = new.channel_id {
                    if !ch.eq(jo_ch) {
                        try_join_channel!(manager, *&new.guild_id.unwrap(), *jo_ch);
                    }
                } else{
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
            start_time_stamp_activity: Arc::new(Default::default()),
            start_time_stamp_voice: Arc::new(Default::default()),
            old_vc: Arc::new(Default::default()),
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
