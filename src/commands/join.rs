use serenity::all::{GuildId, UserId, VoiceState};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::client::Context;
#[allow(deprecated)]
use serenity::framework::standard::CommandResult;
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

#[allow(deprecated)]
pub async fn run(ctx: &Context, options: &[ResolvedOption<'_>]) -> CommandResult {
    let user_id: UserId;

    if let Some(ResolvedOption {
        value: ResolvedValue::User(&ref id, _partial_member),
        ..
    }) = options.first()
    {
        user_id = id.id;
    } else {
        // This should never happen
        return Err("No user mentioned".into());
    };

    let guild_id = ctx
        .cache
        .guilds()
        .get(0)
        .ok_or("No guilds found in cache")?
        .clone();

    if let Some(voice_state) = get_voice_state(ctx.clone(), guild_id, user_id).await {
        let channel_id = voice_state.channel_id.unwrap();
        println!("User is in voice channel ID: {}", channel_id);
        println!("Guild ID: {}", guild_id);

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        manager
            .join(guild_id, channel_id)
            .await
            .expect("TODO: panic message");
    } else {
        return Err("User is not in a voice channel".into());
    }

    Ok(())
}

async fn get_voice_state(ctx: Context, guild_id: GuildId, user_id: UserId) -> Option<VoiceState> {
    let guild = match ctx.cache.guild(guild_id) {
        Some(guild) => guild,
        None => return None,
    };
    let voice_states = &guild.voice_states;
    let voice_state = voice_states.get(&user_id);
    voice_state.cloned()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("join")
        .description("join command")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "mention", "mention")
                .name("user")
                .description("The user to join the voice channel")
                .kind(CommandOptionType::User)
                .required(true),
        )
}
