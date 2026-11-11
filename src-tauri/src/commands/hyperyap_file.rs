use std::path::{Component, Path};
use tauri::command;

const MAX_HYPERYAP_FILE_BYTES: usize = 2 * 1024 * 1024; // 2 MiB cap

fn validate_hyperyap_path(file_path: &str) -> Result<&Path, String> {
    let trimmed = file_path.trim();
    if trimmed.is_empty() {
        return Err("File path must not be empty".to_string());
    }
    let path = Path::new(trimmed);
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Refusing path traversal in .hyperyap file path".to_string());
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("hyperyap") => {}
        _ => return Err("File must have a .hyperyap extension".to_string()),
    }
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err("Refusing symlinked .hyperyap file".to_string());
        }
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Ok(pmeta) = std::fs::symlink_metadata(parent) {
                if pmeta.file_type().is_symlink() {
                    return Err("Refusing symlinked parent directory for .hyperyap file".to_string());
                }
            }
        }
    }
    Ok(path)
}

#[command]
pub fn read_hyperyap_file(file_path: String) -> Result<String, String> {
    let path = validate_hyperyap_path(&file_path)?;
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err("Target is not a regular .hyperyap file".to_string());
    }
    if meta.len() > MAX_HYPERYAP_FILE_BYTES as u64 {
        return Err(format!(
            "File exceeds maximum allowed size ({} bytes)",
            MAX_HYPERYAP_FILE_BYTES
        ));
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let _: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| "Invalid .hyperyap JSON format".to_string())?;
    Ok(content)
}

#[command]
pub fn write_hyperyap_file(file_path: String, content: String) -> Result<(), String> {
    let path = validate_hyperyap_path(&file_path)?;
    if content.len() > MAX_HYPERYAP_FILE_BYTES {
        return Err(format!(
            "Content exceeds maximum allowed size ({} bytes)",
            MAX_HYPERYAP_FILE_BYTES
        ));
    }
    let _: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| "Content must be valid JSON".to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_validate_hyperyap_extension_valid() {
        assert!(validate_hyperyap_path("/tmp/config.hyperyap").is_ok());
    }

    #[test]
    fn test_validate_hyperyap_extension_invalid() {
        assert!(validate_hyperyap_path("/tmp/config.json").is_err());
        assert!(validate_hyperyap_path("/tmp/config").is_err());
        assert!(validate_hyperyap_path("/tmp/config.txt").is_err());
        assert!(validate_hyperyap_path("/tmp/../etc/config.hyperyap").is_err());
    }

    #[test]
    fn test_write_and_read_hyperyap_file() {
        let path = std::env::temp_dir().join("test-hyperyap-rw.hyperyap");
        let path_str = path.to_string_lossy().to_string();
        let content = r#"{"version":1}"#.to_string();

        let write_result = write_hyperyap_file(path_str.clone(), content.clone());
        assert!(write_result.is_ok());

        let read_result = read_hyperyap_file(path_str);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), content);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_read_hyperyap_file_rejects_non_hyperyap() {
        let result = read_hyperyap_file("/tmp/config.json".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".hyperyap"));
    }

    #[test]
    fn test_write_hyperyap_file_rejects_non_hyperyap() {
        let result = write_hyperyap_file("/tmp/config.json".to_string(), "{}".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".hyperyap"));
    }

    #[test]
    fn test_read_hyperyap_file_nonexistent() {
        let path = std::env::temp_dir().join("nonexistent-file-xyz.hyperyap");
        let result = read_hyperyap_file(path.to_string_lossy().to_string());
        assert!(result.is_err());
    }
}
