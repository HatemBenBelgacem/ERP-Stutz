---
name: migration
description: Neue PostgreSQL-Migration für ERP-Stutz anlegen (Tabellen, Spalten, Fremdschlüssel, Rechte für Rollen) und docs/data-model.md nachführen. Verwenden bei jeder Änderung am Datenbankschema.
---

# Migration

Migrationen liegen in `migrations/` und laufen beim Start der App automatisch
(`sqlx::migrate!("./migrations")` in `src/backend/db.rs`).

## Regeln

1. **Bestehende Migrationsdateien nie ändern.** sqlx prüft Prüfsummen; eine geänderte Datei
   verhindert den Start auf jeder bestehenden Datenbank. Immer eine neue Datei anlegen.
2. Dateiname: `migrations/<JJJJMMTT>000001_<kurzname>.sql`, Datum **nach** der neuesten Migration
   (`ls migrations | tail -1`), Kurzname klein mit Unterstrichen.
3. Erste Zeilen: Kommentar auf Deutsch, was und warum, inkl. Kardinalität (`-- Projekt 1:N Buchung`).
4. Konventionen aus den bestehenden Migrationen übernehmen:
   - Tabellennamen Singular, deutsch, klein (`termin`, `bauobjekt`)
   - `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
   - Fremdschlüssel `<ziel>_id UUID … REFERENCES <ziel>(id) ON DELETE …` mit bewusst gewähltem
     `RESTRICT`, `SET NULL` oder `CASCADE`
   - Index auf jeden Fremdschlüssel: `CREATE INDEX <tabelle>_<spalte>_idx ON <tabelle>(<spalte>);`
   - Texte `TEXT NOT NULL DEFAULT ''` statt NULL, wo „leer“ reicht
   - Zeitstempel `TIMESTAMP NOT NULL DEFAULT now()`
   - M:N-Tabellen `<a>_<b>` mit zusammengesetztem Primärschlüssel
5. Bestehende Daten mitdenken: Neue `NOT NULL`-Spalten brauchen einen Standardwert oder ein `UPDATE`.
6. Neuer Bereich mit Rechten? Bestehenden Rollen die Rechte ergänzen, analog
   `20261022000001_adresse_geschaeftsbereich.sql`
   (`UPDATE rolle SET rechte = rechte || '{…}' WHERE NOT ist_admin AND …`).

## Nach dem Anlegen

1. `model.rs` und `repository.rs` des Modells unter `src/backend/<modell>/` anpassen
   (Aufbau siehe `docs/system-design.md`; noch nicht umgestellte Modelle liegen in `src/models.rs` bzw. `src/backend/api.rs`).
2. Skill `build` ausführen.
3. Migration prüfen, wenn eine Datenbank verfügbar ist: App starten oder Datenbank-Test mit
   `#[sqlx::test]` (Skill `test`).
4. `docs/data-model.md` nachführen: Tabellen, Fremdschlüssel, Indizes, Stand-Zeile oben.
