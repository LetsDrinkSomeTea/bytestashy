# bytestashy

bytestashy is a CLI tool that communicates with the ByteStash API to quickly store files as snippets.

## Installation

A recent Rust installation is required (at least Rust 1.74 with Edition 2024). Then the project can be compiled or installed directly:

```bash
cargo install bytestashy
```

## Usage

Before the first upload, an API key for ByteStash must be generated:

```bash
bytestashy login https://example.api.tld
```

After a successful login, files can be uploaded as snippets. The program will interactively prompt for title, description, visibility, and categories:

```bash
bytestashy file1.txt file2.rs
```

Configuration is stored in an OS-specific config folder (e.g., on Linux under `$XDG_CONFIG_HOME/bytestashy/config.json`).
The API key itself is securely saved in the system’s keyring.

## License

This project is licensed under the GPLv3. [LICENSE](LICENSE)
