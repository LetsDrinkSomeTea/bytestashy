use clap::{Parser, Subcommand, ValueEnum};

/// CLI tool for managing code snippets via ByteStash API
#[derive(Parser)]
#[command(
    name = "bytestashy",
    version,
    about = "CLI to push snippets to ByteStash"
)]
pub struct Cli {
    /// Generate shell completions instead of running commands
    #[arg(long, help = "Generate shell completions for the specified shell")]
    pub shell: Option<Shell>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Authenticate with your ByteStash API",
        long_about = "Fetches an API token and stores it in your config file."
    )]
    Login {
        #[arg(help = "URL of your ByteStash server")]
        api_url: String,
        #[arg(help = "API key to use for authentication (optional)")]
        api_key: Option<String>,
    },
    #[command(about = "Create a new snippet")]
    Create {
        #[arg(help = "Files to upload")]
        files: Vec<String>,
    },
    #[command(about = "Retrieve a snippet by ID and write its files")]
    Get {
        #[arg(help = "Numeric snippet identifier")]
        id: usize,
    },
    #[command(about = "Update an existing snippet")]
    Update {
        #[arg(help = "Numeric snippet identifier")]
        id: usize,
        #[arg(help = "Files to upload (replaces existing files)")]
        files: Vec<String>,
    },
    #[command(about = "Delete a snippet by ID")]
    Delete {
        #[arg(help = "Numeric snippet identifier")]
        id: usize,
        #[arg(short, long, help = "Skip confirmation dialog")]
        force: bool,
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
    #[command(about = "Search snippets")]
    Search {
        #[arg(help = "Search query")]
        query: String,
        #[arg(
            short,
            long,
            help = "Sort order: newest, oldest, alpha-asc, alpha-desc"
        )]
        sort: Option<String>,
        #[arg(long, help = "Search within code fragments")]
        search_code: bool,
    },
}

/// Supported shell types for completion generation
#[derive(ValueEnum, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}
