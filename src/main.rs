use std::env;
use std::process::Output;
use std::sync::Arc;
use std::time::SystemTime;

use serenity::all::{ChannelId, Http, Presence, Ready, UserId, VoiceState};
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

struct LocalHandlerCache {
    voice_time_start: Arc<Mutex<SystemTime>>,
    old_vc: Arc<Mutex<ChannelId>>,
    old_activity_name: Arc<Mutex<String>>,
    activity_time_start: Arc<Mutex<SystemTime>>,
    old_pfp: Arc<Mutex<String>>,
}

const G_USER_ID: UserId = DISCORD_ID_JV;

#[async_trait]
impl EventHandler for LocalHandlerCache {
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
        let mut old_activity_name = self.old_activity_name.lock().await;
        let mut cached_start_activity_time = self.activity_time_start.lock().await;
        let ellapsed_time = SystemTime::now()
            .duration_since(*cached_start_activity_time)
            .unwrap()
            .as_secs();

        if new_data.user.id.eq(&G_USER_ID)
            && new_data.guild_id.unwrap().eq(&GUILD_ID)
            && ellapsed_time >= 30
        {
            if let Some(activity) = new_data.activities.first() {
                /// "" On activity.name means no activity
                match (activity.name.as_str(), old_activity_name.as_str()) {
                    // Avoid useless data
                    ("Spotify", _) | ("Hang Status", _) | ("Custom Status", _) => return,
                    (now, old) if now.eq(old) => return,
                    ("", _) => return,
                    // Started playing game from scratch
                    (_, "") => {
                        *old_activity_name = activity.name.clone();
                        *cached_start_activity_time = SystemTime::now();
                    }
                    // Changed games, and it took more than 30 seconds to do so.
                    // Unintentionally, this is also a catch-all statement.
                    (current, old) => {
                        let msg = format!(
                            "{} começou a jogar {} ap\u{00F3}s {} horas, {} minutos e {} segundos jogando {}",
                            &mut new_data.user.id.mention(),
                            current, ellapsed_time / 3600, (ellapsed_time % 3600) / 60, ellapsed_time % 60, old
                        );
                        *old_activity_name = activity.name.clone();
                        *cached_start_activity_time = SystemTime::now();
                        send_message!(GENERAL, &ctx, msg);
                    }
                }

                // testing
                /*println!(
                    "new {:?} \nold {:?}",
                    self.old_pfp.lock().await,
                    new_data.user.to_user().unwrap().avatar_url().unwrap()
                );
                if !self.old_pfp.lock().await.clone().eq(&new_data
                    .user
                    .to_user()
                    .unwrap()
                    .avatar_url()
                    .unwrap())
                {
                    println!("PFP changed");
                } else {
                    println!("PFP didn't change");
                    //send_message!(msgch, &ctx, format!("{}", new_data.user.to_user().unwrap().avatar_url().unwrap()));
                }*/

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
            } else {
                match old_activity_name.as_str() {
                    "" => return,
                    old => {
                        // If stopped playing game, and it took more than 30 seconds to do so
                        let msg = format!(
                            "{} jogou {} por {} horas, {} minutos e {} segundos.",
                            &mut new_data.user.id.mention(),
                            old,
                            ellapsed_time / 3600,
                            (ellapsed_time % 3600) / 60,
                            ellapsed_time % 60
                        );
                        *old_activity_name = String::from("");
                        send_message!(GENERAL, &ctx, msg);
                    }
                }
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild in ctx.cache.guilds() {
            guild
                .set_commands(
                    &ctx.http,
                    vec![
                        commands::ping::register(),
                        commands::join::register(),
                        commands::pic::register(),
                    ],
                )
                .await
                .unwrap();
        }

        let Some(voice_state) = ({
            let guild = GUILD_ID.to_guild_cached(&ctx).unwrap();
            guild.voice_states.get(&G_USER_ID).cloned()
        }) else {
            // User not present in any vsch
            return;
        };

        if let Some(ch) = voice_state.channel_id {
            let songbird = songbird::get(&ctx).await.expect("Songbird");
            try_join_channel!(songbird, GUILD_ID, ch);
            *self.old_vc.lock().await = ch.clone();
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let manager = songbird::get(&ctx).await.expect("Songbird");
        let jo_ch = &mut self.old_vc.lock().await.clone();
        if let Some(cached_ch) = ctx
            .cache
            .guild(GUILD_ID)
            .unwrap()
            .voice_states
            .get(&G_USER_ID)
        {
            *jo_ch = cached_ch.channel_id.unwrap();
        }

        match new.user_id {
            G_USER_ID => {
                if let Some(new_channel) = new.channel_id {
                    match (old.clone(), new.self_stream) {
                        (None, _) => {
                            *self.voice_time_start.lock().await = SystemTime::now();
                            *self.old_vc.lock().await = new_channel;
                            try_join_channel!(manager, *&GUILD_ID, new_channel);
                        }
                        (Some(old), _) if !new_channel.eq(&old.channel_id.unwrap()) => {
                            try_join_channel!(manager, *&GUILD_ID, new_channel);
                            *self.old_vc.lock().await = new_channel;
                        }
                        (_, Some(_)) => {
                            let now = SystemTime::now();
                            let start = *self.voice_time_start.lock().await;
                            let duration = now.duration_since(start).unwrap().as_secs();

                            let diff = format!("{} demorou {} horas, {} minutos, {} segundos para compartilhar a tela", G_USER_ID.mention(), duration / 3600, (duration % 3600) / 60, duration % 60);
                            send_message!(GENERAL, &ctx, diff);

                            let body = format!("Jotave demorou {} horas, {} minutos, {} segundos para compartilhar a tela", duration / 3600, (duration % 3600) / 60, duration % 60);

                            // Fuck this shit i'll figure out how the API works later
                            // TODO study this shit i guess
                            let output = std::process::Command::new("python3")
                                .arg("tweet.py")
                                .arg(body)
                                .output();

                            match output {
                                Ok(Output {
                                    status: _status,
                                    stdout,
                                    stderr,
                                }) => {
                                    if !stdout.is_empty() {
                                        println!("Output: {}", String::from_utf8_lossy(&stdout));
                                    }
                                    if !stderr.is_empty() {
                                        println!("Error: {}", String::from_utf8_lossy(&stderr));
                                    }
                                }
                                Err(e) => println!("Failed to execute command: {}", e),
                            }
                        }
                        _ => {}
                    }
                    return;
                } else {
                    // If user leaves (since new_channel will be None)
                    manager.leave(*&GUILD_ID).await.expect("Failed to leave");
                    return;
                }
            }
            DISCORD_ID_BOT => {
                if let Some(ch) = new.channel_id {
                    if !ch.eq(jo_ch) {
                        // If bot is moved
                        try_join_channel!(manager, *&new.guild_id.unwrap(), *jo_ch);
                    }
                } else if !Some(*jo_ch).is_some() {
                    // If bot is disconnected and the user is in the channel
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
                "foto" => match commands::pic::run(&ctx, &command.data.options()).await {
                    Ok(_) => Some("Changed.".to_string()),
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
        .event_handler(LocalHandlerCache {
            activity_time_start: Arc::new(Mutex::new(SystemTime::now())),
            voice_time_start: Arc::new(Mutex::new(SystemTime::now())),
            old_vc: Arc::new(Default::default()),
            old_activity_name: Arc::new(Mutex::new(String::from(""))),
            old_pfp: Arc::new(Mutex::new(
                G_USER_ID
                    .to_user(Http::new(DISCORD_TOKEN))
                    .await
                    .unwrap()
                    .avatar_url()
                    .unwrap(),
            )),
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
