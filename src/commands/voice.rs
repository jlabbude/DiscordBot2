#![allow(deprecated)]
use reqwest::Client;
use serde_json::{json, Value};
use serenity::all::standard::CommandResult;
use serenity::all::{CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, ResolvedOption, ResolvedValue, VoiceState};
use songbird::tracks::{Track};
use songbird::{input::File as SongbirdFile, Call};
use std::fs::File;
use std::io::Write;
use tokio::sync::MutexGuard;

const FILENAME: &str = "./audio.mp3";

async fn play(
    ctx: &Context,
    guild_id: GuildId,
    text: String,
    voice_state: Option<VoiceState>
) -> CommandResult {
        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        match voice(text.as_str()).await {
            Ok(_) => {
                println!("Audio file created successfully");
            }
            Err(e) => {
                println!("Error creating audio file: {}", e);
                return Ok(());
            }
        };

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;
            play_audio(&mut handler).await?;

        } else {
            // check_msg(
            //     channel_id
            //         .say(&ctx.http, "Not in a voice channel to play in")
            //         .await,
            // );
            match voice_state {
                Some(ref valid_vc) => {
                    let channel_id = valid_vc.channel_id.unwrap();
                    println!("User is in voice channel ID: {}", channel_id);
                    println!("Guild ID: {}", guild_id);

                    match manager.join(guild_id, channel_id).await {
                        Ok(_) => {
                            play_audio(&mut manager.get(guild_id).unwrap().lock().await).await?;
                        }
                        Err(_) => {
                            return Err("He isn't him.".into());
                        }
                    }
                }
                None => {
                    return Err("Not in a vc to play this audio".into());
                }
            }
        }
        Ok(())
}

async fn play_audio(handler: &mut MutexGuard<'_, Call>) -> CommandResult {
    if !std::path::Path::new(FILENAME).exists() {
        return Err(format!("Error: Could not find file {}", FILENAME).into());
    }
    let track = handler.play_only(Track::from(SongbirdFile::new(FILENAME)));
    println!("Track status: {:?}", track.get_info().await);
    Ok(())
}

async fn voice(text: &str) -> Result<(), String> {
    let endpoint = "https://tiktok-tts.weilnet.workers.dev";
    let voice = "en_us_001";

    let client = Client::new();
    let response = client
        .post(format!("{}/api/generation", endpoint))
        .header("Content-Type", "application/json")
        .json(&json!({
            "text": text,
            "voice": voice
        }))
        .send()
        .await
        .map_err(|_| "failed to get response");

    let response_data: Value = response?
        .json()
        .await
        .map_err(|_| "Failed to parse response")?;
    let data = response_data["data"]
        .as_str()
        .ok_or("No data field in response")?;

    let decoded = base64::decode(data).map_err(|_| "Failed to decode base64 data")?;
    let mut file = File::create(FILENAME).map_err(|_| "Failed to create file")?;
    file.write_all(&decoded)
        .map_err(|_| "Failed to write to file")?;

    println!("Audio saved to ./audio.mp3");
    Ok(())
}

pub async fn run(
    ctx: &Context,
    guild_id: &GuildId,
    voice_state: Option<VoiceState>,
    _options: &[ResolvedOption<'_>],
) -> CommandResult {
    let text: &str;
    if let Some(ResolvedOption {
        value: ResolvedValue::String(t),
        ..
    }) = _options.first()
    {
        text = t;
    } else {
        // This should never happen
        return Err("No text provided".into());
    };
    play(ctx, *guild_id, text.to_string(), voice_state).await
}

pub fn register() -> CreateCommand {
    CreateCommand::new("voice")
        .description("Text to jotavoice")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "text", "text to speech")
                .name("text")
                .description("text to speech")
                .kind(CommandOptionType::String)
                .required(true),
        )
}
