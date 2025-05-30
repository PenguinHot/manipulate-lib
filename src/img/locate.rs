use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn locate_dds_chunks(input: &[u8]) -> Vec<(usize, usize)> {
    const DDS_HEADER: &[u8] = &[0x44, 0x44, 0x53, 0x20]; // "DDS "
    const DDS_STOP_SIGN: &[u8] = &[0x50, 0x4F, 0x46, 0x30]; // "POF0"

    locate_chunks(input, DDS_HEADER, DDS_STOP_SIGN)
}

pub fn locate_chunks(input: &[u8], header: &[u8], stop_sign: &[u8]) -> Vec<(usize, usize)> {
    let mut chunks = Vec::new();
    let mut current_pos = 0;

    while let Some(start) = find_chunks(input, header, current_pos) {
        let stop_pos = find_chunks(input, stop_sign, start + header.len());
        let next_header = find_chunks(input, header, start + header.len());

        match (stop_pos, next_header) {
            (Some(stop), Some(next)) => {
                let end = std::cmp::min(stop, next);
                chunks.push((start, end));
                current_pos = end;
            }
            (Some(stop), None) => {
                chunks.push((start, stop));
                current_pos = stop;
            }
            (None, Some(next)) => {
                chunks.push((start, next));
                current_pos = next;
            }
            (None, None) => {
                chunks.push((start, input.len()));
                break;
            }
        }
    }

    chunks
}

pub fn extract_chunks(
    input: &[u8],
    out_folder: &str,
    base_name: &str,
    extension: &str,
    chunks: &[(usize, usize)],
) -> Result<()> {
    for (i, &(start, end)) in chunks.iter().enumerate() {
        let file_name = format!("{}_{:04}{}", base_name, i + 1, extension);
        let path = Path::new(out_folder).join(file_name);
        let mut file = File::create(path)?;
        file.write_all(&input[start..end])?;
    }
    Ok(())
}

pub fn replace_chunks(
    input: &[u8],
    out_path: &Path,
    chunks: &[(usize, usize)],
    replacements: &[Option<&[u8]>],
) -> Result<()> {
    if replacements.len() < chunks.len() {
        anyhow::bail!(
            "Replacements length ({}) must be at least equal to chunks length ({})",
            replacements.len(),
            chunks.len()
        );
    }

    let mut out_file = File::create(out_path)?;
    let mut cursor = 0;

    for (i, &(s, e)) in chunks.iter().enumerate() {
        out_file.write_all(&input[cursor..s])?;

        match &replacements[i] {
            Some(data) => out_file.write_all(data)?,
            None => out_file.write_all(&input[s..e])?,
        }

        cursor = e;
    }

    out_file.write_all(&input[cursor..])?;

    Ok(())
}

fn find_chunks(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if start >= haystack.len() || needle.is_empty() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|pos| pos + start)
}
