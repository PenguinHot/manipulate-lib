use anyhow::{Context, Result};
use std::ptr;

pub const SUCCESS: i32 = 0;
pub const FAILURE: i32 = 1;

#[macro_export]
macro_rules! check_null_ptr {
    ($ptr:expr) => {
        if $ptr.is_null() {
            anyhow::bail!("NULL received for {}", stringify!($ptr));
        }
    };
}

#[macro_export]
macro_rules! api {
    ($func_name:ident($($param:ident: $type:ty),*) $body:block) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $func_name(
            $($param: $type,)*
            error_buffer: *mut u16,
            error_buffer_size: std::os::raw::c_int,
        ) -> std::os::raw::c_int {
            let result = (|| -> anyhow::Result<()> {
                $body
            })();

            match result {
                Ok(_) => SUCCESS,
                Err(err) => set_error_msg(error_buffer, error_buffer_size, &err),
            }
        }
    };
}

pub fn set_error_msg(
    error_buffer: *mut u16,
    error_buffer_size: i32,
    err: impl std::fmt::Display,
) -> i32 {
    if error_buffer.is_null() || error_buffer_size <= 0 {
        return FAILURE;
    }

    let msg = format!("{:#}", err);
    let utf16_msg: Vec<u16> = msg.encode_utf16().collect();
    let max_len = (error_buffer_size as usize).saturating_sub(1);

    unsafe {
        let copy_len = utf16_msg.len().min(max_len);
        ptr::copy_nonoverlapping(utf16_msg.as_ptr(), error_buffer, copy_len);
        *error_buffer.add(copy_len) = 0;
    }

    FAILURE
}

pub fn wchar_to_string(w_char_p: *const u16) -> Result<String> {
    unsafe {
        check_null_ptr!(w_char_p);

        let mut len = 0;
        while *w_char_p.offset(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(w_char_p, len as usize);

        String::from_utf16(slice)
            .with_context(|| format!("Invalid UTF-16 sequence in wchar_t* ({:?})", w_char_p))
    }
}

pub fn wchar_arr_to_vec(ptr: *const *const u16, len: i32) -> Result<Vec<Option<String>>> {
    if len < 0 {
        anyhow::bail!("Invalid length: {}", len);
    }

    if ptr.is_null() || len == 0 {
        return Ok(Vec::new());
    }

    unsafe {
        let paths = std::slice::from_raw_parts(
            ptr,
            usize::try_from(len).context("Invalid length for wchar_t**")?,
        )
        .iter()
        .map(|&item| {
            if item.is_null() {
                Ok(None)
            } else {
                wchar_to_string(item).map(Some)
            }
        })
        .collect::<std::result::Result<Vec<Option<String>>, _>>()?;

        Ok(paths)
    }
}
