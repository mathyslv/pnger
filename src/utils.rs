use std::io::BufWriter;
use crate::error::PngerError;

/// Setup PNG encoder from decoder info
pub fn setup_png_encoder<'a>(info: &png::Info, writer: &'a mut BufWriter<Vec<u8>>) -> Result<png::Encoder<'a, &'a mut BufWriter<Vec<u8>>>, PngerError> {
    let mut encoder = png::Encoder::new(writer, info.width, info.height);
    encoder.set_color(info.color_type);
    encoder.set_depth(info.bit_depth);
    encoder.set_compression(info.compression);
    encoder.set_pixel_dims(info.pixel_dims);
    
    copy_png_metadata(info, &mut encoder);
    
    Ok(encoder)
}

/// Copy metadata from source PNG to destination encoder
pub fn copy_png_metadata<'a>(info: &png::Info, encoder: &mut png::Encoder<'a, &'a mut BufWriter<Vec<u8>>>) {
    if let Some(palette) = &info.palette {
        encoder.set_palette(palette.to_vec());
    }
    if let Some(animation) = &info.animation_control {
        let _ = encoder.set_animated(animation.num_frames, animation.num_plays);
    }
    if let Some(trns) = &info.trns {
        encoder.set_trns(trns.to_vec());
    }
    if let Some(source_gamma) = &info.source_gamma {
        encoder.set_source_gamma(*source_gamma);
    }
    if let Some(source_chromaticities) = &info.source_chromaticities {
        encoder.set_source_chromaticities(*source_chromaticities);
    }
    if let Some(srgb) = &info.srgb {
        encoder.set_source_srgb(*srgb);
    }
}