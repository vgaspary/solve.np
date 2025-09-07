use std::io::Error;
use std::{fs::metadata, os::unix::fs::MetadataExt, path::PathBuf};

pub(crate) fn get_file_metadata(path: &PathBuf) -> std::io::Result<(String, u64)> {
    let file_name: String = path
        .file_name()
        .map(|file_name| file_name.to_string_lossy().into())
        .ok_or(Error::other(format!(
            "Failed to extract the filename from path: {path:?}"
        )))?;
    let file_size = metadata(path)?.size();
    Ok((file_name, file_size))
}

pub(crate) fn bytes_to_string_h(bytes: u64) -> String {
    match bytes {
        0..1_000 => format!("{bytes}B"),
        1_000..10_000 => format!("{:.2}KB", bytes as f64 / 1e3),
        10_000..1_000_000 => format!("{}KB", bytes / 1000),
        1_000_000..10_000_000 => format!("{:.2}MB", bytes as f64 / 1e6),
        10_000_000..1_000_000_000 => format!("{}MB", bytes / 1_000_000),
        1_000_000_000..10_000_000_000 => format!("{:.2}GB", bytes as f64 / 1e9),
        10_000_000_000.. => format!("{}GB", bytes / 1_000_000_000),
    }
}
