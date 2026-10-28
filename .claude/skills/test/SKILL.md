---
name: test
description: ERP-Stutz testen – Formatierung (cargo fmt), Lints (cargo clippy) und Tests (cargo test, Unit-Tests und Datenbank-Tests mit #[sqlx::test]) ausführen sowie neue Tests nach Projektkonvention schreiben. Verwenden vor jedem Commit, nach Code-Änderungen oder wenn Tests ergänzt werden sollen.
---

# Test

Reihenfolge: Formatierung → Lints → Tests. Erst wenn alles grün ist, gilt eine Änderung als fertig.

## 1. Formatierung

Der bestehende Code ist noch nicht durchgehend mit `rustfmt` formatiert (siehe T-005 in `task.md`).
Deshalb nur die geänderten Dateien prüfen:

```bash
git diff --name-only HEAD -- '*.rs' | xargs -r rustfmt --edition 2024 --check
```

Bei Abweichungen dieselben Dateien ohne `--check` formatieren. Nicht `cargo fmt` auf das ganze
Projekt anwenden, sonst landen viele fremde Änderungen im Commit.
Sobald T-005 erledigt ist, gilt `cargo fmt --check` für alles.

## 2. Lints

```bash
cargo clippy --features ssr
```

Der bestehende Code hat bereits Warnungen (v. a. `module_inception`, siehe T-006 in `task.md`).
Massstab: **keine neuen Warnungen** in den geänderten Dateien. Bestehende Warnungen in nicht
geänderten Dateien nicht im selben Commit aufräumen, sondern melden.
Sobald T-006 erledigt ist, gilt `cargo clippy --features ssr -- -D warnings`.

## 3. Tests

```bash
cargo test --features ssr
```

Datenbank-Tests (`#[sqlx::test]`) brauchen `DATABASE_URL` auf einen PostgreSQL-Server,
auf dem der Benutzer Datenbanken anlegen darf. sqlx legt pro Test eine eigene Datenbank an,
führt alle Migrationen aus `migrations/` aus und löscht sie danach.
Lokal z. B. mit Docker:

```bash
docker run -d --name erp-test-db -e POSTGRES_PASSWORD=postgres -p 5432:5432 postgres:16
```

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres cargo test --features ssr
```

Ist keine Datenbank verfügbar, nur die Unit-Tests ausführen und das melden:

```bash
cargo test --features ssr --lib -- --skip db_
```

## Tests schreiben

- **Unit-Tests** im selben Modul am Dateiende:

  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn admin_hat_alle_rechte() {
          // Angenommen … Wenn … Dann …
      }
  }
  ```

- **Datenbank-Tests** (Constraints, Migrationen, SQL der Server-Funktionen); Name mit Präfix `db_`,
  damit sie sich ohne Datenbank überspringen lassen:

  ```rust
  #[cfg(all(test, feature = "ssr"))]
  mod tests {
      #[sqlx::test(migrations = "./migrations")]
      async fn db_termin_ende_vor_beginn_wird_abgelehnt(pool: sqlx::PgPool) {
          // INSERT … und prüfen, dass der CHECK greift
      }
  }
  ```

- Testnamen auf Deutsch, beschreiben das erwartete Verhalten.
- Pro Akzeptanzkriterium der Spezifikation (`features/F-XXX-*.md`, Abschnitt 12) mindestens einen Test,
  wo es automatisierbar ist.
- Rechte immer mit drei Fällen testen: Admin, Rolle mit Recht, Rolle ohne Recht.
- Keine Tests deaktivieren, mit `#[ignore]` versehen oder abschwächen, um grün zu werden.

## Meldung

Dem Benutzer melden: welche Schritte liefen, Anzahl bestandener/fehlgeschlagener Tests,
und bei Fehlern die relevante Ausgabe. Übersprungene Schritte (z. B. keine Datenbank) ausdrücklich nennen.
