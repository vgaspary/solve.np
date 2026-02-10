use pulldown_cmark::Parser;
use pulldown_cmark::html;
use std::fs::metadata;
use std::io::Error;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

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
    let round = |num: f64| (num * 100.0).floor() / 100.0;

    match bytes {
        0..1_000 => format!("{bytes}B"),
        1_000..10_000 => format!("{:.2}KB", round(bytes as f64 / 1e3)),
        10_000..1_000_000 => format!("{}KB", bytes / 1000),
        1_000_000..10_000_000 => format!("{:.2}MB", round(bytes as f64 / 1e6)),
        10_000_000..1_000_000_000 => format!("{}MB", bytes / 1_000_000),
        1_000_000_000..10_000_000_000 => format!("{:.2}GB", round(bytes as f64 / 1e9)),
        10_000_000_000.. => format!("{}GB", bytes / 1_000_000_000),
    }
}

pub(crate) fn md_to_html(md_content: String) -> String {
    let mut html_content = String::new();
    html::push_html(&mut html_content, Parser::new(&md_content));
    html_content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_string_h() {
        assert_eq!(bytes_to_string_h(0), "0B");
        assert_eq!(bytes_to_string_h(100), "100B");
        assert_eq!(bytes_to_string_h(999), "999B");
        assert_eq!(bytes_to_string_h(1_000), "1.00KB");
        assert_eq!(bytes_to_string_h(1_500), "1.50KB");
        assert_eq!(bytes_to_string_h(9_999), "9.99KB");
        assert_eq!(bytes_to_string_h(10_000), "10KB");
        assert_eq!(bytes_to_string_h(15_000), "15KB");
        assert_eq!(bytes_to_string_h(999_999), "999KB");
        assert_eq!(bytes_to_string_h(1_000_000), "1.00MB");
        assert_eq!(bytes_to_string_h(1_500_000), "1.50MB");
        assert_eq!(bytes_to_string_h(9_999_999), "9.99MB");
        assert_eq!(bytes_to_string_h(10_000_000), "10MB");
        assert_eq!(bytes_to_string_h(15_000_000), "15MB");
        assert_eq!(bytes_to_string_h(999_999_999), "999MB");
        assert_eq!(bytes_to_string_h(1_000_000_000), "1.00GB");
        assert_eq!(bytes_to_string_h(1_500_000_000), "1.50GB");
        assert_eq!(bytes_to_string_h(9_999_999_999), "9.99GB");
        assert_eq!(bytes_to_string_h(10_000_000_000), "10GB");
    }
}
