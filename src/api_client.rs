// src/api_client.rs
use anyhow::{Context, Result};
use dialoguer::{Input, Password};
use reqwest::blocking::{Client, multipart};
use reqwest::header;
use serde::{Deserialize};
use serde_json::json;
use std::fs::File;
use std::path::Path;

use crate::config::Config;

/// Struktur, um die Antwort von POST /api/auth/login zu parsen:
#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    // user‐Feld ignorieren wir, wir brauchen nur das JWT‐Token
}

/// Struktur, um die Antwort von POST /api/keys zu parsen:
#[derive(Deserialize)]
struct ApiKeyResponse {
    key: String,
}

pub struct APIClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl APIClient {
    /// Baut einen APIClient, indem er Config::load() aufruft. 
    /// Gibt Err zurück, wenn keine Config (und damit kein API-Key) existiert.
    pub fn new() -> Result<APIClient> {
        if let Some(cfg) = Config::load()? {
            let client = Client::builder().build()?;
            Ok(APIClient {
                client,
                api_url: cfg.api_url,
                api_key: cfg.api_key,
            })
        } else {
            anyhow::bail!("Keine gespeicherte Konfiguration gefunden. Bitte führe zuerst `bits login <api-url>` aus.");
        }
    }

    /// Login-Flow:
    /// 1. Abfrage von Username/Passwort
    /// 2. POST /api/auth/login, erhält JWT
    /// 3. Fragt den Namen für den neuen API-Key ab
    /// 4. POST /api/keys mit Header "Authorization: Bearer <JWT>"
    /// 5. Speichert api_url + api_key in config.json
    pub fn login_and_create_key(api_url: &str) -> Result<()> {
        // 1. Username / Passwort abfragen
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()?;
        let password: String = Password::new()
            .with_prompt("Passwort")
            .interact()?;

        let base = api_url.trim_end_matches('/');
        let login_endpoint = format!("{}/api/auth/login", base);
        let http_client = Client::new();

        // 2. POST /api/auth/login
        let resp = http_client
            .post(&login_endpoint)
            .json(&json!({ "username": username, "password": password }))
            .send()
            .context("Fehler bei der Login-Anfrage (POST /api/auth/login)")?;

        if resp.status().as_u16() != 200 {
            if resp.status().as_u16() == 401 {
                anyhow::bail!("Ungültige Zugangsdaten (401 Unauthorized).");
            } else {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                anyhow::bail!("Login-Fehler: HTTP {} – {}", status, text);
            }
        }

        // 2a. JWT aus der Antwort parsen
        let login_data: LoginResponse = resp
            .json()
            .context("Konnte Login-Antwort nicht als JSON parsen")?;
        let jwt_token = login_data.token;

        // 3. Namen für den API-Key abfragen
        let key_name: String = Input::new()
            .with_prompt("Name für den neu zu erzeugenden API-Key")
            .default("bitscli".into())
            .interact_text()?;

        // 4. POST /api/keys
        let create_key_endpoint = format!("{}/api/keys", base);
        let bearer_header_value = format!("bearer {}", jwt_token);
        let resp_key = http_client
            .post(&create_key_endpoint)
            .header("bytestashauth", bearer_header_value)
            .json(&json!({ "name": key_name }))
            .send()
            .context("Fehler bei der Anfrage POST /api/keys")?;

        if resp_key.status().as_u16() != 201 {
            let status = resp_key.status();
            let text = resp_key.text().unwrap_or_default();
            anyhow::bail!("API-Key-Erzeugung fehlgeschlagen: HTTP {} – {}", status, text);
        }

        // 4a. API-Key aus der Antwort parsen
        let key_data: ApiKeyResponse = resp_key
            .json()
            .context("Konnte Antwort von /api/keys nicht als JSON parsen")?;
        let api_key = key_data.key;

        // 5. Config speichern (nur api_url & api_key)
        let cfg = Config {
            api_url: base.to_string(),
            api_key: api_key.clone(),
        };
        cfg.save().context("Fehler beim Speichern der Konfiguration")?;
        println!("Login und API-Key-Erzeugung erfolgreich. API-Key wurde gespeichert.");

        Ok(())
    }

    /// Baut einen HeaderMap nur mit dem API-Key (z.B. "x-api-key: <api_key>").
    /// Je nach API-Definition kann der Headername auch anders sein.
    fn api_key_header(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        // Hier nehmen wir an, die API erwartet den Key unter "x-api-key". 
        // Wenn deine API stattdessen "Authorization: ApiKey <key>" erwartet, 
        // musst du das entsprechend anpassen.
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }

    /// Erstellt ein neues Snippet: baut multipart/form-data mit allen Feldern und Dateien
    /// und sendet POST /api/v1/snippets/push mit Header "x-api-key: <api_key>".
    pub fn create_snippet(
        &self,
        title: &str,
        description: &str,
        is_public: bool,
        categories: &str,
        file_paths: &[String],
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/snippets/push", self.api_url);
        let mut form = multipart::Form::new()
            .text("title", title.to_string())
            .text("description", description.to_string())
            .text("is_public", is_public.to_string())
            .text("categories", categories.to_string());

        // Dateien anhängen
        for path_str in file_paths {
            let path = Path::new(path_str);
            let file_name = path
                .file_name()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("unknown");
            let file = File::open(path)
                .with_context(|| format!("Konnte Datei nicht öffnen: {}", path_str))?;
            form = form.part(
                "files",
                multipart::Part::reader(file).file_name(file_name.to_string()),
            );
        }

        // Request absenden
        let resp = self
            .client
            .post(&url)
            .headers(self.api_key_header())
            .multipart(form)
            .send()
            .context("Fehler beim Senden der Snippet-Anfrage")?;

        match resp.status().as_u16() {
            201 => {
                let json: serde_json::Value =
                    resp.json().context("Konnte Antwort-JSON nicht parsen")?;
                Ok(json)
            }
            400 => {
                let text = resp.text().unwrap_or_default();
                anyhow::bail!(
                    "Error 400: Mindestens ein Fragment ist erforderlich oder ein Pflichtfeld fehlt.\nDetails: {}",
                    text
                );
            }
            401 => {
                anyhow::bail!("Error 401: API-Key ist ungültig oder abgelaufen. Bitte `bits login` erneut ausführen.");
            }
            other => {
                let text = resp.text().unwrap_or_default();
                anyhow::bail!("Fehler {}: {}", other, text);
            }
        }
    }
}
