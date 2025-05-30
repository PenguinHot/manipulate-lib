#![allow(clippy::module_inception)]

#[cfg(test)]
mod test {
    use crate::img::convert::{convert_dds, convert_stage};
    use crate::img::utils::*;
    use crate::img::{convert_fx, extract_afb};
    use anyhow::Result;
    use directxtex::DXGI_FORMAT;
    use std::path::{Path, PathBuf};

    fn get_temp_image(_dir: &Path, _width: u32, _height: u32) -> PathBuf {
        let random_num = rand::random_range(1..=4);
        Path::new("tests").join(format!("{}.jpg", random_num))
    }

    #[test]
    fn test_is_valid_image_empty_path() -> Result<()> {
        let result = is_valid_image(Path::new(""));
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_is_valid_image_nonexistent_path() -> Result<()> {
        let result = is_valid_image(Path::new("nonexistent_file.png"));
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_is_valid_image_valid_image() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img_path = get_temp_image(temp_dir, 100, 100);
        let result = is_valid_image(&img_path);
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_convert_dds_invalid_dimensions() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img_path = get_temp_image(temp_dir, 100, 100);
        let img = convert_dds(&img_path, 0, 100, DXGI_FORMAT::DXGI_FORMAT_BC1_UNORM);
        assert!(img.is_err());
        Ok(())
    }

    #[test]
    fn test_convert_dds_success() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img_path = get_temp_image(temp_dir, 100, 100);
        let out_path = temp_dir.join("output.dds");
        let img = convert_dds(&img_path, 100, 100, DXGI_FORMAT::DXGI_FORMAT_BC1_UNORM);
        let result = save_dds_file(img?, &out_path);
        assert!(result.is_ok());
        assert!(out_path.exists());
        Ok(())
    }

    #[test]
    fn test_convert_dds_resize() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img_path = get_temp_image(temp_dir, 100, 100);
        let out_path = temp_dir.join("output_resized.dds");
        let img = convert_dds(&img_path, 50, 50, DXGI_FORMAT::DXGI_FORMAT_BC1_UNORM);
        let result = save_dds_file(img?, &out_path);
        assert!(result.is_ok());
        assert!(out_path.exists());
        Ok(())
    }

    #[test]
    fn test_convert_fx_success() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img1 = get_temp_image(temp_dir, 256, 256);
        let img2 = get_temp_image(temp_dir, 256, 256);
        let img3 = get_temp_image(temp_dir, 256, 256);
        let out_path = temp_dir.join("output_fx.dds");
        let inputs = [
            Some(img1.as_path()),
            Some(img2.as_path()),
            None,
            Some(img3.as_path()),
        ];
        let img = convert_fx(&inputs)?;
        let result = save_dds_file(img, &out_path);
        assert!(result.is_ok());
        assert!(out_path.exists());
        Ok(())
    }

    #[test]
    fn test_convert_fx_with_resize() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        let img1 = get_temp_image(temp_dir, 100, 100);
        let img2 = get_temp_image(temp_dir, 200, 200);
        let img3 = get_temp_image(temp_dir, 300, 300);
        let img4 = get_temp_image(temp_dir, 400, 400);
        let out_path = temp_dir.join("output_fx_resize.dds");
        let inputs = [
            Some(img1.as_path()),
            Some(img2.as_path()),
            Some(img3.as_path()),
            Some(img4.as_path()),
        ];
        let img = convert_fx(&inputs)?;
        let result = save_dds_file(img, &out_path);
        assert!(result.is_ok());
        assert!(out_path.exists());
        Ok(())
    }

    #[test]
    fn test_extract_afb() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        _ = std::fs::create_dir(temp_dir);

        let afb_path = Path::new("test_assets/test.afb");
        let out_folder = temp_dir.to_str().unwrap();
        extract_afb(afb_path, out_folder)?;
        assert!(temp_dir.join("test_0001.dds").exists());
        assert!(temp_dir.join("test_0002.dds").exists());
        Ok(())
    }

    #[test]
    fn test_convert_stage() -> Result<()> {
        let temp_dir = Path::new("test_assets/output");
        _ = std::fs::create_dir(temp_dir);

        let bg_image = Path::new("test_assets/bg.png");

        let st_output = temp_dir.join("output_st.afb");
        let nf_output = temp_dir.join("output_nf.afb");

        convert_stage(bg_image, &[], &st_output, &nf_output)?;

        assert!(st_output.exists());
        assert!(nf_output.exists());

        extract_afb(&st_output, temp_dir.to_str().unwrap())?;

        Ok(())
    }

    #[test]
    fn test_convert_stage_invalid_input() {
        let temp_dir = Path::new("test_assets/output");
        let invalid_path = temp_dir.join("nonexistent.jpg");
        let output_path = temp_dir.join("output.afb");

        let result = convert_stage(&invalid_path, &[None, None], &output_path, &output_path);
        assert!(result.is_err(), "Should fail with invalid background image");

        let bg_image = get_temp_image(temp_dir, 1024, 1024);
        let result = convert_stage(
            &bg_image,
            &[Some(&invalid_path), None],
            &output_path,
            &output_path,
        );
        assert!(result.is_err(), "Should fail with invalid FX image");
    }
}
