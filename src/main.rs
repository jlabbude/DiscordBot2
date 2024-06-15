use std::env;

use serenity::all::{GuildId, Ready, VoiceState};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::prelude::*;

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

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guilds: Vec<GuildId> = ctx.cache.guilds();

        for guild in guilds {
            guild.set_commands(&ctx.http, vec![
                    commands::ping::register(),
                    commands::join::register(),
                ]).await.unwrap();
        }
    }

    async fn voice_state_update(&self, _ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let Some(old_state) = old{
            if old_state.user_id.get().to_string().eq(&env::var("DISCORD ID lh").unwrap()){
                println!("OLD STATE: \n {:?} \n\n NEW STATE: {:?}", old_state, new)
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "join" => {
                    match commands::join::run(&ctx, &command.data.options()).await {
                        Ok(_) => Some("Join command executed successfully".to_string()),
                        Err(e) => Some(format!("Error executing join command: {}", e)),
                    }
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
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
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILDS;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}