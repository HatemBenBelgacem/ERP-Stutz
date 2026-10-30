# F-015: Grössen je Artikel als Text

| Feld | Wert |
|---|---|
| ID | F-015 |
| Status | Entwurf |
| Priorität | Soll (vorläufig, bitte bestätigen) |
| Bereich (ERP) | Lager (vorläufig, wie Artikel) |
| Verantwortlich | offen (bitte ergänzen) |
| Erstellt / geändert | 2026-10-30 / 2026-10-30 |
| Abhängig von | F-011 (Tabelle `artikel`, Artikelformular und -liste) |
| Roadmap | [features/index.md](index.md) |

---

## 1. Ziel und Nutzen

- **Problem:** Ein Artikel hat meist mehrere Grössen, z. B. ein T-Shirt in S, M und L. Die bisherige Lösung ordnet jedem Artikel genau eine Grösse aus einer festen Liste zu. Das passt nicht zur Wirklichkeit, und die Beziehung läuft in die falsche Richtung.
- **Ziel:** Ein Artikel hat beliebig viele Grössen (1:N). Jede Grösse ist ein Text, den der Benutzer im Artikel erfasst, z. B. „M“ oder „42“. Die Tabelle `kleidergroesse` gehört per Fremdschlüssel zu genau einem Artikel.
- **Nutzen:** Mehrere Grössen je Artikel ohne feste Vorauswahl. Messbar: Ein Artikel mit drei Grössen speichert alle drei. Ein Artikel ohne Grösse wird abgelehnt.

## 2. Abgrenzung

| Im Umfang | Nicht im Umfang |
|---|---|
| Tabelle `kleidergroesse` mit `artikel_id`, `bezeichnung` und `sortierung`; Beziehung 1:N von `artikel` zu `kleidergroesse`; Erfassung mehrerer Grössen als Text im Artikelformular; Anzeige der Grössen in der Artikelliste; Migration mit Übernahme der bisherigen Grössen; Datenbanktests; `docs/data-model.md` | Grössenstamm, also eine Liste aller Grössen, die Artikel gemeinsam nutzen (Frage 1); eigene Verwaltung der Grössen ausserhalb des Artikels; Bestand je Grösse; Umrechnung zwischen Grössensystemen |

## 3. Begriffe

| Begriff | Bedeutung im Projekt |
|---|---|
| Grösse | Text, mit dem ein Artikel eine Ausführung beschreibt, z. B. „M“ oder „42“. Ein Datensatz der Tabelle `kleidergroesse`. |
| 1:N | Ein Artikel hat viele Grössen. Eine Grösse gehört zu genau einem Artikel. |
| Reihenfolge | Position der Grösse im Artikel, Spalte `sortierung`. Sie entspricht der Eingabereihenfolge. |

## 4. Rollen und Berechtigungen

Rechte haben das Format `<bereich>.<aktion>` (siehe `Bereich` und `Aktion` in `src/models.rs`).
Admin-Rollen (`rolle.ist_admin`) haben immer alle Rechte.

Grössen werden zusammen mit dem Artikel erfasst und geändert. Deshalb entstehen keine neuen Rechte.

| Recht | Wer braucht es | Was ist damit erlaubt |
|---|---|---|
| `lager.sehen` | Rollen, die Artikel ansehen | Grössen in der Artikelliste sehen |
| `lager.anlegen` | Rollen, die Artikel erfassen | Grössen beim Anlegen eingeben |
| `lager.bearbeiten` | Rollen, die Artikel ändern | Grössen hinzufügen, ändern und entfernen |

- Neuer `Bereich` nötig? nein. Migration für Rollen? nein.
- Was sieht ein Benutzer ohne Recht? Wie bei Artikel (F-011).
- Serverseitige Prüfung: in den Server-Funktionen von F-011 (Abschnitt 9).

## 5. User Stories

| ID | Als … | möchte ich … | damit … | Priorität |
|---|---|---|---|---|
| US-1 | Projektteam | jedem Artikel mehrere Grössen als Text zuordnen | alle Ausführungen eines Artikels erfasst sind | Muss |
| US-2 | Benutzer mit `lager.anlegen` oder `lager.bearbeiten` | Grössen im Artikelformular hinzufügen und entfernen | ich die Grössen eines Artikels pflegen kann | Muss |
| US-3 | Projektteam | bestehende Artikel behalten ihre bisherige Grösse | nichts verloren geht | Muss |

## 6. Fachliche Anforderungen

### 6.1 Funktionale Anforderungen

| ID | Anforderung | Story |
|---|---|---|
| FA-1 | Das System legt die Tabelle `kleidergroesse` mit `id`, `artikel_id`, `bezeichnung` und `sortierung` an. | US-1 |
| FA-2 | Jede Grösse gehört zu genau einem Artikel (`artikel_id`). | US-1 |
| FA-3 | Ein Artikel hat beliebig viele Grössen. Ihre Reihenfolge bleibt erhalten. | US-1 |
| FA-4 | Das Artikelformular zeigt eine Liste von Textfeldern für die Grössen. Felder lassen sich hinzufügen und entfernen. | US-2 |
| FA-5 | Die Artikelliste zeigt die Grössen eines Artikels, durch Komma getrennt. | US-2 |
| FA-6 | Bestehende Artikel erhalten ihre bisherige Grösse als Grösse ihres Artikels. | US-3 |

### 6.2 Geschäftsregeln und Validierung

| ID | Regel | Wo geprüft (DB / Server / UI) | Meldung an den Benutzer |
|---|---|---|---|
| GR-1 | Jeder Artikel hat mindestens eine Grösse (Frage 2). | Model (Server und Browser) | „Bitte mindestens eine Grösse eingeben.“ |
| GR-2 | Leerzeichen am Rand werden entfernt. Leere Felder werden ignoriert. | Model | – |
| GR-3 | Eine Grösse darf je Artikel nur einmal vorkommen. | DB (UNIQUE `artikel_id`, `bezeichnung`), Model | „Eine Grösse ist doppelt angegeben.“ |
| GR-4 | Eine Grösse hat höchstens 20 Zeichen (Frage 5). | Model | „Eine Grösse darf höchstens 20 Zeichen haben.“ |
| GR-5 | Beim Löschen eines Artikels werden seine Grössen mitgelöscht (Frage 3). | DB (`ON DELETE CASCADE`) | – |

### 6.3 Status und Abläufe

Entfällt, weil die Grösse keinen Lebenszyklus hat.

## 7. Datenmodell

Bezug: [docs/data-model.md](../docs/data-model.md). Nach der Umsetzung dort nachführen.

### 7.1 Neue oder geänderte Tabellen

| Tabelle | Spalte | Datentyp | Null | Standard | Schlüssel / Bedingung | Bemerkung |
|---|---|---|---|---|---|---|
| `kleidergroesse` | `id` | `UUID` | nein | `gen_random_uuid()` | PK | |
| `kleidergroesse` | `artikel_id` | `UUID` | nein | | FK → `artikel.id`, `ON DELETE CASCADE` | Neu, 1:N |
| `kleidergroesse` | `bezeichnung` | `TEXT` | nein | | CHECK `btrim(bezeichnung) <> ''`, UQ zusammen mit `artikel_id` | Grösse als Text |
| `kleidergroesse` | `sortierung` | `INTEGER` | nein | | | Reihenfolge im Artikel |
| `artikel` | `kleidergroesse_id` | – | – | – | entfällt | Spalte und Fremdschlüssel werden entfernt |

### 7.2 Beziehungen

| Von | Nach | Kardinalität | `ON DELETE` | Begründung |
|---|---|---|---|---|
| `artikel` | `kleidergroesse` | 1:N | CASCADE | Grössen gehören zum Artikel (Frage 3). |

### 7.3 Migration

- Datei: `migrations/20261031000002_kleidergroesse_je_artikel.sql` (nach der letzten Migration `20261031000001_kleidergroesse.sql`). Bestehende Migrationen bleiben unverändert.
- Umgang mit **bestehenden Daten**:
  1. Die bisherige Eindeutigkeit der Bezeichnung über alle Artikel hinweg entfernen.
  2. Spalte `artikel_id` anlegen.
  3. Jede bestehende Zuordnung `artikel.kleidergroesse_id` als eigene Grösse des Artikels kopieren (Bezeichnung und Reihenfolge).
  4. Spalte `artikel.kleidergroesse_id` entfernen.
  5. Die gemeinsamen Vorlagen ohne Artikel löschen. Danach `artikel_id` als `NOT NULL` setzen und die Eindeutigkeit je Artikel anlegen.
- Indizes: Index auf `kleidergroesse.artikel_id`.
- Rechte für bestehende Rollen: keine Migration nötig.

## 8. Benutzeroberfläche

### 8.1 Navigation und Route

Entfällt, weil keine eigene Seite entsteht. Die Grössen erscheinen im Artikelformular von F-011.

### 8.2 Ansichten

Die Artikelliste zeigt in der Spalte „Grösse“ alle Grössen des Artikels, durch Komma getrennt (FA-5). Keine neue Ansicht.

### 8.3 Formularfelder

| Feld | Typ / Eingabe | Pflicht | Standard | Validierung | Hilfetext |
|---|---|---|---|---|---|
| Grösse | Liste von Textfeldern, Knopf „Grösse hinzufügen“, je Feld ein Entfernen-Knopf | ja (mindestens ein Feld) | ein leeres Feld | GR-1 bis GR-4 | – |

### 8.4 Gestaltung und Bedienbarkeit

Entfällt, weil keine neue Oberfläche entsteht. Die Textfelder folgen dem Formular von F-011.

## 9. Schnittstellen

### 9.1 Server-Funktionen

- `artikel_laden` liefert zusätzlich die Grössen je Artikel als Liste von Texten, in der gespeicherten Reihenfolge.
- `artikel_anlegen` und `artikel_bearbeiten` übergeben statt einer Kleidergrösse eine Liste von Texten (`groessen: Vec<String>`).
- Die Server-Funktion `kleidergroessen_laden` entfällt.
- Die Prüfung der Grössen liegt im Modell (GR-1 bis GR-4). Der Server prüft dieselben Regeln erneut.

### 9.2 Weitere Endpunkte und Exporte

Entfällt, weil keine Route in `src/main.rs` nötig ist.

### 9.3 Externe Systeme

Entfällt, weil keine externen Systeme beteiligt sind.

## 10. Nicht-funktionale Anforderungen

| Thema | Anforderung |
|---|---|
| Performance | Grössen werden zusammen mit den Artikeln in einer Abfrage geladen. Je Artikel sind es wenige Einträge. |
| Sicherheit | Rechte wie bei Artikel. Länge der Grössen begrenzen (GR-4). Nur Bind-Parameter. |
| Datenschutz | Entfällt, weil keine Personendaten gespeichert werden. |
| Nachvollziehbarkeit | Entfällt, weil die Grössen mit dem Artikel geändert werden. |
| Mehrbenutzer | Letzte Änderung gewinnt, wie bei Artikel. |
| Kompatibilität | Wie F-011. |

## 11. Fehlerbehandlung und Randfälle

| Fall | Erwartetes Verhalten |
|---|---|
| Artikel ohne Grösse speichern | Meldung aus GR-1, nichts gespeichert. |
| Gleiche Grösse zweimal eingegeben | Meldung aus GR-3, nichts gespeichert. |
| Leere Felder zwischen Grössen | Werden ignoriert (GR-2). |
| Alle Grössen beim Bearbeiten entfernt | Meldung aus GR-1, nichts gespeichert. |
| Grösse länger als 20 Zeichen | Meldung aus GR-4, nichts gespeichert. |
| Artikel wird gelöscht | Seine Grössen werden mitgelöscht (GR-5). |

## 12. Akzeptanzkriterien

Jedes Kriterium ist prüfbar und einer Story zugeordnet.

```gherkin
Szenario: Artikel mit mehreren Grössen speichern (US-1, US-2)
  Angenommen ein Benutzer hat lager.anlegen
  Wenn er einen Artikel mit den Grössen S, M und L speichert
  Dann zeigt die Artikelliste „S, M, L“ in der Spalte Grösse

Szenario: Artikel ohne Grösse wird abgelehnt (US-2)
  Angenommen ein Benutzer hat lager.anlegen
  Wenn er einen Artikel ohne Grösse speichern will
  Dann erscheint die Meldung „Bitte mindestens eine Grösse eingeben.“

Szenario: Bestehender Artikel behält seine Grösse (US-3)
  Angenommen ein Artikel hatte vor der Migration die Grösse L
  Wenn die Migration läuft
  Dann hat der Artikel genau die Grösse L
```

- [ ] AK-1 (US-1): Ein Artikel kann mehrere Grössen haben. Jede Grösse gehört zu genau einem Artikel.
- [ ] AK-2 (US-2): Im Artikelformular lassen sich Grössen hinzufügen und entfernen.
- [ ] AK-3 (US-1): Eine doppelte Grösse je Artikel wird abgelehnt (GR-3).
- [ ] AK-4 (US-2): Ein Artikel ohne Grösse wird abgelehnt (GR-1).
- [ ] AK-5 (US-1): Löschen eines Artikels löscht seine Grössen.
- [ ] AK-6 (US-3): Nach der Migration hat jeder bestehende Artikel genau seine bisherige Grösse.
- [ ] AK-7 (Technik): Skill `build` und Skill `test` laufen ohne Fehler.

## 13. Teststrategie

| Ebene | Was | Wo |
|---|---|---|
| Unit | Bereinigung (Leerzeichen, leere Felder), Pflicht, Doppelte, Länge (GR-1 bis GR-4) | `#[cfg(test)]` in `src/backend/artikel/model.rs` |
| Datenbank | 1:N, UNIQUE je Artikel, CASCADE beim Löschen, Speichern mit mehreren Grössen | `#[sqlx::test]` mit Präfix `db_` (siehe Skill `test`) |
| Build | Skill `build` (Server und WASM) | lokal oder Cloud-Session |
| Manuell / UI | AK-2 im Browser | Checkliste unten |

Manuelle Prüfliste:

- [ ] Grössen hinzufügen, entfernen und speichern
- [ ] Die Artikelliste zeigt alle Grössen durch Komma getrennt

## 14. Betrieb und Einführung

- Neue Umgebungsvariablen: Entfällt.
- Migration läuft beim Start automatisch (`sqlx::migrate!` in `src/backend/db.rs`). Dauer: je Artikel eine Zeile, bei grossen Beständen zu prüfen.
- Datenübernahme: wie in 7.3 beschrieben. Jede bisherige Grösse wird zur Grösse ihres Artikels.
- Rückfallplan: Bestehende Migrationen bleiben unverändert. Ein Rückbau erfolgt über eine neue Migration, die `artikel.kleidergroesse_id` wieder anlegt und aus der ersten Grösse je Artikel befüllt.
- Information an die Benutzer: Die Grössen werden als Text im Artikelformular erfasst, nicht mehr aus einer festen Liste gewählt.

## 15. Offene Fragen und Entscheidungen

| # | Frage / Entscheidung | Status | Antwort / Begründung | Datum |
|---|---|---|---|---|
| 1 | Sollen Grössen über mehrere Artikel wiederverwendet werden (z. B. „M“ für alle T-Shirts)? | offen | Vorschlag: nein. Grössen gehören zu einem Artikel, wie in diesem Entwurf. | 2026-10-30 |
| 2 | Muss jeder Artikel mindestens eine Grösse haben? | offen | Vorschlag: ja (GR-1). | 2026-10-30 |
| 3 | Sollen Grössen beim Löschen des Artikels mitgelöscht werden? | offen | Vorschlag: ja (GR-5, `CASCADE`). | 2026-10-30 |
| 4 | Soll die Reihenfolge der Grössen bearbeitbar sein (nach oben und unten)? | offen | Vorschlag: nein. Die Reihenfolge ist die Eingabereihenfolge. | 2026-10-30 |
| 5 | Längenbegrenzung einer Grösse | offen | Vorschlag: höchstens 20 Zeichen (GR-4). | 2026-10-30 |
| 6 | Priorität (Soll) und Bereich (Lager) bestätigen | offen | Vorläufig aus F-011 übernommen. | 2026-10-30 |

## 16. Aufgaben

Werden nach Klärung der offenen Fragen in [task.md](../task.md) übernommen.

- [ ] Migration `migrations/20261031000002_kleidergroesse_je_artikel.sql`: Übernahme, `artikel_id`, Entfernen von `kleidergroesse_id`
- [ ] Modell, Repository und Server-Funktionen auf Grössen je Artikel umstellen; `kleidergroessen_laden` entfällt
- [ ] Artikelformular mit Textfeldern und Artikelliste mit Grössen
- [ ] Tests und Skills `build` und `test` ohne Fehler, AK-1 bis AK-7 prüfen
- [ ] `docs/data-model.md` nachführen

## 17. Änderungsverlauf

| Datum | Version | Änderung | Wer |
|---|---|---|---|
| 2026-10-30 | 0.1 | Erster Entwurf: Relation zu Artikel statt Enum, offene Fragen in Abschnitt 15 | Claude |
| 2026-10-30 | 0.2 | Richtung korrigiert: Artikel 1:N Grössen als Text; Grössen werden im Artikelformular erfasst | Claude |
