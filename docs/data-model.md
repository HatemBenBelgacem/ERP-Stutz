# Datenmodell

Stand nach allen Migrationen in `migrations/` (bis `20261031000002_kleidergroesse_je_artikel.sql`).
Datenbank: PostgreSQL (über `sqlx`).

Legende: **PK** = Primärschlüssel, **FK** = Fremdschlüssel, **UQ** = eindeutig (UNIQUE).
`NULL` = Spalte darf leer sein; alle anderen Spalten sind `NOT NULL`.

## Übersicht

| Tabelle | Zweck |
|---|---|
| [`kurs`](#kurs) | Stammdaten der angebotenen Weiterbildungen |
| [`termin`](#termin) | Durchführungen eines Kurses |
| [`termin_mitarbeiter`](#termin_mitarbeiter) | Teilnahmen (Termin M:N Mitarbeiter) |
| [`termin_geschaeftsbereich`](#termin_geschaeftsbereich) | Geschäftsbereiche pro Termin (M:N) |
| [`kurs_geschaeftsbereich`](#kurs_geschaeftsbereich) | Geschäftsbereiche pro Kurs (M:N) |
| [`geschaeftsbereich`](#geschaeftsbereich) | Geschäftsbereiche |
| [`mitarbeiter`](#mitarbeiter) | Mitarbeitende |
| [`adresse`](#adresse) | Adressen (Durchführungsorte, Bauherren, Architekten, Mitarbeitende) |
| [`bauobjekt`](#bauobjekt) | Bauobjekte |
| [`projekt`](#projekt) | Projekte zu einem Bauobjekt |
| [`buchung`](#buchung) | Erfasste Stunden auf ein Projekt |
| [`benutzer`](#benutzer) | Benutzerkonten für die Anmeldung |
| [`rolle`](#rolle) | Rollen mit Rechten |
| [`sitzung`](#sitzung) | Angemeldete Sitzungen |
| [`artikel`](#artikel) | Artikel mit Bezeichnung und optionalem Bild |
| [`kleidergroesse`](#kleidergroesse) | Grössen eines Artikels als Text (1:N) |

## Eigene Datentypen

| Typ | Art | Werte |
|---|---|---|
| `termin_status` | ENUM | `neu`, `teamchefs_informiert`, `laufend`, `beendet` |

## Tabellen

### `kurs`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `titel` | `TEXT` | nein | | |
| `beschreibung` | `TEXT` | nein | `''` | |
| `dauer_stunden` | `INTEGER` | nein | `0` | CHECK `dauer_stunden >= 0` |

### `termin`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `kurs_id` | `UUID` | nein | | FK → `kurs.id` |
| `beginn` | `TIMESTAMP` | nein | | |
| `ende` | `TIMESTAMP` | nein | | CHECK `ende > beginn` |
| `max_teilnehmer` | `INTEGER` | nein | `20` | CHECK `max_teilnehmer > 0` |
| `adresse_id` | `UUID` | ja | | FK → `adresse.id` |
| `status` | `termin_status` | nein | `'neu'` | |

### `termin_mitarbeiter`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `termin_id` | `UUID` | nein | | PK, FK → `termin.id` |
| `mitarbeiter_id` | `UUID` | nein | | PK, FK → `mitarbeiter.id` |

### `termin_geschaeftsbereich`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `termin_id` | `UUID` | nein | | PK, FK → `termin.id` |
| `geschaeftsbereich_id` | `UUID` | nein | | PK, FK → `geschaeftsbereich.id` |

### `kurs_geschaeftsbereich`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `kurs_id` | `UUID` | nein | | PK, FK → `kurs.id` |
| `geschaeftsbereich_id` | `UUID` | nein | | PK, FK → `geschaeftsbereich.id` |

### `geschaeftsbereich`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `beschreibung` | `TEXT` | nein | | UQ |

### `mitarbeiter`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `vorname` | `TEXT` | nein | | |
| `nachname` | `TEXT` | nein | | |
| `email` | `TEXT` | nein | | UQ |
| `abteilung` | `TEXT` | nein | `''` | |
| `adresse_id` | `UUID` | ja | | UQ, FK → `adresse.id` |
| `geschaeftsbereich_id` | `UUID` | ja | | FK → `geschaeftsbereich.id` |

### `adresse`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `bezeichnung` | `TEXT` | nein | | |
| `strasse` | `TEXT` | nein | `''` | |
| `plz` | `TEXT` | nein | `''` | |
| `ort` | `TEXT` | nein | `''` | |

### `bauobjekt`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `bezeichnung` | `TEXT` | nein | | |
| `bauherr_id` | `UUID` | ja | | FK → `adresse.id` |
| `architekt_id` | `UUID` | ja | | FK → `adresse.id` |

### `projekt`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `projektnummer` | `TEXT` | nein | | UQ |
| `bezeichnung` | `TEXT` | nein | | |
| `bauobjekt_id` | `UUID` | nein | | FK → `bauobjekt.id` |
| `bauherr_id` | `UUID` | ja | | FK → `adresse.id` |
| `architekt_id` | `UUID` | ja | | FK → `adresse.id` |

### `buchung`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `projekt_id` | `UUID` | nein | | FK → `projekt.id` |
| `zeit` | `DOUBLE PRECISION` | nein | | CHECK `zeit > 0` |
| `erfasst_am` | `TIMESTAMP` | nein | `now()` | |
| `mitarbeiter_id` | `UUID` | ja | | FK → `mitarbeiter.id` |

### `benutzer`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `name` | `TEXT` | nein | | |
| `email` | `TEXT` | nein | | UQ |
| `passwort_hash` | `TEXT` | nein | | |
| `aktiv` | `BOOLEAN` | nein | `TRUE` | |
| `erstellt_am` | `TIMESTAMP` | nein | `now()` | |
| `rolle_id` | `UUID` | ja | | FK → `rolle.id` |
| `microsoft_oid` | `TEXT` | ja | | UQ |

### `rolle`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `name` | `TEXT` | nein | | UQ |
| `beschreibung` | `TEXT` | nein | `''` | |
| `ist_admin` | `BOOLEAN` | nein | `FALSE` | |
| `rechte` | `TEXT[]` | nein | `'{}'` | Format `<bereich>.<aktion>` |
| `erstellt_am` | `TIMESTAMP` | nein | `now()` | |

### `sitzung`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `token_hash` | `TEXT` | nein | | PK (SHA-256-Hash des Cookie-Tokens) |
| `benutzer_id` | `UUID` | nein | | FK → `benutzer.id` |
| `laeuft_ab` | `TIMESTAMP` | nein | | |

### `artikel`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `bezeichnung` | `TEXT` | nein | | CHECK `btrim(bezeichnung) <> ''` |
| `bild` | `TEXT` | ja | | Dateiname des Bildes |
| `bild_daten` | `BYTEA` | ja | | Inhalt des Bildes, höchstens 2 MB (Grenze prüft der Server); CHECK `(bild IS NULL) = (bild_daten IS NULL)` |
| `erstellt_am` | `TIMESTAMP` | nein | `now()` | |
| `geaendert_am` | `TIMESTAMP` | nein | `now()` | |

### `kleidergroesse`

| Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung |
|---|---|---|---|---|
| `id` | `UUID` | nein | `gen_random_uuid()` | PK |
| `artikel_id` | `UUID` | nein | | FK → `artikel.id`, `ON DELETE CASCADE`; UQ zusammen mit `bezeichnung` |
| `bezeichnung` | `TEXT` | nein | | CHECK `btrim(bezeichnung) <> ''`; UQ zusammen mit `artikel_id` |
| `sortierung` | `INTEGER` | nein | | Reihenfolge in der Auswahl |

## Fremdschlüssel

| Tabelle.Spalte | Verweist auf | Beziehung | `ON DELETE` |
|---|---|---|---|
| `termin.kurs_id` | `kurs.id` | Kurs 1:N Termin | `RESTRICT` |
| `termin.adresse_id` | `adresse.id` | Adresse 1:N Termin | `SET NULL` |
| `termin_mitarbeiter.termin_id` | `termin.id` | Termin M:N Mitarbeiter | `CASCADE` |
| `termin_mitarbeiter.mitarbeiter_id` | `mitarbeiter.id` | Termin M:N Mitarbeiter | `CASCADE` |
| `termin_geschaeftsbereich.termin_id` | `termin.id` | Termin M:N Geschäftsbereich | `CASCADE` |
| `termin_geschaeftsbereich.geschaeftsbereich_id` | `geschaeftsbereich.id` | Termin M:N Geschäftsbereich | `CASCADE` |
| `kurs_geschaeftsbereich.kurs_id` | `kurs.id` | Kurs M:N Geschäftsbereich | `CASCADE` |
| `kurs_geschaeftsbereich.geschaeftsbereich_id` | `geschaeftsbereich.id` | Kurs M:N Geschäftsbereich | `CASCADE` |
| `mitarbeiter.adresse_id` | `adresse.id` | Mitarbeiter 1:1 Adresse | `SET NULL` |
| `mitarbeiter.geschaeftsbereich_id` | `geschaeftsbereich.id` | Geschäftsbereich 1:N Mitarbeiter | `SET NULL` |
| `bauobjekt.bauherr_id` | `adresse.id` | Adresse 1:N Bauobjekt | `SET NULL` |
| `bauobjekt.architekt_id` | `adresse.id` | Adresse 1:N Bauobjekt | `SET NULL` |
| `projekt.bauobjekt_id` | `bauobjekt.id` | Bauobjekt 1:N Projekt | `RESTRICT` |
| `projekt.bauherr_id` | `adresse.id` | Adresse 1:N Projekt | `SET NULL` |
| `projekt.architekt_id` | `adresse.id` | Adresse 1:N Projekt | `SET NULL` |
| `buchung.projekt_id` | `projekt.id` | Projekt 1:N Buchung | `RESTRICT` |
| `buchung.mitarbeiter_id` | `mitarbeiter.id` | Mitarbeiter 1:N Buchung | `RESTRICT` |
| `benutzer.rolle_id` | `rolle.id` | Rolle 1:N Benutzer | `RESTRICT` |
| `sitzung.benutzer_id` | `benutzer.id` | Benutzer 1:N Sitzung | `CASCADE` |
| `kleidergroesse.artikel_id` | `artikel.id` | Artikel 1:N Grösse | `CASCADE` |

`RESTRICT`: Der Datensatz lässt sich nicht löschen, solange er verwendet wird.
`SET NULL`: Der Verweis wird geleert. `CASCADE`: Abhängige Zeilen werden mitgelöscht.

## Indizes (ausser Primärschlüssel und UNIQUE)

| Index | Tabelle | Spalte |
|---|---|---|
| `termin_kurs_id_idx` | `termin` | `kurs_id` |
| `termin_adresse_id_idx` | `termin` | `adresse_id` |
| `termin_mitarbeiter_mitarbeiter_id_idx` | `termin_mitarbeiter` | `mitarbeiter_id` |
| `termin_geschaeftsbereich_geschaeftsbereich_id_idx` | `termin_geschaeftsbereich` | `geschaeftsbereich_id` |
| `kurs_geschaeftsbereich_geschaeftsbereich_id_idx` | `kurs_geschaeftsbereich` | `geschaeftsbereich_id` |
| `mitarbeiter_geschaeftsbereich_id_idx` | `mitarbeiter` | `geschaeftsbereich_id` |
| `bauobjekt_bauherr_id_idx` | `bauobjekt` | `bauherr_id` |
| `bauobjekt_architekt_id_idx` | `bauobjekt` | `architekt_id` |
| `projekt_bauobjekt_id_idx` | `projekt` | `bauobjekt_id` |
| `projekt_bauherr_id_idx` | `projekt` | `bauherr_id` |
| `projekt_architekt_id_idx` | `projekt` | `architekt_id` |
| `buchung_projekt_id_idx` | `buchung` | `projekt_id` |
| `buchung_mitarbeiter_id_idx` | `buchung` | `mitarbeiter_id` |
| `benutzer_rolle_id_idx` | `benutzer` | `rolle_id` |
| `sitzung_benutzer_id_idx` | `sitzung` | `benutzer_id` |
| `kleidergroesse_artikel_id_idx` | `kleidergroesse` | `artikel_id` |
