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

## Usage

### Authenticate with your server

Before the first upload, an API key for ByteStash must be generated:

```bash
bytestashy login https://example.api.tld
```

Configuration is stored in an OS-specific config folder (for example under `$XDG_CONFIG_HOME/bytestashy/config.json` on Linux). The API key itself is saved securely in your system keyring.

### Upload files

After a successful login, files can be uploaded as snippets. The program will interactively prompt for title, description, visibility and categories:

```bash
bytestashy file1.txt file2.rs
```

### List existing snippets

```bash
bytestashy list
```

Pagination options `--number` and `--page` are available. Use `--all` to display all entries.

### Retrieve a snippet

```bash
bytestashy get <ID>
```

## License

This project is licensed under the GPLv3. See [LICENSE](LICENSE).
