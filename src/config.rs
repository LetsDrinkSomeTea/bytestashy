// src/config.rs
use directories::ProjectDirs;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

// Constants for keyring
const KEYRING_SERVICE: &str = "bits";
const KEYRING_USERNAME: &str = "api_key";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Basis-URL der API, z.B. "https://meine.app.tld"
    pub api_url: String,
    /// Der erzeugte API-Key, der später für Snippet‐Uploads verwendet wird
    /// Hinweis: Wird nicht in der Konfigurationsdatei gespeichert, sondern im Keyring
    #[serde(skip)]
    pub api_key: String,
}

impl Config {
    /// Versucht, die Konfigurationsdatei zu laden. Gibt Ok(Some(cfg)) zurück, wenn sie existiert, 
    /// Ok(None), wenn sie nicht existiert, andernfalls Err.
    pub fn load() -> anyhow::Result<Option<Config>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "bits") {
            let config_path: PathBuf = proj_dirs.config_dir().join("config.json");
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let mut cfg: Config = serde_json::from_str(&content)?;

                // Versuche, den API-Key aus dem Keyring zu laden
                match Self::get_api_key_from_keyring() {
                    Ok(api_key) => {
                        cfg.api_key = api_key;
                    },
                    Err(err) => {
                        return Err(anyhow::anyhow!("Fehler beim Laden des API-Keys aus dem Keyring: {}", err));
                    }
                }

                return Ok(Some(cfg));
            }
        }
        Ok(None)
    }

    /// Speichert die Config als pretty‐formatted JSON in "<config_dir>/config.json".
    /// Der API-Key wird separat im Keyring gespeichert.
    pub fn save(&self) -> anyhow::Result<()> {
        // Speichere den API-Key im Keyring
        Self::save_api_key_to_keyring(&self.api_key)?;

        if let Some(proj_dirs) = ProjectDirs::from("", "", "bits") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let config_path = config_dir.join("config.json");
            let mut file = fs::File::create(&config_path)?;

            // Der API-Key wird aufgrund des #[serde(skip)] nicht mit serialisiert
            let json = serde_json::to_string_pretty(self)?;
            file.write_all(json.as_bytes())?;

            // Unter Unix könntest du hier noch chmod 600 setzen:
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&config_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&config_path, perms)?;
            }
            Ok(())
        } else {
            anyhow::bail!("Konnte kein Konfigurationsverzeichnis ermitteln.");
        }
    }

    /// Speichert den API-Key im Keyring
    fn save_api_key_to_keyring(api_key: &str) -> anyhow::Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;
        entry.set_password(api_key)?;
        Ok(())   
    }

    /// Lädt den API-Key aus dem Keyring
    fn get_api_key_from_keyring() -> anyhow::Result<String> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USERNAME)?;
        Ok(entry.get_password()?)
    }
}
