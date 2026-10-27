---
name: migrate
description: Datenbank-Migrationen für ERP-Stutz anlegen – neue Tabellen, Spalten und Beziehungen (1:1, 1:N, M:N) mit sqlx/PostgreSQL. Verwenden, wenn das Datenmodell geändert wird oder eine Migration fehlschlägt.
---

# Migrate

Migrationen liegen in `migrations/` und werden beim Start des Servers automatisch von sqlx
ausgeführt (`sqlx::migrate!` in `src/backend/db.rs`). Es gibt keine Down-Migrationen.

## Grundregeln

- **Bestehende Migrationen nie ändern.** sqlx speichert eine Prüfsumme jeder ausgeführten
  Datei; eine Änderung lässt den Start auf bestehenden Datenbanken fehlschlagen.
  Korrekturen immer als neue Migration.
- Dateiname: `YYYYMMDD000001_beschreibung.sql`, Datum **nach** der neuesten Datei in
  `migrations/` (auch wenn das in der Zukunft liegt), Beschreibung auf Deutsch in snake_case,
  z. B. `20261030000001_mitarbeiter_sprache.sql`.
- Kommentare in der Migration auf Deutsch: was und warum, inkl. Art der Beziehung.
- Bestehende Daten mitnehmen: neue Pflichtspalten mit `DEFAULT` anlegen oder zuerst befüllen,
  dann `NOT NULL` setzen.

## Konventionen für Tabellen

- Tabellennamen auf Deutsch im Singular: `mitarbeiter`, `buchung`, `geschaeftsbereich`
  (Umlaute ausgeschrieben: ae, oe, ue).
- Primärschlüssel: `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
- Texte: `TEXT NOT NULL DEFAULT ''` für optionale Felder, Pflichtfelder ohne Default.
- Zeitpunkte: `TIMESTAMP` (ohne Zeitzone), `chrono::NaiveDateTime` in Rust.
- Für jeden Fremdschlüssel einen Index: `CREATE INDEX <tabelle>_<spalte>_idx ON <tabelle>(<spalte>);`

## Beziehungen

| Beziehung | Umsetzung |
|---|---|
| N:1 / 1:N | Spalte `<ziel>_id UUID REFERENCES <ziel>(id)` auf der N-Seite |
| 1:1 | wie N:1, zusätzlich `UNIQUE` auf der Spalte |
| M:N | Zwischentabelle `<a>_<b> (a_id, b_id)` mit `PRIMARY KEY (a_id, b_id)` und `ON DELETE CASCADE` auf beiden Seiten |

Verhalten beim Löschen bewusst wählen und im Kommentar begründen:
- `ON DELETE RESTRICT` – Löschen verhindern, solange abhängige Daten existieren (z. B. Buchungen).
- `ON DELETE SET NULL` – Verweis entfernen, Datensatz bleibt (z. B. Adresse eines Mitarbeiters).
- `ON DELETE CASCADE` – nur für Zwischentabellen und reine Unterdaten.

## Nach der Migration im Code nachziehen

1. **Struct** in `src/models.rs` ergänzen (`#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]`).
2. **Server-Funktionen** in `src/backend/api.rs` bzw. `src/backend/projekte.rs`: SELECT,
   INSERT und UPDATE um die neuen Spalten erweitern. Mehrere Schreibvorgänge (z. B. M:N neu
   setzen) in einer Transaktion (`pool().begin()`).
3. **Fehlermeldungen** in `src/backend/db.rs` (`fehler`): für neue Constraints eine
   verständliche deutsche Meldung, abgefragt über den Constraint-Namen
   (PostgreSQL-Standardnamen: `<tabelle>_<spalte>_fkey`, `<tabelle>_<spalte>_key`,
   `<tabelle>_<spalte>_check`). Codes: `23503` Fremdschlüssel, `23505` eindeutig, `23514` Prüfregel.
4. **Rechte**: Neue Bereiche brauchen einen Eintrag im `enum Bereich` (`src/models.rs`).
   Bestehenden Rollen nur dann per Migration Rechte geben, wenn es fachlich klar ist, z. B.
   `UPDATE rolle SET rechte = rechte || '{bereich.sehen}' WHERE NOT ist_admin AND ...;`
   Ladefunktionen, die andere Seiten als Auswahlliste brauchen, in `recht_pflicht` um deren
   Bereiche erweitern.

## Prüfen

Danach den Skill `test` ausführen, insbesondere Schritt 4 (Migrationen gegen eine frische
Datenbank, Constraint-Namen gegen `db.rs` prüfen).

## Migration schlägt beim Start fehl

- `migration ... was previously applied but has been modified` → eine bestehende Datei wurde
  geändert. Änderung rückgängig machen und als neue Migration anlegen.
- Fehler in einer neuen Migration → sqlx führt jede Datei in einer Transaktion aus; die Datei
  korrigieren, solange sie noch auf keiner geteilten Datenbank (Railway) gelaufen ist.
