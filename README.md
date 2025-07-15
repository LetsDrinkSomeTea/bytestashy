# bytestashy

`bytestashy` is a CLI application that communicates with the ByteStash API to quickly store files as snippets or retrieve existing ones.

## Installation

A recent Rust installation is required (at least Rust 1.74 with Edition 2024). The tool can be installed directly from crates.io:

```bash
cargo install bytestashy
```

Alternatively, clone this repository and build from source:

```bash
cargo build --release
```

## Commands

### Authentication

Before the first upload, an API key for ByteStash must be generated:

```bash
bytestashy login <API_URL>
```

Configuration is stored in an OS-specific config folder (for example under `$XDG_CONFIG_HOME/bytestashy/config.json` on Linux). The API key itself is saved securely in your system keyring.

### Create Snippets

Upload files as snippets. The program will interactively prompt for title, description, visibility and categories:

```bash
bytestashy create <FILES...>
```

### List Snippets

Display a paginated list of your snippets:

```bash
bytestashy list [OPTIONS]
```

**Options:**

- `--all, -a`: Display all snippets (no pagination)
- `--number, -n <N>`: Page size (default: 10)
- `--page, -p <N>`: Page number to display (starting at 1)

### Get Snippets

Retrieve and download a snippet by ID:

```bash
bytestashy get <ID>
```

The command will show snippet details and prompt whether to download the files.

### Update Snippets

Replace all files in an existing snippet:

```bash
bytestashy update <ID> <FILES...>
```

The program will prompt for updated title, description, visibility and categories, pre-filling with current values.

### Delete Snippets

Delete a snippet by ID:

```bash
bytestashy delete <ID> [OPTIONS]
```

**Options:**

- `--force, -f`: Skip confirmation dialog

### Search Snippets

Search through your snippets with various options:

```bash
bytestashy search <QUERY> [OPTIONS]
```

**Options:**

- `--sort, -s <ORDER>`: Sort order (newest, oldest, alpha-asc, alpha-desc)
- `--search-code`: Search within code content (not just titles/descriptions)

**Available sort options:**

- `newest` - Most recently updated first
- `oldest` - Oldest first
- `alpha-asc` - Alphabetical by title (A-Z)
- `alpha-desc` - Alphabetical by title (Z-A)

### Shell Completions

Generate shell completion scripts for enhanced command-line experience:

```bash
bytestashy --shell <SHELL>
```

**Supported shells:** `bash`, `zsh`, `fish`, `powershell`

**Installation examples:**

**Bash:**

```bash
bytestashy --shell bash > /etc/bash_completion.d/bytestashy
# or for user-only installation:
bytestashy --shell bash > ~/.local/share/bash-completion/completions/bytestashy
```

**Zsh:**

```bash
bytestashy --shell zsh > ~/.zsh/completions/_bytestashy
# Make sure ~/.zsh/completions is in your $fpath
```

**Fish:**

```bash
bytestashy --shell fish > ~/.config/fish/completions/bytestashy.fish
```

After installation, restart your shell or source the completion file to enable tab completion for all bytestashy commands and options.

## License

This project is licensed under the GPLv3. See [LICENSE](LICENSE).

## References

- [ByteStash](https://github.com/jordan-dalby/ByteStash)
