use std::path::Path;
use tauri::command;

fn validate_hyperyap_extension(file_path: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("hyperyap") => Ok(()),
        _ => Err("File must have a .hyperyap extension".to_string()),
    }
}

#[command]
pub fn read_hyperyap_file(file_path: String) -> Result<String, String> {
    validate_hyperyap_extension(&file_path)?;
    std::fs::read_to_string(&file_path).map_err(|e| e.to_string())
}

#[command]
pub fn write_hyperyap_file(file_path: String, content: String) -> Result<(), String> {
    validate_hyperyap_extension(&file_path)?;
    std::fs::write(&file_path, content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_validate_hyperyap_extension_valid() {
        assert!(validate_hyperyap_extension("/tmp/config.hyperyap").is_ok());
    }

    #[test]
    fn test_validate_hyperyap_extension_invalid() {
        assert!(validate_hyperyap_extension("/tmp/config.json").is_err());
        assert!(validate_hyperyap_extension("/tmp/config").is_err());
        assert!(validate_hyperyap_extension("/tmp/config.txt").is_err());
    }

    #[test]
    fn test_write_and_read_hyperyap_file() {
        let path = "/tmp/test-hyperyap-rw.hyperyap";
        let content = r#"{"version":1}"#.to_string();

        let write_result = write_hyperyap_file(path.to_string(), content.clone());
        assert!(write_result.is_ok());

        let read_result = read_hyperyap_file(path.to_string());
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
        let result = read_hyperyap_file("/tmp/nonexistent-file.hyperyap".to_string());
        assert!(result.is_err());
    }
}
