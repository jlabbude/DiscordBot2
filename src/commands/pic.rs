use serenity::all::ResolvedOption;
use serenity::builder::CreateCommand;

pub fn run(_options: &[ResolvedOption]) -> String {
    todo!()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("foto").description("Change the profile picture of the bot")
}
