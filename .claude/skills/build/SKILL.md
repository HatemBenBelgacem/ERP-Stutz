---
name: build
description: ERP-Stutz bauen – lokal mit cargo-leptos (Entwicklung und Release) oder als Docker-Image wie auf Railway. Verwenden, wenn die App gebaut, gestartet oder ein Build-Fehler untersucht werden soll.
---

# Build

ERP-Stutz ist eine Leptos-App (Rust): ein Server-Binary (Feature `ssr`, Axum + PostgreSQL)
und ein WebAssembly-Teil für den Browser (Feature `hydrate`). `cargo-leptos` baut beides
zusammen und kompiliert dabei Tailwind/DaisyUI aus `style/tailwind.css`.

## Voraussetzungen (einmalig)

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --version 0.3.10 --locked   # gleiche Version wie im Dockerfile
npm install                                            # Tailwind und DaisyUI
cp .env.example .env                                   # DATABASE_URL eintragen
```

## Entwicklung

```bash
cargo leptos watch      # baut, startet auf http://127.0.0.1:3000 und lädt bei Änderungen neu
```

Beim Start führt der Server ausstehende Migrationen automatisch aus (`src/backend/db.rs`).
Dafür muss `DATABASE_URL` auf eine laufende PostgreSQL-Datenbank zeigen.

## Release

```bash
cargo leptos build --release
```

Ergebnis: `target/release/erp-stutz`, `target/release/hash.txt` und `target/site/`.

## Docker (wie auf Railway)

```bash
docker build -t erp-stutz .
docker run --rm -p 8080:8080 --env-file .env erp-stutz
```

Das Image lauscht auf Port 8080 (`LEPTOS_SITE_ADDR=0.0.0.0:8080`).

## Schnelle Prüfung ohne vollständigen Build

Wenn nur geprüft werden soll, ob der Code kompiliert (z. B. ohne cargo-leptos):

```bash
cargo check --features ssr --no-default-features
cargo check --lib --features hydrate --no-default-features --target wasm32-unknown-unknown
```

Beide müssen fehlerfrei durchlaufen: Code, der nur im Browser läuft, gehört hinter
`#[cfg(feature = "hydrate")]`, Code mit Datenbankzugriff hinter `#[cfg(feature = "ssr")]`.

## Typische Fehler

- `can't find crate for core` → `rustup target add wasm32-unknown-unknown` fehlt.
- Tailwind-Fehler beim Build → `npm install` ausführen; das Lockfile vom Mac enthält nur die
  macOS-Variante von Tailwind, darum installiert das Dockerfile ohne Lockfile.
- `DATABASE_URL ist nicht gesetzt` → `.env` anlegen (siehe `.env.example`).
