// src/main.rs
mod api_client;
mod config;

use clap::{Parser, Subcommand};
use std::process;

use api_client::APIClient;

/// BitsCLI: Ein Tool, um Snippets per API hochzuladen.
#[derive(Parser)]
#[command(name = "bytestashy", version, about = "CLI to push snippets to ByteStash")]
struct Cli {
    files: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Login {
        api_url: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Login { api_url }) => {
            if let Err(err) = APIClient::login_and_create_key(api_url) {
                eprintln!("Error while logging in: {}", err);
                process::exit(1);
            }
        }
        None => {
            let files = &cli.files;
            if files.is_empty() {
                eprintln!("Provide at least one file to upload.");
                process::exit(1);
            }

            let client = match APIClient::new() {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("{}", err);
                    eprintln!("Not logged in. Please run `bytestashy login <url>` first.");
                    process::exit(1);
                }
            };

            let title: String = dialoguer::Input::new()
                .with_prompt("Title:")
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading titels: {}", e);
                    process::exit(1);
                });

            let description: String = dialoguer::Input::new()
                .with_prompt("Description (optional)")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading description: {}", e);
                    process::exit(1);
                });

            let is_public: bool = dialoguer::Confirm::new()
                .with_prompt("Should the snippet be public?")
                .default(false)
                .interact()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading answer: {}", e);
                    process::exit(1);
                });

            let categories: String = dialoguer::Input::new()
                .with_prompt("Categories (Comma-separated, e.g. \"cli,homelab\")")
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading categories: {}", e);
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
                    println!("Snippet created at {}/snippets/{}", client.api_url, json.get("id").unwrap());
                }
                Err(err) => {
                    eprintln!("Error creating snippet: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
