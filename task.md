# Aufgaben

Konkrete, ausführbare Aufgaben. Jede Aufgabe gehört zu einem Feature aus
[features/index.md](features/index.md) oder ist als „Technik“ markiert.

**Regeln**

- Eine Aufgabe ist klein genug für einen Commit bzw. einen Pull Request.
- „Fertig“ heisst: Skill `build` und Skill `test` laufen ohne Fehler, Akzeptanzkriterien geprüft,
  bei Schemaänderung `docs/data-model.md` nachgeführt.
- Erledigte Aufgaben abhaken und mit Datum nach „Erledigt“ verschieben.

## In Arbeit

| ID | Aufgabe | Feature | Verantwortlich | Notiz |
|---|---|---|---|---|
| | | | | |

## Offen

| ID | Aufgabe | Feature | Priorität | Abhängig von |
|---|---|---|---|---|
| T-001 | Erste Unit-Tests für `Bereich`/`Aktion` und Rechteprüfung in `src/models.rs` | Technik | Muss | |
| T-002 | Datenbank-Tests mit `#[sqlx::test]` für Migrationen und Constraints | Technik | Soll | T-001 |
| T-003 | CI (GitHub Actions): `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` | Technik | Soll | T-001, T-005, T-006 |
| T-004 | Spezifikation für das nächste Feature nach `features/_vorlage.md` schreiben | F-010 | Muss | |
| T-005 | Ganzes Projekt einmal mit `cargo fmt` formatieren (175 Abweichungen), eigener Commit | Technik | Soll | |
| T-006 | Clippy-Warnungen beheben (26, davon 17 `module_inception` durch `components/<x>/<x>.rs`) | Technik | Soll | |
| T-007 | <Aufgabe> | <F-…> | | |

## Erledigt

| ID | Aufgabe | Feature | Erledigt am |
|---|---|---|---|
| T-000 | Datenmodell in `docs/data-model.md` dokumentiert | Technik | 2026-10-28 |
