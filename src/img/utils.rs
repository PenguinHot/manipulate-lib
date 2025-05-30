use anyhow::Result;
use directxtex::{
    Blob, CP_FLAGS_NONE, DDS_FLAGS, DXGI_FORMAT, Image, ScratchImage, TEX_COMPRESS_DEFAULT,
};
use image::imageops::FilterType;
use std::io::Read;
use std::path::Path;

pub fn is_valid_image(in_path: &Path) -> Result<()> {
    // https://docs.rs/image/latest/image/fn.guess_format.html
    if in_path.extension().and_then(|s| s.to_str()) == Some("tga") {
        return Ok(());
    }

    let mut file = std::fs::File::open(in_path)?;
    let mut buffer = [0; 32];
    file.read_exact(&mut buffer)?;

    image::guess_format(&buffer)?;
    Ok(())
}

pub(crate) fn compress_image(
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
    pixel_data: &mut Vec<u8>,
) -> Result<ScratchImage> {
    let image = Image {
        width: width as usize,
        height: height as usize,
        format: DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM,
        row_pitch: width as usize * 4,
        slice_pitch: width as usize * height as usize * 4,
        pixels: pixel_data.as_mut_ptr(),
    };

    if image.format != format {
        image
            .compress(format, TEX_COMPRESS_DEFAULT, 0.5)
            .map_err(|e| anyhow::anyhow!("Failed to compress image: {}", e))
    } else {
        let mut scratch_image = ScratchImage::default();
        scratch_image.initialize_from_image(&image, true, CP_FLAGS_NONE)?;
        Ok(scratch_image)
    }
}

pub fn save_dds_file(img: ScratchImage, out_path: &Path) -> Result<()> {
    let blob = img.save_dds(DDS_FLAGS::DDS_FLAGS_NONE)?;
    std::fs::write(out_path, blob.buffer())?;
    Ok(())
}

pub fn save_dds_blob(img: ScratchImage) -> Result<Blob> {
    img.save_dds(DDS_FLAGS::DDS_FLAGS_NONE)
        .map_err(|e| anyhow::anyhow!("Failed to save DDS blob: {}", e))
}

pub(crate) fn resize_if_needed(
    img: image::DynamicImage,
    target_width: u32,
    target_height: u32,
) -> image::DynamicImage {
    if img.width() != target_width || img.height() != target_height {
        img.resize_exact(target_width, target_height, FilterType::Lanczos3)
    } else {
        img
    }
}
