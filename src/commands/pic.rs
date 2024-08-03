use image::{DynamicImage, EncodableLayout, GenericImageView};
use minifb::{Window, WindowOptions};
#[allow(deprecated)]
use serenity::all::standard::CommandResult;
use serenity::all::{Context, ResolvedOption};
use serenity::builder::CreateCommand;

use crate::G_USER_ID;

fn _display_image(image: &DynamicImage) {
    // This function is just for testing purposes
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

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .expect("Failed to update window");
    }
}

pub async fn create_new_pfp(
    ctx: &Context,
) -> Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>> {
    let nosign = image::load_from_memory(include_bytes!("../assets/nosign.png"))?;

    let overlay = overlay_images(
        image::load_from_memory(
            reqwest::get(
                G_USER_ID
                    .to_user(&ctx.http)
                    .await?
                    .avatar_url()
                    .ok_or("No avatar")?,
            )
            .await?
            .bytes()
            .await?
            .as_bytes(),
        )?
        .resize(
            nosign.dimensions().0,
            nosign.dimensions().1,
            image::imageops::FilterType::Lanczos3,
        ),
        nosign,
    );
    overlay.save("pfp.png")?;

    //let test = serenity::builder::CreateAttachment::bytes(overlay.into_bytes(), "pfp.png"); FUCK MY STUPID BAKA LIFE

    ctx.http
        .get_current_user()
        .await?
        .edit(
            &ctx,
            serenity::builder::EditProfile::new().avatar(
                &serenity::builder::CreateAttachment::file(
                    &tokio::fs::File::open("pfp.png").await?,
                    "pfp2.png",
                )
                .await?,
            ),
        )
        .await?;

    std::fs::remove_file("pfp.png")?; // CreateAttachment::bytes doesn't work so this'll have to do until it's fixed

    Ok(())
}

#[allow(deprecated)]
pub async fn run(ctx: &Context, _options: &[ResolvedOption<'_>]) -> CommandResult {
    create_new_pfp(ctx).await // Separate function so I can access it from main as well
}

pub fn register() -> CreateCommand {
    CreateCommand::new("foto").description("Change the profile picture of the bot")
}

fn overlay_images(pfp: DynamicImage, no_sign: DynamicImage) -> DynamicImage {
    let (pfp_width, pfp_height) = pfp.dimensions();

    let mut layered_image = image::RgbaImage::new(pfp_width, pfp_height);
    image::imageops::overlay(&mut layered_image, &pfp, 0, 0);
    image::imageops::overlay(&mut layered_image, &no_sign, 0, 0);

    DynamicImage::from(layered_image)
}
