use std::env;
use std::sync::Arc;
use std::time::SystemTime;

use serenity::all::{ChannelId, Presence, Ready, UserId, VoiceState};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::SerenityInit;

mod commands;

struct Handler {
    start_time_stamp_voice: Arc<Mutex<u64>>,
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

    // TODO: fix bug where reconnecting channel makes the bot send the message again
    async fn presence_update(&self, ctx: Context, new_data: Presence) {
        let msgch: ChannelId = env::var("GENERAL")
            .unwrap()
            .parse()
            .expect("Error parsing channel id");

        if
        /*new_data
        .user
        .id
        .to_string()
        .eq(&env::var("DISCORD ID lh").unwrap())
        &&*/
        new_data
            .guild_id
            .unwrap()
            .to_string()
            .eq(&env::var("GUILD ID").unwrap())
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
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        let guildid = new.guild_id.unwrap();
        let jo: UserId = UserId::from(
            env::var("DISCORD ID lh")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap(),
        );
        let bot: UserId = UserId::from(
            env::var("DISCORD ID BOT")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap(),
        );

        match new.user_id {
            jot if jot.eq(&jo) => {
                if let Some(new_channel) = new.channel_id {
                    let manager = songbird::get(&ctx).await.expect("Songbird").clone();

                    manager
                        .join(*&new.guild_id.unwrap(), new_channel)
                        .await
                        .expect("TODO: panic message");

                    *self.start_time_stamp_voice.lock().await = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                } else {
                    let manager = songbird::get(&ctx).await.expect("Songbird").clone();

                    manager.leave(*&guildid).await.expect("TODO: panic message");
                }
            }
            bo if bo.eq(&bot) => {
                // TODO disallow it to leave
            }
            _ => {}
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {:?}", command.data.name);
            println!("Options: {:?}", command.data.options());

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
    let token = env::var("DISCORD TOKEN").expect("Expected a token in the environment");

    let start_time_stamp_voice = Arc::new(Mutex::new(0));
    let start_time_stamp_activity = Arc::new(Mutex::new(0));

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            start_time_stamp_activity,
            start_time_stamp_voice,
        })
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
