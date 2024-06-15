use serenity::all::{GuildId, PartialMember, UserId, VoiceState};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub(crate) async fn run(ctx: &Context, options: &[ResolvedOption<'_>]) -> CommandResult {

    let member: PartialMember;

    if let Some(ResolvedOption {
                    value: ResolvedValue::User(&ref _id, partial_member), ..
                }) = options.first(){
        member = partial_member.clone().unwrap().to_owned();
    }
    else {
        return Err("No user found in options".into());
    };

    // Get the guild ID from the context
    let guild_id = ctx.cache.guilds().get(0)
    .ok_or("No guilds found in cache")?
    .clone();

    let user_id = match member.user {
        Some(user) => user.id,
        None => return Err("User field is None".into()),
    };

    let voice_state: VoiceState = get_voice_state(ctx.clone(), guild_id, user_id).await.expect("REASON");

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
