use crate::img::assets::{FX_DUMMY, NF_DUMMY, ST_CHUNKS, ST_DUMMY};
use crate::img::locate::{extract_chunks, locate_dds_chunks, replace_chunks};
use crate::img::utils::{compress_image, resize_if_needed, save_dds_blob};
use directxtex::{DXGI_FORMAT, ScratchImage};
use image::imageops::FilterType;
use image::{ImageBuffer, Rgba};
use std::fs;
use std::path::Path;

pub fn convert_dds(
    in_path: &Path,
    width: u32,
    height: u32,
    format: DXGI_FORMAT,
) -> anyhow::Result<ScratchImage> {
    if width == 0 || height == 0 {
        anyhow::bail!("Invalid dimensions: width and height must be greater than 0")
    }

    let rgba_image = image::open(in_path)?.into_rgba8();
    let (orig_width, orig_height) = rgba_image.dimensions();

    let processed = if width != orig_width || height != orig_height {
        image::imageops::resize(&rgba_image, width, height, FilterType::Lanczos3)
    } else {
        rgba_image
    };

    let (width, height) = processed.dimensions();
    let mut pixel_vec = processed.into_raw();
    compress_image(width, height, format, &mut pixel_vec)
}

pub fn convert_bg(in_path: &Path) -> anyhow::Result<ScratchImage> {
    const FORMAT: DXGI_FORMAT = DXGI_FORMAT::DXGI_FORMAT_BC1_UNORM;
    convert_dds(in_path, 1920, 1080, FORMAT)
}

pub fn convert_jk(in_path: &Path) -> anyhow::Result<ScratchImage> {
    const FORMAT: DXGI_FORMAT = DXGI_FORMAT::DXGI_FORMAT_BC1_UNORM;
    convert_dds(in_path, 300, 300, FORMAT)
}

pub fn convert_fx(in_paths: &[Option<&Path>]) -> anyhow::Result<ScratchImage> {
    const TILE: u32 = 256;
    const CANVAS: u32 = TILE * 2;

    let mut output_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(CANVAS, CANVAS);
    let mut count = 0;

    for input_path_opt in in_paths.iter().take(4) {
        let input_path = match input_path_opt {
            Some(path) => path,
            None => continue,
        };

        let img = image::open(input_path)?;

        let img = resize_if_needed(img, TILE, TILE);
        let img = img.to_rgba8();
        let pixels = img.as_raw();

        let i = count as u32;
        let offset_x = (i % 2) * TILE;
        let offset_y = (i / 2) * TILE;

        for y in 0..TILE {
            let out_y = offset_y + y;
            for x in 0..TILE {
                let out_x = offset_x + x;
                let in_idx = ((y * TILE + x) * 4) as usize;
                let pixel = Rgba([
                    pixels[in_idx],
                    pixels[in_idx + 1],
                    pixels[in_idx + 2],
                    pixels[in_idx + 3],
                ]);
                output_buffer.put_pixel(out_x, out_y, pixel);
            }
        }

        count += 1;
        if count >= 4 {
            break;
        }
    }

    let mut pixel_data = output_buffer.into_raw();
    compress_image(
        CANVAS,
        CANVAS,
        DXGI_FORMAT::DXGI_FORMAT_BC3_UNORM,
        &mut pixel_data,
    )
}

pub fn extract_afb(in_path: &Path, out_folder: &str) -> anyhow::Result<()> {
    let data = fs::read(in_path)?;
    let chunks = locate_dds_chunks(&data);
    if chunks.is_empty() {
        anyhow::bail!("No .dds chunks found in the file");
    }
    let base_name = in_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("chunk");
    extract_chunks(&data, out_folder, base_name, ".dds", &chunks)
}

pub fn convert_stage(
    bg_in_path: &Path,
    fx_in_paths: &[Option<&Path>],
    st_out_path: &Path,
    nf_out_path: &Path,
) -> anyhow::Result<()> {
    let bg_dds = save_dds_blob(convert_bg(bg_in_path)?)?;
    let fx_dds = if fx_in_paths.iter().any(Option::is_some) {
        Some(save_dds_blob(convert_fx(fx_in_paths)?)?)
    } else {
        None
    };

    let bg_buffer = bg_dds.buffer();
    let fx_buffer = fx_dds.as_ref().map(|d| d.buffer()).or(Some(FX_DUMMY));

    let replacements = &[Some(bg_buffer), fx_buffer];
    replace_chunks(ST_DUMMY, st_out_path, &ST_CHUNKS, replacements)?;
    fs::write(nf_out_path, NF_DUMMY)?;
    Ok(())
}
