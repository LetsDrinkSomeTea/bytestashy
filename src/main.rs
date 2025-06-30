mod api_client;
mod cli;
mod config;
mod errors;
pub mod models;

use crate::cli::{Cli, Commands, Shell};
use crate::errors::{ByteStashyError, Result};
use crate::models::Snippet;
use api_client::APIClient;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use colored::*;
use std::path::Path;
use std::{fs, process};
use tracing::{error, info, warn};

/// Initialize API client with saved configuration
fn get_client() -> Result<APIClient> {
    APIClient::new().map_err(|e| {
        error!("Failed to initialize API client: {}", e);
        ByteStashyError::Config(e)
    })
}

/// Validate and parse API URL, warn for local networks
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
        // Check for common local/private network addresses
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

/// Check if file exists and is readable
fn validate_file_path(path: &str) -> Result<()> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(ByteStashyError::invalid_input(format!(
            "File does not exist: {path}"
        )));
    }

    if !path_obj.is_file() {
        return Err(ByteStashyError::invalid_input(format!(
            "Path is not a file: {path}"
        )));
    }

    Ok(())
}

/// Validate all provided file paths
fn validate_files(files: &[String]) -> Result<()> {
    if files.is_empty() {
        return Err(ByteStashyError::invalid_input("Provide at least one file"));
    }

    for file in files {
        validate_file_path(file)?;
    }

    Ok(())
}

/// Display formatted list of snippets with truncated descriptions
fn print_snippets_list(snippets: &[Snippet]) {
    println!("{}", "[ ID] TITLE (DESCRIPTION)".underline().bold());
    for snip in snippets {
        // Limit description to 60 chars for display
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
            format!("({desc})").white().to_string()
        };
        let c_title = snip.title.bold();
        let c_id = snip.id.to_string().bright_purple();
        println!("[{c_id:>3}] {c_title} {c_desc}");
    }
}

/// Form data collected from user input
struct SnippetForm {
    title: String,
    description: String,
    is_public: bool,
    categories: String,
}

/// Collect snippet metadata from user via interactive prompts
fn collect_snippet_form_data(defaults: Option<&Snippet>) -> Result<SnippetForm> {
    let title = if let Some(snippet) = defaults {
        dialoguer::Input::new()
            .with_prompt(format!("{}", "Title".bold()))
            .default(snippet.title.clone())
            .interact_text()?
    } else {
        dialoguer::Input::new()
            .with_prompt(format!("{}", "Title".bold()))
            .interact_text()?
    };

    let description = if let Some(snippet) = defaults {
        dialoguer::Input::new()
            .with_prompt(format!("{}", "Description (optional)".bold()))
            .default(snippet.description.clone())
            .allow_empty(true)
            .interact_text()?
    } else {
        dialoguer::Input::new()
            .with_prompt(format!("{}", "Description (optional)".bold()))
            .allow_empty(true)
            .interact_text()?
    };

    let is_public = dialoguer::Confirm::new()
        .with_prompt(format!("Should the snippet be {}?", "public".bold()))
        .default(false)
        .interact()?;

    let categories = if let Some(snippet) = defaults {
        let current_categories = snippet.categories.join(",");
        dialoguer::Input::new()
            .with_prompt(format!(
                "{} (Comma-separated, e.g. \"cli,homelab\")",
                "Categories".bold()
            ))
            .default(current_categories)
            .allow_empty(true)
            .interact_text()?
    } else {
        dialoguer::Input::new()
            .with_prompt(format!(
                "{} (Comma-separated, e.g. \"cli,homelab\")",
                "Categories".bold()
            ))
            .allow_empty(true)
            .interact_text()?
    };

    Ok(SnippetForm {
        title,
        description,
        is_public,
        categories,
    })
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_app(cli) {
        error!("Application error: {}", e);

        // Show user-friendly error messages
        match e {
            ByteStashyError::Auth { message } => {
                eprintln!("Authentication failed: {message}");
                eprintln!("Please run `bytestashy login <url>` to authenticate.");
                process::exit(1);
            }
            ByteStashyError::InvalidInput(msg) => {
                eprintln!("Invalid input: {msg}");
                process::exit(2);
            }
            ByteStashyError::Api { status, message } => {
                eprintln!("API error ({status}): {message}");
                process::exit(3);
            }
            _ => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    }
}

fn run_app(cli: Cli) -> Result<()> {
    // Generate shell completions if requested
    if let Some(shell) = cli.shell {
        let mut cmd = Cli::command();
        match shell {
            Shell::Bash => generate(shells::Bash, &mut cmd, "bytestashy", &mut std::io::stdout()),
            Shell::Zsh => generate(shells::Zsh, &mut cmd, "bytestashy", &mut std::io::stdout()),
            Shell::Fish => generate(shells::Fish, &mut cmd, "bytestashy", &mut std::io::stdout()),
            Shell::Powershell => generate(
                shells::PowerShell,
                &mut cmd,
                "bytestashy",
                &mut std::io::stdout(),
            ),
        }
        return Ok(());
    }

    // Process CLI commands
    match cli.command {
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()
                .map_err(|e| ByteStashyError::Config(anyhow::Error::from(e)))?;
            return Ok(());
        }
        Some(command) => match &command {
            Commands::Login { api_url } => {
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
            Commands::Create { files } => {
                validate_files(files)?;
                info!("Validated {} files for upload", files.len());

                let client = get_client()?;
                let form_data = collect_snippet_form_data(None)?;

                info!("Creating snippet with {} files", files.len());
                match client.create_snippet(
                    &form_data.title,
                    &form_data.description,
                    form_data.is_public,
                    &form_data.categories,
                    files,
                ) {
                    Ok(json) => {
                        let id = json.get("id").ok_or_else(|| {
                            ByteStashyError::invalid_input("Server response missing snippet ID")
                        })?;
                        let url = format!("{}/snippets/{}", client.api_url, id);
                        println!("Snippet created at {}", url.bright_purple().underline());
                        info!("Successfully created snippet with ID: {}", id);
                    }
                    Err(err) => {
                        return Err(ByteStashyError::Config(err));
                    }
                }
            }
            Commands::Get { id } => {
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
                            println!("- {c_file_name}");
                        }

                        // Ask user if they want to preview code
                        let want_show_code: bool = dialoguer::Confirm::new()
                            .with_prompt(format!("{}", "Show code?".bold()))
                            .default(false)
                            .interact()?;

                        if want_show_code {
                            for fragment in &snippet.fragments {
                                let want_show_fragment: bool = dialoguer::Confirm::new()
                                    .with_prompt(format!("Show {}", fragment.file_name.bright_purple().bold()))
                                    .default(true)
                                    .interact()?;
                                if want_show_fragment {
                                    println!("{}\n", fragment.code);
                                }
                            }
                        }
                        
                        // Confirm before downloading files
                        let want_continue: bool = dialoguer::Confirm::new()
                            .with_prompt(format!(
                                "{}",
                                (if snippet.fragments.len() > 1 
                                { "Should the files be downloaded?" } 
                                else { "Should the file be downloaded?" }).bold()
                            ))
                            .default(true)
                            .interact()?;
                        if !want_continue {
                            return Ok(());
                        }

                        for fragment in snippet.fragments {
                            let path = Path::new(&fragment.file_name);

                            // Create parent directories if needed
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
                        return if err.to_string().contains("404") {
                            Err(ByteStashyError::invalid_input("Snippet not found"))
                        } else {
                            Err(ByteStashyError::Config(err))
                        };
                    }
                }
            }
            Commands::Update { id, files } => {
                validate_files(files)?;
                let client = get_client()?;

                let current_snippet: Snippet = match client.get_snippet(id) {
                    Ok(json_value) => serde_json::from_value(json_value)?,
                    Err(err) => {
                        return if err.to_string().contains("404") {
                            Err(ByteStashyError::invalid_input("Snippet not found"))
                        } else {
                            Err(ByteStashyError::Config(err))
                        };
                    }
                };

                let form_data = collect_snippet_form_data(Some(&current_snippet))?;

                info!("Updating snippet {} with {} files", id, files.len());
                match client.update_snippet(
                    id,
                    &form_data.title,
                    &form_data.description,
                    form_data.is_public,
                    &form_data.categories,
                    files,
                ) {
                    Ok(json) => {
                        let updated_id = json.get("id").ok_or_else(|| {
                            ByteStashyError::invalid_input("Server response missing snippet ID")
                        })?;
                        let url = format!("{}/snippets/{}", client.api_url, updated_id);
                        println!("Snippet updated at {}", url.bright_purple().underline());
                        info!("Successfully updated snippet with ID: {}", updated_id);
                    }
                    Err(err) => {
                        return if err.to_string().contains("404") {
                            Err(ByteStashyError::invalid_input("Snippet not found"))
                        } else {
                            Err(ByteStashyError::Config(err))
                        };
                    }
                }
            }
            Commands::Delete { id, force } => {
                let client = get_client()?;

                let snippet_data = client.get_snippet(id).map_err(ByteStashyError::Config)?;
                let snippet: Snippet = serde_json::from_value(snippet_data)?;

                if !force {
                    let confirm = dialoguer::Confirm::new()
                        .with_prompt(format!(
                            "Are you sure you want to delete snippet {} [{id}]?",
                            snippet.title.bright_purple().bold()
                        ))
                        .default(false)
                        .interact()?;

                    if !confirm {
                        println!("{}", "Deletion cancelled".yellow());
                        return Ok(());
                    }
                }

                match client.delete_snippet(id) {
                    Ok(json_value) => {
                        let deleted_id = json_value.get("id").ok_or_else(|| {
                            ByteStashyError::invalid_input("Server response missing snippet ID")
                        })?;
                        println!(
                            "Snippet {} {}",
                            deleted_id,
                            "deleted successfully".green().bold()
                        );
                        info!("Successfully deleted snippet with ID: {}", deleted_id);
                    }
                    Err(err) => {
                        return if err.to_string().contains("404") {
                            Err(ByteStashyError::invalid_input("Snippet not found"))
                        } else {
                            Err(ByteStashyError::Config(err))
                        };
                    }
                }
            }
            Commands::List { all, number, page } => {
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

                let display_snippets: Vec<Snippet> =
                    snippets.into_iter().skip(offset).take(count).collect();

                print_snippets_list(&display_snippets);

                // Show pagination info
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
            Commands::Search {
                query,
                sort,
                search_code,
            } => {
                let client = get_client()?;

                // Check sort parameter is valid
                if let Some(sort_value) = sort {
                    match sort_value.as_str() {
                        "newest" | "oldest" | "alpha-asc" | "alpha-desc" => {}
                        _ => {
                            return Err(ByteStashyError::invalid_input(
                                "Sort must be one of: newest, oldest, alpha-asc, alpha-desc",
                            ));
                        }
                    }
                }

                match client.search_snippets(
                    query.as_ref(),
                    sort.as_deref(),
                    if *search_code { Some(true) } else { None },
                ) {
                    Ok(json_value) => {
                        let snippets: Vec<Snippet> = serde_json::from_value(json_value)?;

                        if snippets.is_empty() {
                            println!(
                                "{}",
                                "No snippets found matching your search criteria".yellow()
                            );
                            return Ok(());
                        }

                        let count = snippets.len();
                        print_snippets_list(&snippets);

                        println!(
                            "Found {} matching snippets",
                            count.to_string().bright_yellow().bold()
                        );
                    }
                    Err(err) => {
                        return Err(ByteStashyError::Config(err));
                    }
                }
            }
        },
    }

    Ok(())
}
