mod api;
pub mod img;

use crate::api::{SUCCESS, set_error_msg, wchar_arr_to_vec, wchar_to_string};
use std::ffi::c_int;
use std::path::Path;

api!(validate_image(in_path: *const u16) {
    check_null_ptr!(in_path);
    let path_str = wchar_to_string(in_path)?;
    img::is_valid_image(Path::new(&path_str))
});

api!(extract_afb(
    in_path: *const u16,
    out_folder: *const u16
) {
    check_null_ptr!(in_path);
    check_null_ptr!(out_folder);

    let in_path_str = wchar_to_string(in_path)?;
    let out_folder_str = wchar_to_string(out_folder)?;

    img::extract_afb(Path::new(&in_path_str), &out_folder_str)
});

api!(convert_stage(
    bg_in_path: *const u16,
    fx_in_paths: *const *const u16,
    fx_in_paths_count: c_int,
    st_out_path: *const u16,
    nf_out_path: *const u16
) {
    check_null_ptr!(bg_in_path);
    check_null_ptr!(st_out_path);
    check_null_ptr!(nf_out_path);
    if fx_in_paths.is_null() && fx_in_paths_count > 0 {
        anyhow::bail!("NULL received for fx_in_paths while count is greater than 0");
    }

    let in_path_str = wchar_to_string(bg_in_path)?;
    let fx_path_vec = wchar_arr_to_vec(fx_in_paths, fx_in_paths_count)?;
    let st_out_path_str = wchar_to_string(st_out_path)?;
    let nf_out_path_str = wchar_to_string(nf_out_path)?;

    let fx_in_paths: Vec<Option<&Path>> = fx_path_vec
        .iter()
        .map(|opt_str| opt_str.as_ref().map(Path::new))
        .collect();

    img::convert_stage(
        Path::new(in_path_str.as_str()),
        &fx_in_paths,
        Path::new(st_out_path_str.as_str()),
        Path::new(nf_out_path_str.as_str()),
    )
});

api!(convert_jk(
    in_path: *const u16,
    out_path: *const u16
) {
    check_null_ptr!(in_path);
    check_null_ptr!(out_path);

    let in_path_str = wchar_to_string(in_path)?;
    let out_path_str = wchar_to_string(out_path)?;

    let dds = img::convert_jk(Path::new(&in_path_str))?;
    img::save_dds_file(dds, Path::new(&out_path_str))
});
