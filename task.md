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
| T-005 | Ganzes Projekt einmal mit `cargo fmt` formatieren (175 Abweichungen), eigener Commit | Technik | Soll | |
| T-006 | Clippy-Warnungen beheben (26, davon 17 `module_inception` durch `components/<x>/<x>.rs`) | Technik | Soll | |
| T-007 | Umstellung Schritt 1: `backend/db.rs`, `backend/gemeinsam.rs`, `Bereich`/`Aktion` nach `backend/rolle/model.rs` (siehe `docs/system-design.md`, Abschnitt 7) | Technik | Soll | |
| T-008 | Umstellung Schritt 2: Modell `geschaeftsbereich` in neue Struktur | Technik | Soll | T-007 |
| T-009 | Umstellung Schritte 3–8: `adresse`, `kurs`, `mitarbeiter`, `bauobjekt`/`projekt`/`buchung`, `termin`, `rolle`/`benutzer`/`auth` (je ein PR) | Technik | Soll | T-008 |
| T-010 | Umstellung Schritt 9: `backend/api.rs`, `backend/projekte.rs`, `models.rs` entfernen | Technik | Soll | T-009 |
| T-011 | Gemeinsame Navigationsliste nach `src/navigation.rs` verschieben (`Ziel`, `NAVIGATION`), Unit-Test für die Sichtbarkeit | F-010 | Muss | |
| T-012 | Komponente `src/components/startansicht/` mit Kachelraster, Leer-Zustand und Ladeanzeige | F-010 | Muss | T-011 |
| T-013 | Route `/` auf Startansicht umstellen, `Startseite` entfernen, `/personal` und Bereichsschutz leiten auf `/` um | F-010 | Muss | T-012 |
| T-014 | Skill `build` und `test` ohne Fehler, AK-1 bis AK-7 im Browser prüfen | F-010 | Muss | T-013 |

## Erledigt

| ID | Aufgabe | Feature | Erledigt am |
|---|---|---|---|
| T-000 | Datenmodell in `docs/data-model.md` dokumentiert | Technik | 2026-10-28 |
| T-004 | Spezifikation für F-010 nach `features/_vorlage.md` schreiben | F-010 | 2026-10-30 |
