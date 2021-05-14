use std::sync::Once;
use magick_rust::{magick_wand_genesis, MagickWand};

static START: Once = Once::new();

pub(crate) fn resize(data: &Vec<u8>) -> Result<Vec<u8>, &'static str> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let wand = MagickWand::new();
    wand.read_image_blob(data)?;

    let new_width = wand.get_image_width() / 5;
    let new_height = wand.get_image_height() / 5;
    wand.adaptive_resize_image(new_width as usize, new_height as usize)?;

    let (res_x, res_y) = wand.get_image_resolution()?;
    let new_res_x = res_x * 0.55;
    let new_res_y = res_y * 0.55;

    wand.set_resolution(new_res_x, new_res_y)?;

    wand.write_image_blob("jpeg")
}