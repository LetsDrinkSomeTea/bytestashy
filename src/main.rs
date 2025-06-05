// src/main.rs
mod api_client;
mod config;

use clap::{Parser, Subcommand};
use std::process;

use api_client::APIClient;

/// BitsCLI: Ein Tool, um Snippets per API hochzuladen.
#[derive(Parser)]
#[command(name = "bits", version, about = "CLI für Snippet‐Uploads auf ByteStash")]
struct Cli {
    files: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Loggt ein, erzeugt einen API-Key und speichert diesen lokal.
    Login {
        /// Basis-URL der API, z. B. https://meine.app.tld
        api_url: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Login { api_url }) => {
            // Verhalten wie bisher für "bits login <api_url>"
            if let Err(err) = APIClient::login_and_create_key(api_url) {
                eprintln!("Fehler beim Login/API-Key-Erzeugung: {}", err);
                process::exit(1);
            }
        }
        None => {
            // Falls kein Subcommand angegeben wurde, behandeln wir die übergebenen Dateien als Upload
            let files = &cli.files;
            if files.is_empty() {
                eprintln!("Bitte mindestens eine Datei angeben.");
                process::exit(1);
            }

            // 1. Versuchen, aus config.json api_url + api_key zu laden
            let client = match APIClient::new() {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("{}", err);
                    eprintln!("Führe zuerst `bits login <api-url>` aus, um den API-Key zu erzeugen.");
                    process::exit(1);
                }
            };

            // 2. Interaktive Abfragen: title, description, is_public, categories, fragments
            let title: String = dialoguer::Input::new()
                .with_prompt("Titel für das Snippet")
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Fehler beim Einlesen des Titels: {}", e);
                    process::exit(1);
                });

            let description: String = dialoguer::Input::new()
                .with_prompt("Beschreibung (optional)")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Fehler beim Einlesen der Beschreibung: {}", e);
                    process::exit(1);
                });

            let is_public: bool = dialoguer::Confirm::new()
                .with_prompt("Soll das Snippet öffentlich sein?")
                .default(false)
                .interact()
                .unwrap_or_else(|e| {
                    eprintln!("Fehler beim Einlesen der Auswahl: {}", e);
                    process::exit(1);
                });

            let categories: String = dialoguer::Input::new()
                .with_prompt("Kategorien (Komma-separiert, z.B. \"rust,cli,security\")")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Fehler beim Einlesen der Kategorien: {}", e);
                    process::exit(1);
                });

            // 3. Snippet erstellen
            match client.create_snippet(
                &title,
                &description,
                is_public,
                &categories,
                &files,
            ) {
                Ok(json) => {
                    println!("Snippet erfolgreich erstellt. Server-Antwort:");
                    println!("{}", serde_json::to_string_pretty(&json).unwrap());
                }
                Err(err) => {
                    eprintln!("Fehler beim Erstellen des Snippets: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
