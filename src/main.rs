// src/main.rs
mod api_client;
mod config;
mod errors;
pub mod models;

use crate::errors::{ByteStashyError, Result};
use crate::models::Snippet;
use api_client::APIClient;
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Number;
use std::path::Path;
use std::{fs, process};
use tracing::{error, info, warn};

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

fn get_client() -> Result<APIClient> {
    APIClient::new().map_err(|e| {
        error!("Failed to initialize API client: {}", e);
        ByteStashyError::Config(e)
    })
}

fn validate_api_url(url: &str) -> Result<url::Url> {
    let parsed_url = url::Url::parse(url)?;

    if !matches!(parsed_url.scheme(), "http" | "https") {
        return Err(ByteStashyError::invalid_input(
            "URL must use http or https scheme",
        ));
    }

    if parsed_url.host().is_none() {
        return Err(ByteStashyError::invalid_input("URL must have a valid host"));
    }

    if let Some(host) = parsed_url.host_str() {
        if host == "localhost"
            || host == "127.0.0.1"
            || host.starts_with("192.168.")
            || host.starts_with("10.")
        {
            warn!("Using local/private network URL: {}", host);
        }
    }

    Ok(parsed_url)
}

fn validate_file_path(path: &str) -> Result<()> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(ByteStashyError::invalid_input(format!(
            "File does not exist: {}",
            path
        )));
    }

    if !path_obj.is_file() {
        return Err(ByteStashyError::invalid_input(format!(
            "Path is not a file: {}",
            path
        )));
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_app(cli) {
        error!("Application error: {}", e);

        // Print user-friendly error message
        match e {
            ByteStashyError::Auth { message } => {
                eprintln!("Authentication failed: {}", message);
                eprintln!("Please run `bytestashy login <url>` to authenticate.");
                process::exit(1);
            }
            ByteStashyError::InvalidInput(msg) => {
                eprintln!("Invalid input: {}", msg);
                process::exit(2);
            }
            ByteStashyError::Api { status, message } => {
                eprintln!("API error ({}): {}", status, message);
                process::exit(3);
            }
            _ => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn run_app(cli: Cli) -> Result<()> {
    match &cli.command {
        Some(Commands::Login { api_url }) => {
            validate_api_url(api_url)?;

            let result = APIClient::login_and_create_key(api_url);

            match result {
                Ok(_) => {
                    println!("{}", "Login successful!".green().bold());
                }
                Err(e) => {
                    return Err(ByteStashyError::Auth {
                        message: e.to_string(),
                    });
                }
            }
        }
        Some(Commands::List { all, number, page }) => {
            let client = get_client()?;

            let json_value = client.list().map_err(ByteStashyError::Config)?;

            let snippets: Vec<Snippet> = serde_json::from_value(json_value)?;

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

        Some(Commands::Get { id }) => {
            let client = get_client()?;

            match client.get_snippet(id) {
                Ok(json_value) => {
                    let snippet: Snippet = serde_json::from_value(json_value)?;
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
                        let c_file_name = fragment.file_name.to_string();
                        println!("- {}", c_file_name);
                    }

                    let want_continue: bool = dialoguer::Confirm::new()
                        .with_prompt(format!(
                            "Should the file{} be downloaded?",
                            if snippet.fragments.len() > 1 { "s" } else { "" }
                        ))
                        .default(true)
                        .interact()?;
                    if !want_continue {
                        return Ok(());
                    }

                    for fragment in snippet.fragments {
                        let path = Path::new(&fragment.file_name);

                        if let Some(parent) = path.parent() {
                            fs::create_dir_all(parent).map_err(|e| {
                                ByteStashyError::file_operation(parent.display().to_string(), e)
                            })?;
                        }

                        fs::write(path, &fragment.code).map_err(|e| {
                            ByteStashyError::file_operation(fragment.file_name.clone(), e)
                        })?;
                    }
                    println!("{}", "Successfully downloaded".bright_purple());
                }
                Err(err) => {
                    if err.to_string().contains("404") {
                        return Err(ByteStashyError::invalid_input("Snippet not found"));
                    } else {
                        return Err(ByteStashyError::Config(err));
                    }
                }
            }
        }
        None => {
            let files = &cli.files;
            if files.is_empty() {
                return Err(ByteStashyError::invalid_input(
                    "Provide at least one file to upload",
                ));
            }

            // Validate all files before proceeding
            for file in files {
                validate_file_path(file)?;
            }
            info!("Validated {} files for upload", files.len());

            let client = get_client()?;

            let title: String = dialoguer::Input::new()
                .with_prompt(format!("{}", "Title".bold()))
                .interact_text()?;

            let description: String = dialoguer::Input::new()
                .with_prompt(format!("{}", "Description (optional)".bold()))
                .allow_empty(true)
                .interact_text()?;

            let is_public: bool = dialoguer::Confirm::new()
                .with_prompt(format!("Should the snippet be {}?", "public".bold()))
                .default(false)
                .interact()?;

            let categories: String = dialoguer::Input::new()
                .with_prompt(format!(
                    "{} (Comma-separated, e.g. \"cli,homelab\")",
                    "Categories".bold()
                ))
                .allow_empty(true)
                .interact_text()?;

            // Create upload progress bar
            let pb = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}")?);
            pb.set_message(format!("Uploading {} files...", files.len()));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            info!("Creating snippet with {} files", files.len());
            match client.create_snippet(&title, &description, is_public, &categories, files) {
                Ok(json) => {
                    pb.finish_with_message("Upload completed");
                    let id = json.get("id").ok_or_else(|| {
                        ByteStashyError::invalid_input("Server response missing snippet ID")
                    })?;
                    let url = format!("{}/snippets/{}", client.api_url, id);
                    println!("Snippet created at {}", url.bright_purple().underline());
                    info!("Successfully created snippet with ID: {}", id);
                }
                Err(err) => {
                    pb.finish_with_message("Upload failed");
                    return Err(ByteStashyError::Config(err));
                }
            }
        }
    }

    Ok(())
}
