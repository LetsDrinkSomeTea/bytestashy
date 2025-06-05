# BitsCLI

BitsCLI ist ein CLI-Tool, das mit der API von ByteStash kommuniziert, um Dateien schnell als Snippets abzulegen.

## Installation

Voraussetzung ist eine aktuelle Rust-Installation (mindestens Rust 1.74 mit Edition 2024). Anschließend kann das Projekt kompiliert oder direkt installiert werden:

```bash
cargo install --path .
```

## Benutzung

Vor dem ersten Upload muss ein API-Schlüssel für ByteStash erzeugt werden.

```bash
bits login https://beispiel.api.tld
```

Nach erfolgreichem Login können Dateien als Snippet hochgeladen werden. 
Das Programm fragt dabei interaktiv Titel, Beschreibung, Öffentlichkeit und Kategorien ab:

```bash
bits datei1.txt datei2.rs
```

Die Konfiguration wird in einem betriebssystemspezifischen Konfigurationsordner abgelegt (z. B. unter Linux in `$XDG_CONFIG_HOME/bitscli/config.json`). Der API-Key selbst wird sicher im Keyring des Systems gespeichert.

## Lizenz

Dieses Projekt steht unter der MIT-Lizenz.
