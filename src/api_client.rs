use anyhow::{Context, Result};
use dialoguer::{Input, Password};
use reqwest::blocking::{Client, Response, multipart};
use reqwest::header;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::path::Path;

use crate::config::Config;

/// Response from login endpoint
#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    // Only need the JWT token
}

/// Response from API key creation endpoint
#[derive(Deserialize)]
struct ApiKeyResponse {
    key: String,
}

/// HTTP client for ByteStash API operations
pub struct APIClient {
    client: Client,
    pub(crate) api_url: String,
    api_key: String,
}

impl APIClient {
    /// Create new API client from saved config
    pub fn new() -> Result<APIClient> {
        if let Some(cfg) = Config::load()? {
            let client = Client::builder().build()?;
            Ok(APIClient {
                client,
                api_url: cfg.api_url,
                api_key: cfg.api_key,
            })
        } else {
            anyhow::bail!("No saved api key found. Run `bytestashy login <api-url>`.");
        }
    }

    /// Interactive login flow - authenticate and create API key
    pub fn login_and_create_key(api_url: &str) -> Result<()> {
        let username: String = Input::new().with_prompt("Username").interact_text()?;
        let password: String = Password::new().with_prompt("Password").interact()?;

        let base = api_url.trim_end_matches('/');
        let login_endpoint = format!("{base}/api/auth/login");
        let http_client = Client::new();

        let resp = http_client
            .post(&login_endpoint)
            .json(&json!({ "username": username, "password": password }))
            .send()
            .context("Error login in (POST /api/auth/login)")?;

        // Handle authentication errors
        if resp.status().as_u16() != 200 {
            if resp.status().as_u16() == 401 {
                anyhow::bail!("Invalid credentials (401 Unauthorized).");
            } else {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                anyhow::bail!("Login error: HTTP {} – {}", status, text);
            }
        }

        let login_data: LoginResponse = resp
            .json()
            .context("Invalid response, unable to parse JSON")?;
        let jwt_token = login_data.token;

        let key_name: String = Input::new()
            .with_prompt("Name of the api key to generate")
            .default("bytestashy".into())
            .interact_text()?;

        let create_key_endpoint = format!("{base}/api/keys");
        let bearer_header_value = format!("bearer {jwt_token}");
        let resp_key = http_client
            .post(&create_key_endpoint)
            .header("bytestashauth", bearer_header_value)
            .json(&json!({ "name": key_name }))
            .send()
            .context("Error creating key (POST /api/keys)")?;

        // Check API key creation was successful
        if resp_key.status().as_u16() != 201 {
            let status = resp_key.status();
            let text = resp_key.text().unwrap_or_default();
            anyhow::bail!("api key generation failed: HTTP {} – {}", status, text);
        }

        let key_data: ApiKeyResponse = resp_key
            .json()
            .context("Invalid response from /api/keys, couldn't parse JSON")?;
        let api_key = key_data.key;

        let cfg = Config {
            api_url: base.to_string(),
            api_key: api_key.clone(),
        };
        cfg.save().context("Error saving config")?;
        println!("Login successful, api key saved to keyring");

        Ok(())
    }

    /// Build HTTP headers with API key authentication
    fn api_key_header(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(&self.api_key).unwrap(),
        );
        headers
    }

    /// Fetch all user snippets
    pub fn list(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/snippets", self.api_url);
        let resp = self
            .client
            .get(&url)
            .headers(self.api_key_header())
            .send()
            .context("Error sending GET request to /api/v1/snippets")?;

        match resp.status().as_u16() {
            200 => {
                let json: serde_json::Value = resp
                    .json()
                    .context("Error parsing JSON response from /api/v1/snippets")?;
                Ok(json)
            }
            401 => {
                anyhow::bail!(
                    "Error 401: api key is invalid. Run 'bytestashy login <url>' to regenerate it."
                );
            }
            other => {
                let text = resp.text().unwrap_or_default();
                anyhow::bail!("Error {}: {}", other, text);
            }
        }
    }

    /// Fetch single snippet by ID
    pub fn get_snippet(&self, id: &usize) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/snippets/{}", self.api_url, id);
        let resp = self
            .client
            .get(&url)
            .headers(self.api_key_header())
            .send()
            .context("Error sending GET request to /api/v1/snippets")?;
        self.check_result(resp)
    }

    /// Upload files and create new snippet
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

        // Add each file to multipart form
        for path_str in file_paths {
            let path = Path::new(path_str);
            let file_name = path
                .file_name()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("unknown");
            let file =
                File::open(path).with_context(|| format!("Couldn't read file: {path_str}"))?;
            form = form.part(
                "files",
                multipart::Part::reader(file).file_name(file_name.to_string()),
            );
        }

        // Send request
        let resp = self
            .client
            .post(&url)
            .headers(self.api_key_header())
            .multipart(form)
            .send()
            .context("Error sending POST request to /api/v1/snippets/push")?;

        self.check_result(resp)
    }

    /// Delete snippet by ID
    pub fn delete_snippet(&self, id: &usize) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/snippets/{}", self.api_url, id);
        let resp = self
            .client
            .delete(&url)
            .headers(self.api_key_header())
            .send()
            .context("Error sending DELETE request to /api/v1/snippets")?;
        self.check_result(resp)
    }

    /// Update existing snippet with new files and metadata
    pub fn update_snippet(
        &self,
        id: &usize,
        title: &str,
        description: &str,
        is_public: bool,
        categories: &str,
        file_paths: &[String],
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/snippets/{}", self.api_url, id);
        let mut form = multipart::Form::new()
            .text("title", title.to_string())
            .text("description", description.to_string())
            .text("is_public", is_public.to_string())
            .text("categories", categories.to_string());

        // Attach files
        // Add each file to multipart form
        for path_str in file_paths {
            let path = Path::new(path_str);
            let file_name = path
                .file_name()
                .and_then(|osstr| osstr.to_str())
                .unwrap_or("unknown");
            let file =
                File::open(path).with_context(|| format!("Couldn't read file: {path_str}"))?;
            form = form.part(
                "files",
                multipart::Part::reader(file).file_name(file_name.to_string()),
            );
        }

        // Send request
        let resp = self
            .client
            .put(&url)
            .headers(self.api_key_header())
            .multipart(form)
            .send()
            .context("Error sending PUT request to /api/v1/snippets")?;

        self.check_result(resp)
    }

    /// Search snippets with query parameters
    pub fn search_snippets(
        &self,
        query: &str,
        sort: Option<&str>,
        search_code: Option<bool>,
    ) -> Result<serde_json::Value> {
        let mut url = format!("{}/api/v1/snippets/search", self.api_url);
        // Build query parameters
        let mut params = Vec::new();

        params.push(format!("q={}", urlencoding::encode(query)));

        if let Some(s) = sort {
            params.push(format!("sort={s}"));
        }
        if let Some(sc) = search_code {
            params.push(format!("searchCode={sc}"));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let resp = self
            .client
            .get(&url)
            .headers(self.api_key_header())
            .send()
            .context("Error sending GET request to /api/v1/snippets/search")?;
        self.check_result(resp)
    }

    /// Parse HTTP response and handle common error codes
    fn check_result(&self, resp: Response) -> Result<serde_json::value::Value> {
        match resp.status().as_u16() {
            200 => {
                let json: serde_json::Value = resp
                    .json()
                    .context("Error parsing JSON response from /api/v1/snippets")?;
                Ok(json)
            }
            201 => {
                let json: serde_json::Value = resp
                    .json()
                    .context("Error parsing JSON response from /api/v1/snippets/push")?;
                Ok(json)
            }
            401 => {
                anyhow::bail!(
                    "Error 401: api key is invalid. Run 'bytestashy login <url>' to regenerate it."
                );
            }
            404 => {
                anyhow::bail!("Error 404: Snippet not found");
            }
            other => {
                let text = resp.text().unwrap_or_default();
                anyhow::bail!("Error {}: {}", other, text);
            }
        }
    }
}
