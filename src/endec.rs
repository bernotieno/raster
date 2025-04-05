//!  A module for encoding/decoding.

// from rust
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

// from external crate
use gif;
use png;

// from local crate
use error::{RasterError, RasterResult};
use Image;
use ImageFormat;

// Decode GIF
pub fn decode_gif(image_file: &File) -> RasterResult<Image> {
    let mut decoder = gif::Decoder::new(image_file);

    // Configure the decoder such that it will expand the image to RGBA.
    gif::SetParameter::set(&mut decoder, gif::ColorOutput::RGBA);

    // Read the file header
    let mut reader = decoder.read_info()?;

    // Read frame 1.
    // TODO: Work on all frames
    if let Some(_) = reader.next_frame_info()? {
        let mut bytes = vec![0; reader.buffer_size()];
        reader.read_into_buffer(&mut bytes)?;
        Ok(Image {
            width: reader.width() as i32,
            height: reader.height() as i32,
            bytes: bytes,
        })
    } else {
        Err(RasterError::Decode(
            ImageFormat::Gif,
            "Error getting frame info".to_string(),
        ))
    }
}

// Encode GIF
pub fn encode_gif(image: &Image, path: &Path) -> RasterResult<()> {
    // Open the file with basic error check
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let frame = gif::Frame::from_rgba(
        image.width as u16,
        image.height as u16,
        &mut image.bytes.clone(),
    ); // TODO: Perf issue?
    let mut encoder = gif::Encoder::new(writer, frame.width, frame.height, &[])?;
    encoder.write_frame(&frame).map_err(RasterError::Io)?;
    Ok(())
}

// Decode PNG
pub fn decode_png(image_file: &File) -> RasterResult<Image> {
    let decoder = png::Decoder::new(image_file);
    let mut reader = decoder.read_info()?;
    let mut bytes = vec![0; reader.output_buffer_size()];

    reader.next_frame(&mut bytes)?;
    let info = reader.info();

    // Handle different color types
    match info.color_type {
        png::ColorType::Rgb => {
            // Convert RGB to RGBA by adding alpha channel
            let mut rgba_bytes = Vec::with_capacity((info.width * info.height) as usize * 4);
            for i in 0..(info.width * info.height) as usize {
                let idx = i * 3;
                rgba_bytes.extend_from_slice(&bytes[idx..idx + 3]);
                rgba_bytes.push(255); // Add alpha channel (fully opaque)
            }
            bytes = rgba_bytes;
        }
        png::ColorType::Grayscale => {
            // Convert grayscale to RGBA
            let mut rgba_bytes = Vec::with_capacity((info.width * info.height) as usize * 4);
            for i in 0..(info.width * info.height) as usize {
                let gray = bytes[i];
                rgba_bytes.push(gray);
                rgba_bytes.push(gray);
                rgba_bytes.push(gray);
                rgba_bytes.push(255); // Add alpha channel (fully opaque)
            }
            bytes = rgba_bytes;
        }
        png::ColorType::GrayscaleAlpha => {
            // Convert grayscale+alpha to RGBA
            let mut rgba_bytes = Vec::with_capacity((info.width * info.height) as usize * 4);
            for i in 0..(info.width * info.height) as usize {
                let idx = i * 2;
                let gray = bytes[idx];
                let alpha = bytes[idx + 1];
                rgba_bytes.push(gray);
                rgba_bytes.push(gray);
                rgba_bytes.push(gray);
                rgba_bytes.push(alpha);
            }
            bytes = rgba_bytes;
        }
        png::ColorType::Indexed => {
            // Convert indexed to RGBA
            let mut rgba_bytes = Vec::with_capacity((info.width * info.height) as usize * 4);
            let palette = info.palette.as_ref().ok_or_else(|| {
                RasterError::Decode(
                    ImageFormat::Png,
                    "Missing palette for indexed image".to_string(),
                )
            })?;

            for i in 0..(info.width * info.height) as usize {
                let idx = bytes[i] as usize * 3;
                if idx + 2 < palette.len() {
                    rgba_bytes.push(palette[idx]);
                    rgba_bytes.push(palette[idx + 1]);
                    rgba_bytes.push(palette[idx + 2]);
                    rgba_bytes.push(255); // Add alpha channel (fully opaque)
                } else {
                    // Handle out of bounds palette index
                    rgba_bytes.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
            bytes = rgba_bytes;
        }
        png::ColorType::Rgba => {
            // Already in RGBA format, no conversion needed
        }
    }

    Ok(Image {
        width: info.width as i32,
        height: info.height as i32,
        bytes: bytes,
    })
}

// Encode PNG
pub fn encode_png(image: &Image, path: &Path) -> RasterResult<()> {
    // Open the file with basic error check
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, image.width as u32, image.height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image.bytes)?;
    Ok(())
}
