use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// Constants for keyring
const KEYRING_SERVICE: &str = "bytestashy";
const KEYRING_USERNAME: &str = "api_key";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub api_url: String,
    #[serde(skip)]
    pub api_key: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Option<Config>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "bytestashy") {
            let config_path: PathBuf = proj_dirs.config_dir().join("config.json");
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let mut cfg: Config = serde_json::from_str(&content)?;

                // Versuche, den API-Key aus dem Keyring zu laden
                match Self::get_api_key_from_keyring() {
                    Ok(api_key) => {
                        cfg.api_key = api_key;
                    }
                    Err(err) => {
                        return Err(anyhow::anyhow!(
                            "Error loading api key from keyring: {}",
                            err
                        ));
                    }
                }

                return Ok(Some(cfg));
            }
        }
        Ok(None)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        Self::save_api_key_to_keyring(&self.api_key)?;

        if let Some(proj_dirs) = ProjectDirs::from("", "", "bytestashy") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let config_path = config_dir.join("config.json");
            let mut file = fs::File::create(&config_path)?;

            let json = serde_json::to_string_pretty(self)?;
            file.write_all(json.as_bytes())?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&config_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&config_path, perms)?;
            }
            Ok(())
        } else {
            anyhow::bail!("Could not save config file. Could not determine project directory.");
        }
    }

    fn save_api_key_to_keyring(api_key: &str) -> anyhow::Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;
        entry.set_password(api_key)?;
        Ok(())
    }

    fn get_api_key_from_keyring() -> anyhow::Result<String> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;
        Ok(entry.get_password()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config {
            api_url: "https://example.com".to_string(),
            api_key: "test-key".to_string(),
        };

        assert_eq!(config.api_url, "https://example.com");
        assert_eq!(config.api_key, "test-key");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            api_url: "https://example.com".to_string(),
            api_key: "test-key".to_string(), // This should be skipped in serialization
        };

        let json = serde_json::to_string(&config).unwrap();

        // api_key should be skipped due to #[serde(skip)]
        assert!(json.contains("api_url"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("test-key"));
    }

    #[test]
    fn test_config_deserialization() {
        let json = r#"{"api_url":"https://example.com"}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.api_url, "https://example.com");
        assert_eq!(config.api_key, ""); // Default empty string for skipped field
    }
}
