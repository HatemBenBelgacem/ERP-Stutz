---
name: build
description: ERP-Stutz bauen und auf Compile-Fehler prüfen (Server mit Feature ssr, Browser-WebAssembly mit Feature hydrate, Release-Build mit cargo-leptos oder Docker). Verwenden, wenn Code geändert wurde und geprüft werden soll, ob er kompiliert, oder wenn ein Release- bzw. Docker-Build gefragt ist.
---

# Build

Die App ist ein Leptos-Projekt mit zwei Builds aus derselben Crate:

| Teil | Feature | Ziel | Inhalt |
|---|---|---|---|
| Server | `ssr` | Host (`x86_64`/`aarch64`) | Axum, sqlx/PostgreSQL, Server-Funktionen |
| Browser | `hydrate` | `wasm32-unknown-unknown` | Leptos-Komponenten als WebAssembly |

Ohne Feature kompiliert die Crate nicht sinnvoll. Immer ein Feature angeben.

## 1. Schnell prüfen (Standard nach jeder Änderung)

```bash
cargo check --features ssr
```

Wenn das WASM-Ziel installiert ist, auch den Browser-Teil prüfen. Code hinter
`#[cfg(feature = "ssr")]` fehlt dort, deshalb treten hier andere Fehler auf:

```bash
rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
cargo check --lib --features hydrate --target wasm32-unknown-unknown
```

Die SQL-Abfragen nutzen `sqlx::query` zur Laufzeit (keine `query!`-Makros).
Für den Build ist deshalb **keine** Datenbank und kein `DATABASE_URL` nötig.

## 2. Vollständiger Build mit cargo-leptos

Baut Server, WebAssembly und Tailwind-CSS (inkl. DaisyUI) zusammen nach `target/site`.

Voraussetzungen einmalig:

```bash
cargo install cargo-leptos --version 0.3.10 --locked
```

```bash
npm install
```

Debug-Build bzw. Release-Build:

```bash
cargo leptos build
```

```bash
cargo leptos build --release
```

Lokal starten (braucht PostgreSQL und `.env`, siehe `.env.example`; Migrationen laufen beim Start):

```bash
cargo leptos watch
```

## 3. Docker-Build (wie im Deployment)

```bash
docker build -t erp-stutz .
```

Nur ausführen, wenn Docker verfügbar ist und der Release-Build geprüft werden soll; er dauert lange.

## Vorgehen und Meldung

1. Mit Schritt 1 beginnen. Bei Fehlern die **erste** Fehlermeldung beheben, dann erneut prüfen.
2. Warnungen nicht ignorieren: neue Warnungen aus der eigenen Änderung beheben.
3. Schritt 2 oder 3 nur, wenn danach gefragt wird oder Änderungen an `Cargo.toml`,
   `style/`, `package.json`, `Dockerfile` oder Assets betroffen sind.
4. Dem Benutzer melden: welche Befehle liefen, ob sie erfolgreich waren, und was übersprungen wurde
   (z. B. „WASM-Prüfung übersprungen, Ziel nicht installiert“).
