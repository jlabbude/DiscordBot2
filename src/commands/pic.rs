use image::{DynamicImage, GenericImageView};
#[allow(deprecated)]
use serenity::all::standard::CommandResult;
use serenity::all::{Context, ResolvedOption};
use serenity::builder::CreateCommand;

use crate::G_USER_ID;

#[allow(deprecated)]
pub async fn run(ctx: &Context, _options: &[ResolvedOption<'_>]) -> CommandResult {
    Ok(ctx.http
        .get_current_user()
        .await?
        .edit(
            &ctx,
            serenity::builder::EditProfile::new().avatar(
                &serenity::builder::CreateAttachment::bytes(
                    overlay_images(
                        image::load_from_memory(
                            &reqwest::get(
                                G_USER_ID
                                    .to_user(&ctx)
                                    .await
                                    .unwrap()
                                    .avatar_url()
                                    .ok_or("No avatar")?,
                            )
                                .await
                                .unwrap()
                                .bytes()
                                .await?,
                        )?,
                        image::load_from_memory(include_bytes!("../assets/nosign.png"))?,
                    )
                        .as_bytes(),
                    "pfp.png",
                ),
            ),
        )
        .await?)
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
