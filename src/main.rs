use std::env;

use serenity::all::{ChannelId, Presence, Ready, VoiceState};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::prelude::*;
use songbird::SerenityInit;

mod commands;

struct Handler;

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
        if
        /* new_data.user.id.get().to_string().eq(&env::var("DISCORD ID lh").expect("error"))
        && */
        new_data
            .guild_id
            .expect("No guild")
            .to_string()
            .eq(&env::var("GUILD ID").expect("error"))
        {
            print!("{:?}", new_data.activities);

            let msgch: ChannelId = env::var("GENERAL")
                .unwrap()
                .parse()
                .expect("Error parsing channel id");

            let mut msg = format!(
                "{} começou a jogar {}",
                new_data.user.name.clone().unwrap(),
                new_data.activities.first().expect("").name
            );

            if let Some(a) = new_data.clone().activities.first() {
                // doesn't work
                msg = format!(
                    "{:?} \n\n {:?}",
                    a.clone().assets.unwrap().large_text,
                    a.clone().assets.unwrap().small_text
                );
            }
            msgch
                .say(&ctx.http, msg)
                .await
                .expect("Error sending message");
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

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let Some(old_state) = old {
            if let Some(_stream) = new.self_stream {
                if old_state
                    .user_id
                    .get()
                    .to_string()
                    .eq(&env::var("DISCORD ID lh").unwrap())
                {
                    let msgch: ChannelId = env::var("GENERAL")
                        .unwrap()
                        .parse()
                        .expect("Error parsing channel id");

                    msgch
                        .say(&ctx.http, "Stream started")
                        .await
                        .expect("Error sending message");
                }
            }
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

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
