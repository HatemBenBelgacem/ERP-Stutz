# F-011: Neues Modul Artikel

| Feld | Wert |
|---|---|
| ID | F-011 |
| Status | Bereit |
| Priorität | Soll |
| Bereich (ERP) | Lager (Menüpunkt „Artikel“) |
| Verantwortlich | offen (bitte ergänzen) |
| Erstellt / geändert | 2026-10-30 / 2026-10-30 |
| Abhängig von | F-002 (Rechte), F-010 (Navigationsliste und Startansicht), F-015 (Grösse als Kleidergrösse) |
| Roadmap | [features/index.md](index.md) |

---

## 1. Ziel und Nutzen

- **Problem:** Es gibt keine Tabelle und keine Seite für Artikel. Artikel lassen sich im System nicht erfassen.
- **Ziel:** Unter Lager gibt es einen Menüpunkt „Artikel“. Die Seite zeigt alle Artikel mit Bezeichnung und Kleidergrösse (F-015). Artikel lassen sich anlegen, bearbeiten und löschen, auch mit einem Bild.
- **Nutzen:** Artikel sind erfasst, und jeder mit Bild erkennbar. Messbar: Alle Akzeptanzkriterien in Abschnitt 12 sind erfüllt.

## 2. Abgrenzung

| Im Umfang | Nicht im Umfang |
|---|---|
| Tabelle `artikel` (Migration), Grösse als Verweis auf `kleidergroesse` (F-015); Seite `/lager/artikel` mit Liste, Dialog zum Anlegen und Bearbeiten, Löschen nach Bestätigung; Bild hochladen (JPG, PNG, höchstens 2 MB) und als Vorschau zeigen; Menüpunkt „Artikel“ und Kachel in der Startansicht; Rechte `lager.sehen`, `lager.anlegen`, `lager.bearbeiten`, `lager.loeschen`; Datenbank-Tests; `docs/data-model.md` | Import bestehender Artikel (der Bestand startet leer); Bestände, Lagerorte und Materialbewegungen (das Modul Lager bleibt sonst Platzhalter); Suche, Filter und Sortierung über die Standardsortierung hinaus; Bild in der Liste; mehrere Bilder je Artikel; Bildbearbeitung (Zuschnitt, Skalierung) |

## 3. Begriffe

| Begriff | Bedeutung im Projekt |
|---|---|
| Artikel | Ein Datensatz der Tabelle `artikel`. |
| Bezeichnung | Name des Artikels. Spalte `bezeichnung`, Datentyp `TEXT`. Pflicht, Duplikate sind erlaubt. |
| Bild | Hochgeladene Bilddatei, JPG oder PNG, höchstens 2 MB. Optional. Spalte `bild` (`TEXT`) hält den Dateinamen, Spalte `bild_daten` (`BYTEA`) den Inhalt. |
| Grösse | Verweis auf eine Kleidergrösse (F-015), Spalte `kleidergroesse_id`. Pflicht. |

## 4. Rollen und Berechtigungen

Rechte haben das Format `<bereich>.<aktion>` (siehe `Bereich` und `Aktion` in `src/models.rs`).
Admin-Rollen (`rolle.ist_admin`) haben immer alle Rechte.

Artikel gehört zum bestehenden Bereich `Bereich::Lager` (Schlüssel `lager`). Der Bereich hatte bisher nur das Recht „Sehen“. Die Aktionen von `Bereich::Lager` werden in `Bereich::aktionen` (`src/models.rs`) auf die Standardaktionen `Aktion::STANDARD` erweitert.

| Recht | Wer braucht es | Was ist damit erlaubt |
|---|---|---|
| `lager.sehen` | Rollen, die Artikel ansehen | Seite `/lager/artikel` öffnen, Liste und Bilder sehen, Menüpunkt und Kachel sehen |
| `lager.anlegen` | Rollen, die Artikel erfassen | Neuen Artikel mit Bild speichern |
| `lager.bearbeiten` | Rollen, die Artikel ändern | Bezeichnung, Grösse und Bild ändern |
| `lager.loeschen` | Rollen, die Artikel entfernen | Artikel löschen |

- Neuer `Bereich` nötig? nein. Migration für bestehende Rollen? nein. Bestehende Rollen behalten ihr Recht `lager.sehen`. Die neuen Rechte vergibt die Administration in der Rechteverwaltung, weil sie in `rolle.rechte` als Text stehen und keine Spalte dafür nötig ist.
- Was sieht ein Benutzer **ohne** `lager.sehen`? Menüpunkt und Kachel fehlen. `/lager/artikel` leitet über den Bereichsschutz auf `/` um.
- Was sieht ein Benutzer ohne `lager.anlegen`, `lager.bearbeiten` oder `lager.loeschen`? Der Knopf „Neu“ bzw. die jeweilige Zeilenaktion fehlt.
- Serverseitige Prüfung in jeder Server-Funktion und in der Bild-Route (Abschnitt 9).

## 5. User Stories

| ID | Als … | möchte ich … | damit … | Priorität |
|---|---|---|---|---|
| US-1 | Projektteam | eine Tabelle `artikel` mit Bezeichnung, Bild und Kleidergrösse | Artikel gespeichert werden können | Muss |
| US-2 | Benutzer mit `lager.sehen` | alle Artikel auf einer Seite sehen und den Menüpunkt „Artikel“ öffnen | ich Artikel finde | Muss |
| US-3 | Benutzer mit `lager.anlegen`, `lager.bearbeiten` oder `lager.loeschen` | Artikel erfassen, ändern und löschen | die Artikelliste aktuell bleibt | Muss |
| US-4 | Benutzer mit `lager.anlegen` oder `lager.bearbeiten` | ein Bild zum Artikel hochladen und als Vorschau sehen | ich den Artikel erkenne | Muss |

## 6. Fachliche Anforderungen

### 6.1 Funktionale Anforderungen

| ID | Anforderung | Story |
|---|---|---|
| FA-1 | Das System legt die Tabelle `artikel` mit `id`, `bezeichnung`, `bild`, `bild_daten`, `kleidergroesse_id`, `erstellt_am` und `geaendert_am` an. | US-1 |
| FA-2 | Die Spalte `kleidergroesse_id` verweist auf eine vorhandene Kleidergrösse (F-015). | US-1 |
| FA-3 | `/lager/artikel` zeigt alle Artikel als Tabelle mit Bezeichnung und Grösse, sortiert nach Bezeichnung aufsteigend. | US-2 |
| FA-4 | Ein Dialog legt einen Artikel an oder ändert ihn. Löschen erfolgt nach einer Bestätigung. | US-3 |
| FA-5 | Ein Bild (JPG oder PNG, höchstens 2 MB) wird beim Anlegen oder Bearbeiten gespeichert. Im Dialog wird das gespeicherte Bild als Vorschau gezeigt. | US-4 |
| FA-6 | Bearbeiten ohne neue Bilddatei behält das bestehende Bild. „Bild entfernen“ löscht es. | US-4 |
| FA-7 | Der Menüpunkt „Artikel“ steht unter Lager und führt zu `/lager/artikel`. Die Startansicht (F-010) zeigt dafür eine Kachel. | US-2 |

### 6.2 Geschäftsregeln und Validierung

| ID | Regel | Wo geprüft (DB / Server / UI) | Meldung an den Benutzer |
|---|---|---|---|
| GR-1 | `kleidergroesse_id` ist Pflicht und verweist auf eine vorhandene Kleidergrösse (F-015). | DB (NOT NULL, Fremdschlüssel), UI (Auswahl) | „Bitte eine Grösse wählen.“ |
| GR-2 | `bezeichnung` ist Pflicht. Leerzeichen am Rand zählen nicht. | DB (CHECK `btrim(bezeichnung) <> ''`), Server, UI | „Bitte eine Bezeichnung eingeben.“ |
| GR-3 | `bezeichnung` darf doppelt vorkommen. | – | – |
| GR-4 | Ein Bild ist optional. Erlaubt sind die Endungen `.jpg`, `.jpeg` und `.png`, und der Dateiinhalt muss zum Format passen. | Server (Dateikopf), UI (Auswahl) | „Das Bild muss ein JPG oder PNG sein.“ |
| GR-5 | Ein Bild ist höchstens 2 MB gross (2 097 152 Byte). | Server, UI | „Das Bild ist grösser als 2 MB.“ |
| GR-6 | `bild` und `bild_daten` sind entweder beide leer oder beide gefüllt. | DB (CHECK) | – (nur bei einem Fehler im Code) |

### 6.3 Status und Abläufe

Entfällt, weil das Objekt keinen Lebenszyklus hat.

## 7. Datenmodell

Bezug: [docs/data-model.md](../docs/data-model.md). Nach der Umsetzung dort nachführen.

### 7.1 Neue oder geänderte Tabellen

Die Grösse verweist auf die Tabelle `kleidergroesse` (F-015). Der Datentyp `artikel_groesse` entfällt.

| Tabelle | Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung | Bemerkung |
|---|---|---|---|---|---|---|
| `artikel` | `id` | `UUID` | nein | `gen_random_uuid()` | PK | |
| `artikel` | `bezeichnung` | `TEXT` | nein | | CHECK `btrim(bezeichnung) <> ''` | Pflicht, Duplikate erlaubt |
| `artikel` | `bild` | `TEXT` | ja | | | Dateiname, z. B. `schraube.jpg` |
| `artikel` | `bild_daten` | `BYTEA` | ja | | CHECK `(bild IS NULL) = (bild_daten IS NULL)` | Inhalt der Bilddatei, höchstens 2 MB |
| `artikel` | `kleidergroesse_id` | `UUID` | nein | | FK → `kleidergroesse.id`, `ON DELETE RESTRICT` | Pflicht, ersetzt `groesse` (F-015) |
| `artikel` | `erstellt_am` | `TIMESTAMP` | nein | `now()` | | |
| `artikel` | `geaendert_am` | `TIMESTAMP` | nein | `now()` | | Bei jeder Änderung neu setzen |

### 7.2 Beziehungen

`artikel.kleidergroesse_id` → `kleidergroesse.id`, N:1, `ON DELETE RESTRICT` (siehe F-015).

### 7.3 Migration

- Datei: `migrations/20261030000001_artikel.sql` (nach der letzten Migration `20261029000001_mitarbeiter_beziehungen.sql`)
- Umgang mit **bestehenden Daten**: Entfällt, weil die Tabelle neu und leer ist.
- Die Änderung an `artikel` durch F-015 (Grösse als Relation) folgt in einer eigenen Migration.
- Indizes: Index auf `artikel.kleidergroesse_id` (siehe F-015). Die Bezeichnung ist nicht eindeutig.
- Rechte für bestehende Rollen: keine Migration nötig, siehe Abschnitt 4.

## 8. Benutzeroberfläche

### 8.1 Navigation und Route

| Route | Komponente | Menü | Schutz |
|---|---|---|---|
| `/lager/artikel` | `src/components/artikel/artikel.rs` (`ArtikelSeite`) | Eintrag „Artikel“ in der Gruppe „Lager“ in `NAVIGATION` (`src/navigation.rs`), wie Personal › Weiterbildung; Icon aus `icondata` (`LuPackage`) | `Bereichsschutz bereich=Bereich::Lager` |

In `src/app.rs` wird die Route `lager` zur verschachtelten Route: Bei `""` leitet sie auf `/lager/artikel` weiter (wie `/personal` auf `/`), bei `artikel` steht die Artikelseite. Einen Menüpunkt „Lager“ gibt es nicht mehr; die Gruppe „Lager“ enthält nur „Artikel“.

### 8.2 Ansichten

Die Artikelseite folgt dem Muster von `AdresseSeite` (`src/components/adresse/adresse.rs`): Liste auf einer Seite, Anlegen und Bearbeiten im Dialog, Zeilenaktionen.

**Artikelliste (`/lager/artikel`)**

- **Zweck und Inhalt:** Seitenkopf „Artikel“. Tabelle mit den Spalten Bezeichnung und Grösse, sortiert nach Bezeichnung aufsteigend. Keine Suche, kein Filter.
- **Aktionen:** Knopf „Neu“ (mit `lager.anlegen`). Pro Zeile „Bearbeiten“ (mit `lager.bearbeiten`) und „Löschen“ (mit `lager.loeschen`). Löschen fragt vorher nach.
- **Zustände:** Laden mit `Laden`. Leer: `Leer` mit dem Text „Noch keine Artikel erfasst.“. Fehler: `Fehlermeldung`. Ohne Recht: Umleitung auf `/`.
- **Skizze / Screenshot:** Entfällt, weil die Tabelle hier vollständig beschrieben ist.

**Dialog „Neuer Artikel“ / „Artikel bearbeiten“**

- Felder wie in Abschnitt 8.3, Knöpfe „Speichern“ und „Abbrechen“ (`DialogAktionen`).
- Bild: Auswahl einer Datei, Vorschau des gespeicherten Bildes, Knopf „Bild entfernen“.

**Bestätigung „Löschen“**

- Frage: „Artikel „{Bezeichnung}“ wirklich löschen?“ mit Knöpfen „Löschen“ und „Abbrechen“.

### 8.3 Formularfelder

| Feld | Typ / Eingabe | Pflicht | Standard | Validierung | Hilfetext |
|---|---|---|---|---|---|
| Bezeichnung | Text | ja | leer | GR-2 | – |
| Grösse | Auswahl aus den Kleidergrössen (F-015) | ja | keine Vorauswahl | GR-1 | – |
| Bild | Datei (JPG, PNG) | nein | kein Bild | GR-4, GR-5 | „JPG oder PNG, höchstens 2 MB“ |

### 8.4 Gestaltung und Bedienbarkeit

- DaisyUI-Komponenten wie bei den Adressen (Tabelle, Dialog, `Leer`, `Laden`, `Fehlermeldung` aus `src/components/gemeinsam`) wiederverwenden.
- Kompakte Tabellen-Einstellung berücksichtigen.
- Mobil: Die Tabelle bleibt mit zwei Spalten lesbar. Dialog bildschirmfüllend.
- Tastaturbedienung der Dialoge und Zeilenaktionen.
- Texte auf Deutsch.

## 9. Schnittstellen

### 9.1 Server-Funktionen

Validierung und Umwandlung liegen in `model.rs` (Muster laut `docs/system-design.md`, Abschnitt 8).

| Funktion | Datei | Parameter | Rückgabe | Recht | Fehlerfälle |
|---|---|---|---|---|---|
| `artikel_laden` | `src/backend/artikel/server_fns.rs` | – | `Vec<Artikel>` ohne `bild_daten` | `lager.sehen` | nicht angemeldet, kein Recht |
| `artikel_anlegen` | `src/backend/artikel/server_fns.rs` | `eingabe: ArtikelEingabe` (Bezeichnung, Kleidergrösse, Bild optional) | `Uuid` | `lager.anlegen` | GR-1 bis GR-5 |
| `artikel_bearbeiten` | `src/backend/artikel/server_fns.rs` | `id: Uuid, eingabe: ArtikelEingabe` (Bild: neu, behalten oder entfernen) | `()` | `lager.bearbeiten` | GR-1 bis GR-5, Datensatz fehlt |
| `artikel_loeschen` | `src/backend/artikel/server_fns.rs` | `id: Uuid` | `()` | `lager.loeschen` | Datensatz fehlt |
| `kleidergroessen_laden` | `src/backend/artikel/server_fns.rs` | – | `Vec<Kleidergroesse>` | `lager.sehen` | nicht angemeldet, kein Recht |

### 9.2 Weitere Endpunkte und Exporte

- `GET /artikel/{id}/bild` in `src/main.rs`: liefert `bild_daten` mit `Content-Type` `image/jpeg` oder `image/png`, je nach Endung in `bild`. Die Sitzung wird wie bei den Server-Funktionen geprüft. Ohne Anmeldung oder Recht `lager.sehen`: `403`. Kein Bild oder unbekannte `id`: `404`.

### 9.3 Externe Systeme

Entfällt, weil keine externen Systeme beteiligt sind. Bilder liegen in der Datenbank, deshalb ist kein Dateispeicher auf dem Server nötig.

## 10. Nicht-funktionale Anforderungen

| Thema | Anforderung |
|---|---|
| Performance | Die Liste lädt ohne `bild_daten`. Bilder werden einzeln über die Bild-Route geladen. Bilder sind höchstens 2 MB gross. |
| Sicherheit | Rechte serverseitig in allen Server-Funktionen und in der Bild-Route. Bildformat am Dateikopf prüfen, Grösse begrenzen, nur Bind-Parameter. |
| Datenschutz | Bilder können Personen zeigen. Sichtbar nur mit `lager.sehen`. Das Löschen eines Artikels löscht auch sein Bild. |
| Nachvollziehbarkeit | `erstellt_am` und `geaendert_am` werden gesetzt. Wer geändert hat, wird nicht gespeichert. |
| Mehrbenutzer | Letzte Änderung gewinnt. |
| Kompatibilität | Wie die übrige App: aktuelle Browser, Bildschirmbreiten ab Smartphone. |

## 11. Fehlerbehandlung und Randfälle

| Fall | Erwartetes Verhalten |
|---|---|
| Keine Kleidergrösse gewählt oder Verweis auf eine nicht vorhandene Grösse | Dialog zeigt die Meldung aus GR-1. Die Datenbank lehnt den Datensatz ab. |
| `bezeichnung` ist leer oder nur Leerzeichen | Meldung aus GR-2, nichts gespeichert. |
| `bezeichnung` kommt doppelt vor | Erlaubt (GR-3). |
| Bild größer als 2 MB | Meldung aus GR-5, nichts gespeichert. |
| Datei mit Endung `.gif` oder mit JPG-Endung, aber falschem Inhalt | Meldung aus GR-4, nichts gespeichert. |
| Artikel wird gelöscht, während der Dialog offen ist | Meldung „Der Artikel existiert nicht mehr.“, Liste wird neu geladen. |
| Keine Artikel vorhanden | Leer-Hinweis (Abschnitt 8.2). |
| Recht `lager.sehen` während der Sitzung entzogen | Die Kachel bleibt bis zur nächsten Seitenladung. Beim Öffnen leitet der Bereichsschutz auf `/` um. |
| Sitzung abgelaufen | Weiterleitung zu `/anmelden` (unverändert). |

## 12. Akzeptanzkriterien

Jedes Kriterium ist prüfbar und einer Story zugeordnet.

```gherkin
Szenario: Tabelle wird angelegt (US-1)
  Angenommen die Migration 20261030000001_artikel ist noch nicht ausgeführt
  Wenn die App startet
  Dann existiert die Tabelle artikel mit den Spalten id, bezeichnung, bild, bild_daten, kleidergroesse_id, erstellt_am und geaendert_am

Szenario: Verweis auf eine nicht vorhandene Kleidergrösse wird abgelehnt (US-1)
  Angenommen die Tabelle artikel existiert
  Wenn ein Artikel auf eine nicht vorhandene Kleidergrösse verweist
  Dann lehnt die Datenbank den Datensatz ab

Szenario: Artikel mit Bild anlegen (US-3, US-4)
  Angenommen ein Benutzer hat lager.sehen und lager.anlegen
  Wenn er einen Artikel mit Bezeichnung, Grösse M und einem JPG von 500 KB speichert
  Dann erscheint der Artikel in der Liste und das Bild als Vorschau im Dialog

Szenario: Zu grosses Bild wird abgelehnt (US-4)
  Angenommen ein Benutzer hat lager.anlegen
  Wenn er ein Bild von 3 MB hochladen will
  Dann erscheint die Meldung „Das Bild ist grösser als 2 MB.“ und nichts wird gespeichert

Szenario: Benutzer ohne Recht sieht keine Artikel (US-2)
  Angenommen ein Benutzer hat kein Recht lager.sehen
  Wenn er /lager/artikel öffnet
  Dann landet er auf /

Szenario: Artikel löschen (US-3)
  Angenommen ein Benutzer hat lager.loeschen
  Wenn er einen Artikel löscht und die Bestätigung bestätigt
  Dann ist der Artikel nicht mehr in der Liste
```

- [ ] AK-1 (US-1): Die Tabelle `artikel` verweist über `kleidergroesse_id` auf die Tabelle `kleidergroesse` (F-015).
- [ ] AK-2 (US-1): Ein Artikel mit einer vorhandenen Kleidergrösse wird gespeichert. Ein unbekannter Verweis wird von der Datenbank abgelehnt.
- [ ] AK-3 (US-3): Eine leere Bezeichnung oder nur Leerzeichen wird in der Datenbank und im Dialog abgelehnt.
- [ ] AK-4 (US-2): `/lager/artikel` zeigt Bezeichnung und Grösse, sortiert nach Bezeichnung. Ohne Artikel erscheint der Leer-Hinweis.
- [ ] AK-5 (US-3): Ein Benutzer mit `lager.sehen`, aber ohne `lager.anlegen`, sieht keinen Knopf „Neu“.
- [ ] AK-6 (US-4): Ein JPG oder PNG bis 2 MB wird gespeichert und im Dialog als Vorschau gezeigt.
- [ ] AK-7 (US-4): Ein Bild über 2 MB, eine `.gif`-Datei und eine Datei mit falschem Inhalt werden abgelehnt. Es wird nichts gespeichert.
- [ ] AK-8 (US-4): Bearbeiten ohne neue Datei behält das Bild. „Bild entfernen“ löscht es.
- [ ] AK-9 (US-3): Löschen zeigt eine Bestätigung und entfernt den Artikel danach.
- [ ] AK-10 (US-2): Ohne `lager.sehen` leitet `/lager/artikel` auf `/` um, und der Menüpunkt „Artikel“ fehlt.
- [ ] AK-11 (US-2): Der Menüpunkt „Artikel“ und die Kachel in der Startansicht führen zu `/lager/artikel`.
- [ ] AK-12 (US-4): `GET /artikel/{id}/bild` liefert das Bild nur mit `lager.sehen`, sonst `403`.
- [ ] AK-13 (Technik): Skill `build` und Skill `test` laufen ohne Fehler.

## 13. Teststrategie

| Ebene | Was | Wo |
|---|---|---|
| Unit | Validierung: Bezeichnung mit Leerzeichen, Grösse, Bildendung, Bildgrösse (2 MB erlaubt, 2 MB + 1 Byte abgelehnt), Dateikopf JPG und PNG | `#[cfg(test)]` in `src/backend/artikel/model.rs` |
| Unit | `Bereich::Lager` hat `Sehen`, `Anlegen`, `Bearbeiten`, `Loeschen` | `#[cfg(test)]` in `src/models.rs` |
| Datenbank | Migration, Constraints (Kleidergrösse, Bezeichnung, `bild`/`bild_daten`), `artikel_laden` ohne `bild_daten`. Rechte: Admin, Rolle mit Recht, Rolle ohne Recht | `#[sqlx::test]` mit Präfix `db_` (siehe Skill `test`) |
| Build | Skill `build` (Server und WASM) | lokal oder Cloud-Session |
| Manuell / UI | AK-4 bis AK-11 im Browser | Checkliste unten |

Manuelle Prüfliste:

- [ ] Als Admin: Artikel mit JPG anlegen, bearbeiten, Bild entfernen, löschen
- [ ] Als Rolle mit nur `lager.sehen`: Liste sichtbar, kein Knopf „Neu“
- [ ] Ohne Recht: Menüpunkt fehlt, `/lager/artikel` leitet auf `/`
- [ ] Bild über 2 MB und eine `.gif`-Datei: Fehlermeldung, nichts gespeichert

## 14. Betrieb und Einführung

- Neue Umgebungsvariablen: Entfällt, weil die Bilder in der Datenbank liegen. Ein Speicherordner oder Volume auf Railway ist nicht nötig.
- Migration läuft beim Start automatisch (`sqlx::migrate!` in `src/backend/db.rs`). Dauer: Entfällt, weil die Tabelle neu und leer ist.
- Datenübernahme / Initialdaten: Entfällt, weil der Bestand leer startet.
- Rückfallplan: Bestehende Migrationen werden nicht geändert. Ein Rückbau erfolgt über eine neue Migration (`DROP TABLE artikel; DROP TYPE artikel_groesse;`), solange keine Artikel erfasst sind. Nach F-015 gilt der Rückbau aus dessen Abschnitt 14. Die Aktionen von `Bereich::Lager` werden auf `[Sehen]` zurückgesetzt.
- Information an die Benutzer: Neuer Menüpunkt „Artikel“ unter Lager. Die Rechte Anlegen, Bearbeiten und Löschen für Lager müssen Rollen zugewiesen werden. Die Startansicht zeigt dann zehn Kacheln, ohne „Lager“.

## 15. Offene Fragen und Entscheidungen

| # | Frage / Entscheidung | Status | Antwort / Begründung | Datum |
|---|---|---|---|---|
| 1 | Umfang: nur Tabelle oder mit Oberfläche? | entschieden | Mit Oberfläche: Liste, Dialog, Löschen, Bildupload (Abschnitt 2). | 2026-10-30 |
| 2 | Zu welchem Bereich gehört Artikel? | entschieden | Bestehender Bereich Lager. Seine Aktionen werden auf die Standardaktionen erweitert. | 2026-10-30 |
| 3 | Was speichert das Feld `bild`? | entschieden | Eine hochgeladene Datei. | 2026-10-30 |
| 4 | Wo wird der Bildinhalt gespeichert? | entschieden | In der Datenbank (`bild_daten`, `BYTEA`). Ein Ordner auf dem Server wäre auf Railway ohne Volume nicht dauerhaft. | 2026-10-30 |
| 5 | Bildtyp: Wie passt „String“ zu einer Datei? | entschieden | `bild` (`TEXT`) hält den Dateinamen, `bild_daten` (`BYTEA`) den Inhalt. | 2026-10-30 |
| 6 | Bildformate und maximale Grösse | entschieden | JPG und PNG, höchstens 2 MB. | 2026-10-30 |
| 7 | Pflichtfelder | entschieden | Bezeichnung und Grösse sind Pflicht, Bild ist optional. | 2026-10-30 |
| 8 | Muss `bezeichnung` eindeutig sein? | entschieden | Nein, Duplikate sind erlaubt. | 2026-10-30 |
| 9 | Sollen `erstellt_am` und `geaendert_am` geführt werden? | entschieden | Ja, beide. | 2026-10-30 |
| 10 | Gibt es bestehende Artikel, die übernommen werden? | entschieden | Nein, die Tabelle startet leer. | 2026-10-30 |
| 11 | Wo liegt die Artikelseite? | entschieden | Unter `/lager/artikel`. `/lager` bleibt Platzhalter. | 2026-10-30 |
| 12 | Wie gelangt man im Menü zu Artikel? | entschieden | Gruppe „Lager“ mit Eintrag „Artikel“, wie Personal › Weiterbildung. Der Eintrag „Lager“ entfällt, die Startansicht zeigt zehn Module (siehe F-010). | 2026-10-30 |
| 13 | Standardentscheidungen von Claude, bitte prüfen | entschieden (Vorschlag) | Sortierung nach Bezeichnung aufsteigend; Grösse ohne Vorauswahl; Löschen nach Bestätigung; „Bild entfernen“ möglich; keine Suche und kein Filter; kein Bild in der Liste; Bildformat am Dateikopf prüfen. | 2026-10-30 |
| 14 | Grösse als Enum wird durch die Relation Kleidergrösse ersetzt | entschieden | Die Tabelle `kleidergroesse` (N:1) ersetzt den Enum `artikel_groesse`. Details in F-015. | 2026-10-30 |

## 16. Aufgaben

Werden in [task.md](../task.md) übernommen (T-015 bis T-021).

- [ ] Migration `migrations/20261030000001_artikel.sql`: Datentyp `artikel_groesse`, Tabelle `artikel`, CHECKs (T-015)
- [ ] `Bereich::Lager` bekommt die Standardaktionen in `src/models.rs`, mit Unit-Test (T-016)
- [ ] Backend `src/backend/artikel/`: `model.rs` (Eingabe, Validierung, Tests), `repository.rs` (SQL, `db_`-Tests), `server_fns.rs`, `mod.rs` (T-017)
- [ ] Bild-Route `GET /artikel/{id}/bild` in `src/main.rs` (T-018)
- [ ] Komponente `src/components/artikel/`: Seite mit Liste, Dialog und Löschen (T-019)
- [ ] Route `/lager/artikel` in `src/app.rs`, Menüpunkt in `src/navigation.rs`, Gruppe Lager mit Artikel, Test `zehn_module`, F-010 nachgeführt (T-020)
- [ ] Skill `build` und Skill `test` ohne Fehler, AK-1 bis AK-13 prüfen, `docs/data-model.md` nachführen (T-021)

## 17. Änderungsverlauf

| Datum | Version | Änderung | Wer |
|---|---|---|---|
| 2026-10-30 | 0.1 | Erster Entwurf nach Vorlage, offene Fragen in Abschnitt 15 | Claude |
| 2026-10-30 | 0.2 | Fragen geklärt (Umfang mit Oberfläche, Bereich Lager, Bild in der Datenbank), Abschnitte ergänzt, Aufgaben abgeleitet | Claude |
| 2026-10-30 | 0.3 | Menüpunkt „Artikel“ als eingerückter Unterpunkt unter Lager | Claude |
| 2026-10-30 | 0.4 | Gruppe „Lager“ wie Personal › Weiterbildung, Eintrag „Lager“ entfällt, /lager leitet auf /lager/artikel | Claude |
| 2026-10-30 | 0.5 | Grösse als Relation zu Kleidergrösse (F-015) statt Enum `artikel_groesse` | Claude |
