#![allow(deprecated)]

use serde_json::{json, Value};
use serenity::all::standard::{CommandError, CommandResult};
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, ResolvedOption,
    ResolvedValue, VoiceState,
};
use songbird::tracks::Track;
use songbird::{input::File as SongbirdFile, Call, Songbird};
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::MutexGuard;

const FILENAME: &str = "./audio.mp3";
const ENDPOINT: &str = "https://tiktok-tts.weilnet.workers.dev";

#[derive(Default, strum_macros::EnumString, strum_macros::Display)]
enum VoiceType {
    #[default]
    Male,
    Female,
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
enum VoiceLanguage {
    Portuguese(VoiceType),
    English(VoiceType),
    Japanese(VoiceType),
}

impl Into<&str> for VoiceLanguage {
    fn into(self) -> &'static str {
        match self {
            VoiceLanguage::Portuguese(voice) => match voice {
                VoiceType::Male => "br_005",
                VoiceType::Female => "br_001",
            },
            VoiceLanguage::English(voice) => match voice {
                VoiceType::Male => "",
                VoiceType::Female => "en_us_001",
            },
            VoiceLanguage::Japanese(voice) => match voice {
                VoiceType::Male => "jp_006",
                VoiceType::Female => "jp_001",
            },
        }
    }
}

async fn play(
    ctx: &Context,
    guild_id: GuildId,
    text: String,
    voice_state: Option<VoiceState>,
    voice_lang: VoiceLanguage,
) -> CommandResult {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    fetch_voice_from_tk_api(ctx, text.as_str(), voice_lang.into()).await?;
    join_vc(guild_id, voice_state, manager.clone()).await?;
    play_audio(
        &mut manager
            .get(guild_id)
            .ok_or(format!(
                "Failed to get manager (if this happened kill <@{}>)",
                crate::DISCORD_ID_LH
            ))?
            .lock()
            .await,
    )
    .await
}

async fn join_vc(
    guild_id: GuildId,
    voice_state: Option<VoiceState>,
    manager: Arc<Songbird>,
) -> CommandResult {
    match voice_state {
        Some(ref valid_vc) => match manager.join(guild_id, valid_vc.channel_id.unwrap()).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        },
        None => Err("You are not in a voice chat".into()),
    }
}

async fn play_audio(handler: &mut MutexGuard<'_, Call>) -> CommandResult {
    std::path::Path::new(FILENAME)
        .exists()
        .then(|| {})
        .ok_or::<CommandError>(format!("Error: Could not find file {}", FILENAME).into())?;
    let track = handler.play_only(Track::from(SongbirdFile::new(FILENAME)));
    println!("Track status: {:?}", track.get_info().await);
    Ok(())
}

async fn fetch_voice_from_tk_api(ctx: &Context, text: &str, voice: &str) -> CommandResult {
    let client = {
        let data = ctx.data.read().await;
        data.get::<crate::HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };
    let response = client
        .post(format!("{}/api/generation", ENDPOINT))
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
    options: &[ResolvedOption<'_>],
) -> CommandResult {
    let mut voice_lang = VoiceLanguage::Portuguese(VoiceType::Male);
    let text = options
        .iter()
        .find(|opt| opt.name == "text")
        .and_then(|opt| {
            if let ResolvedValue::String(t) = opt.value {
                Some(t.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| "No text provided".to_string())?;

    if let Some(voice_type_opt) = options.iter().find(|opt| opt.name == "voice_type") {
        if let ResolvedValue::String(voice_type_str) = voice_type_opt.value {
            let voice_type = VoiceType::from_str(voice_type_str)?;
            extract_voice_lang(options, &mut voice_lang, Some(voice_type))?;
        }
    } else {
        extract_voice_lang(options, &mut voice_lang, None)?;
    }

    play(ctx, *guild_id, text, voice_state, voice_lang).await
}

fn extract_voice_lang(
    options: &[ResolvedOption],
    voice_lang: &mut VoiceLanguage,
    voice_type: Option<VoiceType>,
) -> Result<(), CommandError> {
    if let Some(lang_opt) = options.iter().find(|opt| opt.name == "voice_language") {
        if let ResolvedValue::String(lang_str) = lang_opt.value {
            match VoiceLanguage::from_str(lang_str)? {
                VoiceLanguage::Portuguese(_) => {
                    *voice_lang = VoiceLanguage::Portuguese(voice_type.unwrap_or(VoiceType::Male))
                }
                VoiceLanguage::English(_) => {
                    *voice_lang = VoiceLanguage::English(voice_type.unwrap_or(VoiceType::Male))
                }
                VoiceLanguage::Japanese(_) => {
                    *voice_lang = VoiceLanguage::Japanese(voice_type.unwrap_or(VoiceType::Male))
                }
            }
        } else {
            *voice_lang = VoiceLanguage::Portuguese(voice_type.unwrap_or(VoiceType::Male))
        }
    }
    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("voice")
        .description("Text to jotavoice")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "text",
                "Text to convert to speech",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "voice_type",
                "Whether the voice is male or female",
            )
            .add_string_choice(VoiceType::Male.to_string(), VoiceType::Male.to_string())
            .add_string_choice(VoiceType::Female.to_string(), VoiceType::Female.to_string())
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "voice_language",
                "Language for the voice",
            )
            .add_string_choice(
                VoiceLanguage::English(VoiceType::Male).to_string(),
                VoiceLanguage::English(VoiceType::Male).to_string(),
            )
            .add_string_choice(
                VoiceLanguage::Portuguese(VoiceType::Male).to_string(),
                VoiceLanguage::Portuguese(VoiceType::Male).to_string(),
            )
            .add_string_choice(
                VoiceLanguage::Japanese(VoiceType::Male).to_string(),
                VoiceLanguage::Japanese(VoiceType::Male).to_string(),
            )
            .required(false),
        )
}
