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
                    value: ResolvedValue::User(&ref id, _partial_member), ..
                }
    ) = options.first() {
        user_id = id.id;
    }
    else {
        return Err("No user found in options".into());
    };

    let guild_id = ctx.cache.guilds().get(0)
        .ok_or("No guilds found in cache")?
        .clone();

    let voice_state: VoiceState =
        get_voice_state(
            ctx.clone(),
            guild_id,
            user_id)
        .await
        .expect("REASON");

    if let Some(channel_id) = voice_state.channel_id {
        println!("User is in voice channel ID: {}", channel_id);
        println!("Guild ID: {}", guild_id);
    } else {
        println!("User is not in a voice channel");
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
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "mention", "mention")
            .name("user")
            .description("The user to join the voice channel")
            .kind(CommandOptionType::User)
            .required(true)
        )
}
