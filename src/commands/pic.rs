use crate::G_USER_ID;
use image::GenericImageView;
use minifb::{Key, Window, WindowOptions};
#[allow(deprecated)]
use serenity::all::standard::CommandResult;
use serenity::all::{Context, ResolvedOption};
use serenity::builder::CreateCommand;

#[allow(deprecated)]
pub async fn run(ctx: &Context, _options: &[ResolvedOption<'_>]) -> CommandResult {
    let url = G_USER_ID.to_user(&ctx).await.unwrap().avatar_url().unwrap();

    let img_bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let img = image::load_from_memory(&*img_bytes).unwrap();

    display_image(img);

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("foto").description("Change the profile picture of the bot")
}
fn display_image(image: image::DynamicImage) {
    let (width, height) = image.dimensions();
    let raw_pixels = image.to_rgba8().into_raw();
    let buffer: Vec<u32> = raw_pixels
        .chunks_exact(4)
        .map(|chunk| {
            ((chunk[3] as u32) << 24)
                | ((chunk[0] as u32) << 16)
                | ((chunk[1] as u32) << 8)
                | (chunk[2] as u32)
        })
        .collect();

    let mut window = Window::new(
        "Display Image",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .expect("Failed to create a window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .expect("Failed to update window");
    }
}
