# F-015: Kleidergrösse als Tabelle

| Feld | Wert |
|---|---|
| ID | F-015 |
| Status | Entwurf |
| Priorität | Soll (vorläufig, bitte bestätigen) |
| Bereich (ERP) | Lager (vorläufig, wie Artikel) |
| Verantwortlich | offen (bitte ergänzen) |
| Erstellt / geändert | 2026-10-30 / 2026-10-30 |
| Abhängig von | F-011 (Tabelle `artikel`, deren Grösse ersetzt wird) |
| Roadmap | [features/index.md](index.md) |

---

## 1. Ziel und Nutzen

- **Problem:** Die Grösse eines Artikels ist ein fester Datentyp mit den Werten S, M, L, XL (`artikel_groesse`). Eine neue Grösse braucht eine Migration. Eine Grösse hat keine Bezeichnung und keine Reihenfolge.
- **Ziel:** Kleidergrössen stehen in einer eigenen Tabelle `kleidergroesse` mit Bezeichnung. Jeder Artikel verweist per Fremdschlüssel auf genau eine Kleidergrösse (N:1). Der Datentyp `artikel_groesse` entfällt.
- **Nutzen:** Neue Grössen brauchen keine Migration mehr. Messbar: Nach der Migration verweist jeder Artikel auf eine Kleidergrösse, und der Datentyp `artikel_groesse` ist entfernt.

## 2. Abgrenzung

| Im Umfang | Nicht im Umfang |
|---|---|
| Tabelle `kleidergroesse` mit `id` und `bezeichnung`; Fremdschlüssel `artikel.kleidergroesse_id` (N:1); Entfernen von `artikel.groesse` und des Datentyps `artikel_groesse`; Übernahme der bestehenden Grössen (Vorschlag, Frage 3); Auswahl der Kleidergrösse im Artikelformular und Anzeige in der Artikelliste (Änderung an F-011); Datenbanktests; `docs/data-model.md` | Verwaltung der Kleidergrössen in der Oberfläche, also anlegen, ändern und löschen (Frage 1); Umrechnung zwischen Grössensystemen; Grössen für andere Artikel als Kleidung; Mengen oder Bestände je Grösse |

## 3. Begriffe

| Begriff | Bedeutung im Projekt |
|---|---|
| Kleidergrösse | Eine Grösse für Kleidung, z. B. S, M, L, XL. Ein Datensatz der Tabelle `kleidergroesse`. |
| Bezeichnung | Anzeigetext der Grösse. Spalte `bezeichnung`, Datentyp `TEXT`. |
| N:1 | Viele Artikel gehören zu derselben Kleidergrösse. Ein Artikel hat genau eine Kleidergrösse. |

## 4. Rollen und Berechtigungen

Rechte haben das Format `<bereich>.<aktion>` (siehe `Bereich` und `Aktion` in `src/models.rs`).
Admin-Rollen (`rolle.ist_admin`) haben immer alle Rechte.

Die Verwaltung der Kleidergrössen ist nicht im Umfang (Frage 1). Deshalb entstehen in diesem Feature keine neuen Rechte.

| Recht | Wer braucht es | Was ist damit erlaubt |
|---|---|---|
| `lager.sehen` | Rollen, die Artikel ansehen | Kleidergrösse in der Artikelliste sehen, Auswahlliste laden |
| `lager.anlegen`, `lager.bearbeiten` | Rollen, die Artikel erfassen oder ändern | Kleidergrösse im Artikelformular wählen |
| `<bereich>.anlegen`, `<bereich>.bearbeiten`, `<bereich>.loeschen` | offen (Frage 4) | Verwaltung der Kleidergrössen, nicht im Umfang |

- Neuer `Bereich` nötig? offen (Frage 4). Vorschlag: Bereich `Lager`, keine neuen Rechte.
- Was sieht ein Benutzer ohne Recht? Wie bei Artikel (F-011): Kein Zugriff auf die Artikelseite, also auch keine Auswahl.
- Serverseitige Prüfung: in der Server-Funktion `kleidergroessen_laden` (Abschnitt 9).

## 5. User Stories

| ID | Als … | möchte ich … | damit … | Priorität |
|---|---|---|---|---|
| US-1 | Projektteam | Kleidergrössen mit Bezeichnung in einer eigenen Tabelle führen | Artikel auf eine Grösse verweisen können | Muss |
| US-2 | Benutzer mit `lager.anlegen` | im Artikelformular eine Kleidergrösse auswählen | der Artikel die richtige Grösse hat | Muss |
| US-3 | Projektteam | bestehende Artikel mit ihrer bisherigen Grösse wiederfinden | keine Grösse verloren geht | Muss |

## 6. Fachliche Anforderungen

### 6.1 Funktionale Anforderungen

| ID | Anforderung | Story |
|---|---|---|
| FA-1 | Das System legt die Tabelle `kleidergroesse` mit `id` und `bezeichnung` an. | US-1 |
| FA-2 | Jeder Artikel verweist über `artikel.kleidergroesse_id` auf genau eine Kleidergrösse. | US-1, US-2 |
| FA-3 | Die Spalte `artikel.groesse` und der Datentyp `artikel_groesse` entfallen. | US-1 |
| FA-4 | Bestehende Artikel erhalten eine Kleidergrösse aus ihrem bisherigen Wert (Übernahme, Frage 3). | US-3 |
| FA-5 | Das Artikelformular zeigt die Kleidergrössen als Auswahl, in der festgelegten Reihenfolge (Frage 5). | US-2 |
| FA-6 | Die Artikelliste zeigt die Bezeichnung der Kleidergrösse. | US-2 |

### 6.2 Geschäftsregeln und Validierung

| ID | Regel | Wo geprüft (DB / Server / UI) | Meldung an den Benutzer |
|---|---|---|---|
| GR-1 | `kleidergroesse_id` ist Pflicht (Frage 6) und verweist auf eine vorhandene Kleidergrösse. | DB (NOT NULL, Fremdschlüssel), UI (Auswahl) | „Bitte eine Grösse wählen.“ |
| GR-2 | `bezeichnung` ist Pflicht. Leerzeichen am Rand zählen nicht. | DB (CHECK `btrim(bezeichnung) <> ''`), UI | „Bitte eine Bezeichnung eingeben.“ |
| GR-3 | Eine Kleidergrösse, die von Artikeln verwendet wird, lässt sich nicht löschen. | DB (`ON DELETE RESTRICT`) | „Die Kleidergrösse kann nicht gelöscht werden, solange Artikel sie verwenden.“ |

### 6.3 Status und Abläufe

Entfällt, weil die Kleidergrösse keinen Lebenszyklus hat.

## 7. Datenmodell

Bezug: [docs/data-model.md](../docs/data-model.md). Nach der Umsetzung dort nachführen.

### 7.1 Neue oder geänderte Tabellen

| Tabelle | Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung | Bemerkung |
|---|---|---|---|---|---|---|
| `kleidergroesse` | `id` | `UUID` | nein | `gen_random_uuid()` | PK | |
| `kleidergroesse` | `bezeichnung` | `TEXT` | nein | | CHECK `btrim(bezeichnung) <> ''` | Eindeutig? Frage 2 |
| `artikel` | `kleidergroesse_id` | `UUID` | nein (Frage 6) | | FK → `kleidergroesse.id`, `ON DELETE RESTRICT` | Neu, ersetzt `groesse` |
| `artikel` | `groesse` | `artikel_groesse` | – | – | entfällt | Spalte und Datentyp werden entfernt |

### 7.2 Beziehungen

| Von | Nach | Kardinalität | `ON DELETE` | Begründung |
|---|---|---|---|---|
| `artikel.kleidergroesse_id` | `kleidergroesse.id` | N:1 | RESTRICT | Eine Kleidergrösse, die Artikel verwenden, darf nicht verschwinden. |

### 7.3 Migration

- Datei: `migrations/20261031000001_kleidergroesse.sql` (nach der letzten Migration `20261030000001_artikel.sql`). Bestehende Migrationen bleiben unverändert.
- Umgang mit **bestehenden Daten** (Vorschlag, Frage 3):
  1. Tabelle `kleidergroesse` anlegen und die Grössen `S`, `M`, `L`, `XL` eintragen.
  2. Jeden Artikel über seinen bisherigen Wert `groesse` der passenden Kleidergrösse zuordnen.
  3. Danach `artikel.groesse` und den Datentyp `artikel_groesse` entfernen.
  4. Erst dann `kleidergroesse_id` als `NOT NULL` setzen, damit die Migration nicht an leeren Verweisen scheitert.
- Indizes: Index auf `artikel.kleidergroesse_id`.
- Rechte für bestehende Rollen: keine Migration nötig.

## 8. Benutzeroberfläche

### 8.1 Navigation und Route

Entfällt, weil keine eigene Seite entsteht (Frage 1). Die Auswahl erscheint im Artikelformular (F-011).

### 8.2 Ansichten

Die Artikelliste zeigt in der Spalte „Grösse“ die Bezeichnung der Kleidergrösse (FA-6). Es entsteht keine neue Ansicht.

### 8.3 Formularfelder

| Feld | Typ / Eingabe | Pflicht | Standard | Validierung | Hilfetext |
|---|---|---|---|---|---|
| Grösse | Auswahl aus den Kleidergrössen | ja (Frage 6) | keine Vorauswahl | GR-1 | – |

### 8.4 Gestaltung und Bedienbarkeit

Entfällt, weil es keine neue Oberfläche gibt. Die Auswahl folgt den Formularfeldern von F-011.

## 9. Schnittstellen

### 9.1 Server-Funktionen

| Funktion | Datei | Parameter | Rückgabe | Recht | Fehlerfälle |
|---|---|---|---|---|---|
| `kleidergroessen_laden` | `src/backend/artikel/server_fns.rs` | – | `Vec<Kleidergroesse>` (`id`, `bezeichnung`, in der festgelegten Reihenfolge) | `lager.sehen` | nicht angemeldet, kein Recht |

- `artikel_anlegen` und `artikel_bearbeiten` übergeben statt `groesse` die `kleidergroesse_id` (`Uuid`).
- Die Verwaltung (anlegen, ändern, löschen) ist nicht im Umfang (Frage 1).

### 9.2 Weitere Endpunkte und Exporte

Entfällt, weil keine Route in `src/main.rs` nötig ist.

### 9.3 Externe Systeme

Entfällt, weil keine externen Systeme beteiligt sind.

## 10. Nicht-funktionale Anforderungen

| Thema | Anforderung |
|---|---|
| Performance | Die Auswahl lädt alle Kleidergrössen. Es sind wenige Einträge. |
| Sicherheit | Der Fremdschlüssel verhindert ungültige Grössen. Rechte werden serverseitig geprüft (`lager.sehen`). |
| Datenschutz | Entfällt, weil keine Personendaten gespeichert werden. |
| Nachvollziehbarkeit | Entfällt, weil die Verwaltung nicht im Umfang ist. |
| Mehrbenutzer | Entfällt, weil in diesem Umfang nur gelesen und zugeordnet wird. |
| Kompatibilität | Wie F-011. |

## 11. Fehlerbehandlung und Randfälle

| Fall | Erwartetes Verhalten |
|---|---|
| Kleidergrösse wird gelöscht, während Artikel sie verwenden | Die Datenbank verweigert das Löschen (GR-3). |
| Artikel ohne Kleidergrösse speichern | Meldung aus GR-1, nichts gespeichert. |
| Keine Kleidergrösse vorhanden | Die Auswahl ist leer, der Artikel lässt sich nicht speichern. Hinweistext offen (Frage 1). |
| Kleidergrösse wird geändert oder gelöscht, während das Formular offen ist | Meldung „Die Auswahl ist nicht mehr gültig, bitte neu wählen.“ (Vorschlag) |
| Bestehender Artikel ohne bisherigen Wert | Offen (Frage 3). |
| Doppelte Bezeichnung | Offen (Frage 2). |

## 12. Akzeptanzkriterien

Jedes Kriterium ist prüfbar und einer Story zugeordnet.

```gherkin
Szenario: Kleidergrösse wird angelegt (US-1)
  Angenommen die Tabelle kleidergroesse existiert
  Wenn eine Kleidergrösse mit der Bezeichnung XL gespeichert wird
  Dann ist sie in der Auswahl der Artikel vorhanden

Szenario: Artikel verweist auf eine Kleidergrösse (US-2)
  Angenommen ein Benutzer hat lager.anlegen und die Kleidergrösse M existiert
  Wenn er einen Artikel mit der Grösse M speichert
  Dann zeigt die Artikelliste M in der Spalte Grösse

Szenario: Kleidergrösse mit Artikeln lässt sich nicht löschen (US-1)
  Angenommen ein Artikel verweist auf die Kleidergrösse M
  Wenn M gelöscht werden soll
  Dann lehnt die Datenbank das Löschen ab

Szenario: Bestehende Artikel behalten ihre Grösse (US-3)
  Angenommen ein Artikel hatte vor der Migration die Grösse L
  Wenn die Migration läuft
  Dann verweist der Artikel auf die Kleidergrösse L
```

- [ ] AK-1 (US-1): Die Tabelle `kleidergroesse` hat `id` und `bezeichnung`. Eine leere Bezeichnung lehnt die Datenbank ab.
- [ ] AK-2 (US-1): Die Spalte `artikel.groesse` und der Datentyp `artikel_groesse` sind entfernt.
- [ ] AK-3 (US-1, US-2): Jeder Artikel verweist auf eine vorhandene Kleidergrösse. Ohne Verweis speichert die Datenbank nicht.
- [ ] AK-4 (US-1): Eine Kleidergrösse mit Artikeln lässt sich nicht löschen (`RESTRICT`).
- [ ] AK-5 (US-2): Die Auswahl im Artikelformular zeigt die Kleidergrössen in der festgelegten Reihenfolge.
- [ ] AK-6 (US-3): Nach der Migration hat jeder bestehende Artikel eine Kleidergrösse, die seinem bisherigen Wert entspricht.
- [ ] AK-7 (Technik): Skill `build` und Skill `test` laufen ohne Fehler.

## 13. Teststrategie

| Ebene | Was | Wo |
|---|---|---|
| Unit | Eingabeprüfung: `kleidergroesse_id` ist Pflicht | `#[cfg(test)]` in `src/backend/artikel/model.rs` |
| Datenbank | Verweis gültig und ungültig, Pflicht, `RESTRICT` beim Löschen, leere Bezeichnung abgelehnt, Übernahme der Werte | `#[sqlx::test]` mit Präfix `db_` (siehe Skill `test`) |
| Build | Skill `build` (Server und WASM) | lokal oder Cloud-Session |
| Manuell / UI | AK-5 im Browser | Checkliste unten |

Manuelle Prüfliste:

- [ ] Die Auswahl im Artikelformular zeigt alle Kleidergrössen in der richtigen Reihenfolge
- [ ] Die Artikelliste zeigt die Bezeichnung der Kleidergrösse

## 14. Betrieb und Einführung

- Neue Umgebungsvariablen: Entfällt.
- Migration läuft beim Start automatisch (`sqlx::migrate!` in `src/backend/db.rs`). Dauer: bei wenigen Artikeln unkritisch, bei grossen Beständen zu prüfen.
- Datenübernahme: wie in 7.3 beschrieben (Frage 3).
- Rückfallplan: Bestehende Migrationen bleiben unverändert. Der Rückbau erfolgt über eine neue Migration, die `artikel.groesse` und den Datentyp `artikel_groesse` wieder anlegt und die Werte aus `kleidergroesse` zurückschreibt.
- Information an die Benutzer: Die Grösse wird künftig aus einer Liste gewählt. Diese Liste kommt aus der Tabelle `kleidergroesse`.

## 15. Offene Fragen und Entscheidungen

| # | Frage / Entscheidung | Status | Antwort / Begründung | Datum |
|---|---|---|---|---|
| 1 | Umfang: Verwaltung der Kleidergrössen (anlegen, ändern, löschen) in der Oberfläche oder vorerst nur in der Datenbank? | offen | Vorschlag: vorerst nur Tabelle und Auswahl. Die Verwaltung kommt als Folgefeature. | 2026-10-30 |
| 2 | Muss die Bezeichnung einer Kleidergrösse eindeutig sein? | offen | Vorschlag: ja, UNIQUE. | 2026-10-30 |
| 3 | Übernahme: Sollen S, M, L, XL als Kleidergrössen angelegt und den bestehenden Artikeln zugeordnet werden? | offen | Vorschlag: ja, siehe 7.3. | 2026-10-30 |
| 4 | Bereich und Rechte für die Verwaltung der Kleidergrössen | offen | Vorschlag: Bereich Lager, keine neuen Rechte, bis Frage 1 entschieden ist. | 2026-10-30 |
| 5 | Reihenfolge der Auswahl. Alphabetisch ergäbe L, M, S, XL. | offen | Vorschlag: Spalte `sortierung` (`INTEGER`), Reihenfolge S, M, L, XL. | 2026-10-30 |
| 6 | Muss ein Artikel eine Kleidergrösse haben? | offen | Vorschlag: ja, `NOT NULL` (GR-1). | 2026-10-30 |
| 7 | Priorität (Soll) und Bereich (Lager) bestätigen | offen | Vorläufig aus F-011 übernommen. | 2026-10-30 |

## 16. Aufgaben

Werden nach Klärung der offenen Fragen in [task.md](../task.md) übernommen.

- [ ] Migration `migrations/20261031000001_kleidergroesse.sql`: Tabelle `kleidergroesse`, Übernahme, Fremdschlüssel, Entfernen von `groesse`
- [ ] Backend `src/backend/artikel/`: Modell, Repository, Server-Funktion `kleidergroessen_laden`, Eingabe mit `kleidergroesse_id`
- [ ] Artikelformular und Artikelliste auf die Relation umstellen
- [ ] Tests und Skills `build` und `test` ohne Fehler, AK-1 bis AK-7 prüfen
- [ ] `docs/data-model.md` nachführen

## 17. Änderungsverlauf

| Datum | Version | Änderung | Wer |
|---|---|---|---|
| 2026-10-30 | 0.1 | Erster Entwurf: Relation zu Artikel (N:1) statt Enum `artikel_groesse`, offene Fragen in Abschnitt 15 | Claude |
