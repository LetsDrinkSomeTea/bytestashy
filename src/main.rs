// src/main.rs
mod api_client;
mod config;
pub mod models;

use crate::models::Snippet;
use api_client::APIClient;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::Number;
use std::path::Path;
use std::{fs, process};

/// BitsCLI: Ein Tool, um Snippets per API hochzuladen.
#[derive(Parser)]
#[command(
    name = "bytestashy",
    version,
    about = "CLI to push snippets to ByteStash"
)]
struct Cli {
    files: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(
        about = "Authenticate with your ByteStash API",
        long_about = "Fetches an API token and stores it in your config file."
    )]
    Login {
        #[arg(help = "URL of your ByteStash server")]
        api_url: String,
    },
    #[command(about = "Show a paginated list of snippets")]
    List {
        #[arg(short, long, help = "Display every snippet, not just the first N")]
        all: bool,
        #[arg(short = 'n', long, help = "Page size N")]
        number: Option<usize>,
        #[arg(short = 'p', long, help = "Page number to display (starting at 1)")]
        page: Option<usize>,
    },
    #[command(about = "Retrieve a snippet by ID and write its files")]
    Get {
        #[arg(help = "Numeric snippet identifier")]
        id: Number,
    },
}

fn get_client() -> Result<APIClient, String> {
    let client = match APIClient::new() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{}", err);
            eprintln!("Not logged in. Please run `bytestashy login <url>` first.");
            process::exit(1);
        }
    };
    Ok(client)
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
        Some(Commands::List { all, number, page }) => match get_client().unwrap().list() {
            Ok(json_value) => {
                let snippets: Vec<Snippet> =
                    serde_json::from_value(json_value).unwrap_or_else(|e| {
                        eprintln!("Failed to parse snippets JSON: {}", e);
                        process::exit(1);
                    });

                let total = snippets.len();
                let page_size = number.unwrap_or(10).min(total);
                let page_index = page.unwrap_or(1).max(1);
                let offset = if *all {
                    0
                } else {
                    (page_index - 1) * page_size
                };
                let count = if *all { total } else { page_size };

                println!("{}", "[ ID] TITLE (DESCRIPTION)".underline().bold());
                snippets
                    .into_iter()
                    .skip(offset)
                    .take(count)
                    .for_each(|snip| {
                        // Truncate description to 60 chars
                        let desc = {
                            let d = &snip.description;
                            if d.chars().count() > 60 {
                                d.chars().take(60).collect::<String>() + "…"
                            } else {
                                d.clone()
                            }
                        };
                        let c_desc = if desc.is_empty() {
                            String::new()
                        } else {
                            format!("({})", desc).white().to_string()
                        };
                        let c_title = snip.title.bold();
                        let c_id = snip.id.to_string().bright_purple();
                        println!(
                            "[{id:>3}] {title} {desc}",
                            title = c_title,
                            desc = c_desc,
                            id = c_id
                        );
                    });

                // Footer
                if *all {
                    println!(
                        "Total of {} snippets",
                        total.to_string().bright_yellow().bold()
                    );
                } else {
                    let num_pages = (total - 1) / page_size + 1;
                    println!(
                        "{}{}/{}{}{}",
                        "page: ".white(),
                        page_index.to_string().bright_yellow().bold(),
                        num_pages.to_string().bright_yellow().bold(),
                        " - total snippets: ".white(),
                        total.to_string().bright_yellow().bold(),
                    );
                }
            }
            Err(err) => {
                eprintln!("Error listing snippets: {}", err);
                process::exit(1);
            }
        },

        Some(Commands::Get { id }) => match get_client().unwrap().get_snippet(id) {
            Ok(json_value) => {
                let snippet: Snippet = match serde_json::from_value(json_value) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to parse snippets JSON: {}", e);
                        process::exit(1);
                    }
                };
                let c_desc = if snippet.description.is_empty() {
                    String::new()
                } else {
                    format!("({})", snippet.description).white().to_string()
                };
                println!(
                    "{} {}\n{}",
                    snippet.title.bright_purple().bold(),
                    c_desc,
                    "Files:".white()
                );
                for fragment in &snippet.fragments {
                    let c_file_name = format!("{}", fragment.file_name);
                    println!("- {}", c_file_name);
                }

                let want_continue: bool = dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "Should the file{} be downloaded?",
                        (&snippet.fragments.len() > &1)
                            .then(|| "s")
                            .unwrap_or_else(|| "")
                    ))
                    .default(true)
                    .interact()
                    .unwrap_or_else(|e| {
                        eprintln!("Error reading answer: {}", e);
                        process::exit(1);
                    });
                if !want_continue {
                    process::exit(0);
                }

                for fragment in snippet.fragments {
                    let path = Path::new(&fragment.file_name);

                    if let Some(parent) = path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            eprintln!("Failed to create directories for {:?}: {}", parent, e);
                            process::exit(1);
                        }
                    }

                    if let Err(e) = fs::write(&path, &fragment.code) {
                        eprintln!("Failed to write {}: {}", fragment.file_name, e);
                        process::exit(1);
                    }
                }
                println!("{}", "Successfully downloaded".bright_purple());
            }
            Err(err) => {
                if err.to_string().contains("404") {
                    eprintln!("{} not found", "Snippet".bright_purple().bold());
                } else {
                    eprintln!("Error getting snippets: {}", err);
                }
                process::exit(1);
            }
        },
        None => {
            let files = &cli.files;
            if files.is_empty() {
                eprintln!("Provide at least one file to upload.");
                process::exit(1);
            }

            let client = get_client().unwrap();

            let title: String = dialoguer::Input::new()
                .with_prompt(format!("{}", "Title".bold()))
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading title: {}", e);
                    process::exit(1);
                });

            let description: String = dialoguer::Input::new()
                .with_prompt(format!("{}", "Description (optional)".bold()))
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading description: {}", e);
                    process::exit(1);
                });

            let is_public: bool = dialoguer::Confirm::new()
                .with_prompt(format!("Should the snippet be {}?", "public".bold()))
                .default(false)
                .interact()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading answer: {}", e);
                    process::exit(1);
                });

            let categories: String = dialoguer::Input::new()
                .with_prompt(format!(
                    "{} (Comma-separated, e.g. \"cli,homelab\")",
                    "Categories".bold()
                ))
                .allow_empty(true)
                .interact_text()
                .unwrap_or_else(|e| {
                    eprintln!("Error reading categories: {}", e);
                    process::exit(1);
                });

            // 3. Snippet erstellen
            match client.create_snippet(&title, &description, is_public, &categories, &files) {
                Ok(json) => {
                    let url = format!("{}/snippets/{}", client.api_url, json.get("id").unwrap());
                    println!("Snippet created at {}", url.bright_purple().underline());
                }
                Err(err) => {
                    eprintln!("Error creating snippet: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
