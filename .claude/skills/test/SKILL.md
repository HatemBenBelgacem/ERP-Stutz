---
name: test
description: ERP-Stutz prüfen – kompilieren für Server und Browser, Clippy, Unit-Tests und Migrationen gegen eine frische PostgreSQL-Datenbank. Verwenden vor jedem Commit, nach Änderungen am Code oder an Migrationen, oder wenn nach Tests gefragt wird.
---

# Test

Vor jedem Commit alle Schritte ausführen und erst committen, wenn alles sauber ist.

## 1. Kompilieren (Server und Browser)

```bash
cargo check --features ssr --no-default-features
cargo check --lib --features hydrate --no-default-features --target wasm32-unknown-unknown
```

Beide müssen ohne Fehler und ohne Warnungen durchlaufen.
Fehlt das Target: `rustup target add wasm32-unknown-unknown`.

## 2. Clippy

```bash
cargo clippy --features ssr --no-default-features
```

Keine **neuen** Warnungen einführen. Bekannte, bestehende Warnungen (nicht beheben, ausser
die Aufgabe verlangt es):
- `module_inception` bei `src/components/*/mod.rs` (Muster des Projekts: eine Komponente pro Verzeichnis)
- `unused_unit` bei `{()}` als leerem Inhalt von `<Seitenkopf>`
- `type_complexity` bei grossen Tupeln aus `sqlx::query_as` in `src/backend/projekte.rs`

Neue komplexe Tupel lieber mit einem lokalen `type Zeile = (...);` benennen.

## 3. Unit-Tests

```bash
cargo test --features ssr --no-default-features
```

Neue reine Hilfsfunktionen (z. B. Formatierung, Parsen, Rechte in `src/models.rs`) mit
`#[cfg(test)] mod tests` in derselben Datei testen.

## 4. Migrationen gegen eine frische Datenbank

Bei Änderungen in `migrations/` alle Migrationen auf eine leere Datenbank anwenden und die
neuen SQL-Abfragen mit Testdaten ausprobieren:

```bash
createdb erp_stutz_test
for f in migrations/*.sql; do psql -v ON_ERROR_STOP=1 -q -d erp_stutz_test -f "$f" || echo "FEHLER: $f"; done
# Testdaten einfügen und die Abfragen aus src/backend/*.rs ausprobieren, z. B.:
psql -d erp_stutz_test -c "SELECT ... "
dropdb erp_stutz_test
```

Dabei auch prüfen, dass Einschränkungen greifen (CHECK, UNIQUE, Fremdschlüssel) und dass ihre
Namen zu den Meldungen in `src/backend/db.rs` (`fehler`) passen.
Alternativ einfach die App mit `cargo leptos watch` gegen eine leere Datenbank starten:
sqlx führt dann alle Migrationen aus und bricht bei Fehlern ab.

## 5. Im Browser

Geänderte Seiten mit `cargo leptos watch` öffnen und durchklicken: anlegen, bearbeiten,
löschen, Fehlermeldungen, schmale Bildschirmbreite (Smartphone) und ein Benutzer ohne
Admin-Rolle (Rechte!). Wurde das nicht gemacht, im Bericht ausdrücklich erwähnen.
