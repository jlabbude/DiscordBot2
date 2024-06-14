use std::env;

use serenity::all::{Ready, VoiceState};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

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

    async fn ready(&self, _ctx: Context, data_about_bot: Ready) {
        println!("{:?}", data_about_bot.user.name)
    }

    async fn voice_state_update(&self, _ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let Some(old_state) = old{
            if old_state.user_id.get().to_string().eq(&env::var("DISCORD ID lh").unwrap()){
                println!("OLD STATE: \n {:?} \n\n NEW STATE: {:?}", old_state, new)
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