# F-010: Startansicht mit Modulen

| Feld | Wert |
|---|---|
| ID | F-010 |
| Status | Bereit |
| Priorität | Muss |
| Bereich (ERP) | Startseite (`/`) |
| Verantwortlich | offen (bitte ergänzen) |
| Erstellt / geändert | 2026-10-30 / 2026-10-30 |
| Abhängig von | F-002 (Rechte, bereits umgesetzt) |
| Roadmap | [features/index.md](index.md) |

---

## 1. Ziel und Nutzen

- **Problem:** Nach dem Login öffnet sich `/`, und die App leitet sofort auf die erste erlaubte Seite weiter, meist die Terminübersicht. Der Benutzer sieht dadurch nicht, welche Module ihm offenstehen.
- **Ziel:** `/` zeigt nach dem Login alle für den Benutzer freigegebenen Module als Kacheln (DaisyUI `card`). Ein Klick öffnet das Modul.
- **Nutzen:** Der Benutzer hat seine Module auf einen Blick. Messbar: Nach dem Login ist die erste Ansicht die Kachelübersicht, ohne Weiterleitung.

## 2. Abgrenzung

| Im Umfang | Nicht im Umfang |
|---|---|
| Route `/` als Kachelübersicht, Kacheln aus der gemeinsamen Navigationsliste, Sichtbarkeit nach Recht `sehen`, Leer-Hinweis, Umleitung ohne Recht auf `/`, Umleitung von `/personal` auf `/` | Einstellungen (Benutzer, Rollen, Geschäftsbereiche) als Kachel (bleiben im Menü), eigene Anordnung oder Ausblenden von Kacheln, Suche und Filter, Lizenzverwaltung (gibt es nicht), Datenbankänderungen, Änderungen an den Modulseiten selbst |

## 3. Begriffe

| Begriff | Bedeutung im Projekt |
|---|---|
| Modul | Ein Eintrag der Navigation (`NAVIGATION` in `src/components/layout/layout.rs`). Ein Modul kann mehrere Bereiche umfassen, z. B. Weiterbildung: Termine, Kurse, Anmeldung. Die Komponente `ModulSeite` (Platzhalter für Einkauf, Lager, Werkhof, Finanzen) hat damit nichts zu tun. |
| Bereich | Eintrag `Bereich` in `src/models.rs`. Jeder Bereich hat Rechte im Format `<bereich>.<aktion>`, z. B. `termine.sehen`. |
| Freigegeben | Der Benutzer hat das Recht `<bereich>.sehen` für mindestens einen Bereich des Moduls. Admins (`rolle.ist_admin`) haben alle Module. |
| Kachel | Anklickbares Element (DaisyUI `card`) mit Icon und Name, das zu einem Modul führt. |

## 4. Rollen und Berechtigungen

Rechte haben das Format `<bereich>.<aktion>` (siehe `Bereich` und `Aktion` in `src/models.rs`).
Admin-Rollen (`rolle.ist_admin`) haben immer alle Rechte.

Dieses Feature braucht nur das Recht `sehen`. Es entstehen keine neuen Rechte.

| Recht | Wer braucht es | Was ist damit erlaubt |
|---|---|---|
| `<bereich>.sehen` (pro Bereich eines Moduls) | Benutzer, die das Modul nutzen | Kachel des Moduls sehen und öffnen |
| `<bereich>.anlegen`, `.bearbeiten`, `.loeschen` | – | Kein Bezug zu diesem Feature |

- Neuer `Bereich` nötig: nein.
- Migration für bestehende Rollen: nein, die Rechte `sehen` gibt es bereits (`Bereich::aktionen` in `src/models.rs`).
- Was sieht ein Benutzer ohne Recht: Die Kachel erscheint nicht. Hat er kein Modul, zeigt `/` den Leer-Hinweis (Abschnitt 8.2). Öffnet er eine Modulseite ohne Recht, leitet der Bereichsschutz auf `/` um.
- Serverseitige Prüfung: Die Rechte kommen vom Server (`aktueller_benutzer_laden` in `src/app.rs`). Die Kacheln sind nur Navigation und verschaffen keinen Zugriff. Der Schutz der Modulseiten bleibt unverändert.

## 5. User Stories

| ID | Als … | möchte ich … | damit … | Priorität |
|---|---|---|---|---|
| US-1 | angemeldeter Benutzer | nach dem Login alle für mich freigegebenen Module als Kacheln sehen und per Klick öffnen | ich sofort sehe, womit ich arbeiten kann | Muss |

## 6. Fachliche Anforderungen

### 6.1 Funktionale Anforderungen

| ID | Anforderung | Story |
|---|---|---|
| FA-1 | `/` zeigt nach dem Login die Kachelübersicht statt einer Weiterleitung. | US-1 |
| FA-2 | Pro Eintrag der Navigation gibt es genau eine Kachel. Sie ist sichtbar, wenn mindestens ein Bereich des Moduls freigegeben ist. | US-1 |
| FA-3 | Ein Klick auf eine Kachel öffnet die Route aus Abschnitt 8.1. | US-1 |
| FA-4 | Kacheln und Menüpunkte stammen aus derselben Liste (`NAVIGATION`), damit sie nicht auseinanderlaufen. | US-1 |
| FA-5 | Ist keine Kachel sichtbar, zeigt `/` einen Leer-Hinweis und leitet nicht weiter. | US-1 |
| FA-6 | Eine Modulseite ohne Recht ist nicht erreichbar. Der Bereichsschutz leitet auf `/` um. | US-1 |

### 6.2 Geschäftsregeln und Validierung

| ID | Regel | Wo geprüft (DB / Server / UI) | Meldung an den Benutzer |
|---|---|---|---|
| GR-1 | Eine Kachel ist sichtbar, wenn der Benutzer `sehen` für mindestens einen Bereich des Moduls hat. Admins sehen alle. | UI (Filter), Rechte kommen vom Server | – |
| GR-2 | Ohne sichtbare Kachel: Leer-Hinweis, keine Weiterleitung. | UI | „Noch keine Module freigegeben. Bitte an die Administration wenden.“ |

### 6.3 Status und Abläufe

Entfällt, weil das Feature keinen Lebenszyklus hat.

## 7. Datenmodell

Bezug: [docs/data-model.md](../docs/data-model.md). Entfällt, weil keine Tabelle angelegt oder geändert wird.

### 7.1 Neue oder geänderte Tabellen

Entfällt, weil keine Tabelle und keine Spalte geändert werden.

### 7.2 Beziehungen

Entfällt, weil keine Beziehung neu entsteht.

### 7.3 Migration

Entfällt, weil keine Migration nötig ist. Bestehende Rollen behalten ihre Rechte.

## 8. Benutzeroberfläche

### 8.1 Navigation und Route

Die Route `/` zeigt künftig die Startansicht. Sie bleibt innerhalb des Login-Schutzes (`ProtectedParentRoute` in `src/app.rs`). Sie gehört keinem Bereich, deshalb gibt es hier keinen `Bereichsschutz`.

Komponente: `src/components/startansicht/startansicht.rs` (mit `mod.rs`, Registrierung in `src/components/mod.rs`).

Kacheln in der Reihenfolge des Menüs:

| Kachel | Ziel | Sichtbar bei Recht `sehen` für |
|---|---|---|
| Weiterbildung | `/personal/weiterbildung` | `termine`, `kurse` oder `anmeldung` |
| Mitarbeiter | `/personal/mitarbeiter` | `mitarbeiter` |
| Adressen | `/adressen` | `adressen` |
| Einkauf | `/einkauf` | `einkauf` |
| Artikel | `/lager/artikel` | `lager` (F-011, in der Gruppe Lager) |
| Projekte | `/projekte` | `projekte` oder `bauobjekte` |
| Stunden erfassen | `/stunden-erfassen` | `stunden` |
| Buchungen | `/buchungen` | `buchungen` |
| Werkhof | `/werkhof` | `werkhof` |
| Finanzen | `/finanzen` | `finanzen` |

Einstellungen ist keine Kachel. Der Eintrag bleibt im Menü.

### 8.2 Ansichten

Eine Ansicht: die Startansicht.

- **Zweck und Inhalt:** Überschrift „Module“ und ein Kachelraster. Jede Kachel zeigt Icon und Name. Keine Sortierung, kein Filter, keine Suche.
- **Aktionen:** Eine Kachel anklicken öffnet das Modul. Weitere Knöpfe gibt es nicht.
- **Zustände:**
  - Laden: Ladeanzeige wie auf den übrigen Seiten.
  - Leer (kein freigegebenes Modul): `Leer`-Komponente mit dem Text aus GR-2.
  - Fehler: Fehlermeldung, falls die Rechte nicht geladen werden können. Ohne angemeldeten Benutzer greift der Login-Schutz (Abschnitt 11).
  - Ohne Recht: entspricht dem Leer-Zustand.
- **Skizze / Screenshot:** Entfällt, weil das Kachelraster in diesem Abschnitt vollständig beschrieben ist.

### 8.3 Formularfelder

Entfällt, weil es keine Eingabe gibt.

### 8.4 Gestaltung und Bedienbarkeit

- DaisyUI `card` für die Kacheln. Icons wie in der Navigation (`icondata`).
- Kacheln sind Links, sie lassen sich mit der Tastatur bedienen, der Fokus ist sichtbar.
- Mobil: eine Spalte. Ab Tablet: mehrere Spalten.
- Texte auf Deutsch.

## 9. Schnittstellen

### 9.1 Server-Funktionen

Entfällt, weil keine neue Server-Funktion nötig ist. Die Rechte kommen aus der bestehenden Funktion `aktueller_benutzer_laden` (`src/app.rs`).

### 9.2 Weitere Endpunkte und Exporte

Entfällt, weil keine neue Route in `src/main.rs` nötig ist.

### 9.3 Externe Systeme

Entfällt, weil keine externen Systeme beteiligt sind.

## 10. Nicht-funktionale Anforderungen

| Thema | Anforderung |
|---|---|
| Performance | Die Kachelübersicht braucht keine zusätzliche Datenbankabfrage. Sie nutzt die bereits geladenen Rechte des Benutzers. |
| Sicherheit | Keine neuen Rechte. Kacheln sind nur Navigation. Modulseiten bleiben durch Bereichsschutz und Server-Funktionen geschützt. |
| Datenschutz | Keine Personendaten. |
| Nachvollziehbarkeit | Entfällt, weil keine Daten geschrieben werden. |
| Mehrbenutzer | Entfällt, weil nur Rechte gelesen werden. |
| Kompatibilität | Wie die übrige App: aktuelle Browser, Bildschirmbreiten ab Smartphone. |

## 11. Fehlerbehandlung und Randfälle

| Fall | Erwartetes Verhalten |
|---|---|
| Benutzer hat kein freigegebenes Modul | Leer-Hinweis auf `/`, keine Weiterleitung. |
| Benutzer hat nur Rechte für Bauobjekte (ohne Projekte) | Kachel „Projekte“ ist sichtbar. `/projekte` öffnet sich über den Bereichsschutz mit „oder“. |
| Modulseite ohne Recht aufgerufen, z. B. über ein Lesezeichen | Bereichsschutz leitet auf `/` um. |
| Recht während der Sitzung entzogen | Die Kachel bleibt bis zur nächsten Seitenladung. Beim Öffnen leitet der Bereichsschutz auf `/` um. |
| Admin | Alle zehn Kacheln sind sichtbar, auch ohne einzelne Rechte. |
| Sitzung abgelaufen | Weiterleitung zu `/anmelden` (unverändert). |

## 12. Akzeptanzkriterien

Jedes Kriterium ist prüfbar und einer Story zugeordnet.

```gherkin
Szenario: Startansicht nach dem Login (US-1)
  Angenommen ein Benutzer hat die Rechte termine.sehen und mitarbeiter.sehen
  Wenn er sich anmeldet
  Dann zeigt / die Kacheln „Weiterbildung“ und „Mitarbeiter“ und keine Weiterleitung

Szenario: Kachel öffnet das Modul (US-1)
  Angenommen der Benutzer sieht die Kachel „Mitarbeiter“
  Wenn er die Kachel anklickt
  Dann öffnet sich /personal/mitarbeiter

Szenario: Kein freigegebenes Modul (US-1)
  Angenommen ein Benutzer hat kein Recht mit sehen
  Wenn er / öffnet
  Dann sieht er den Leer-Hinweis und keine Kachel

Szenario: Admin sieht alle Module (US-1)
  Angenommen ein Benutzer ist Admin
  Wenn er / öffnet
  Dann sieht er alle zehn Kacheln

Szenario: Modulseite ohne Recht (US-1)
  Angenommen ein Benutzer hat kein Recht mitarbeiter.sehen
  Wenn er /personal/mitarbeiter öffnet
  Dann landet er auf /
```

- [ ] AK-1: Nach dem Login zeigt `/` die Kacheln, ohne Weiterleitung auf eine Modulseite.
- [ ] AK-2: Jede Kachel führt zur Route aus Abschnitt 8.1.
- [ ] AK-3: Ein Benutzer ohne `sehen`-Recht sieht keine Kachel, sondern den Leer-Hinweis.
- [ ] AK-4: Ein Admin sieht alle zehn Kacheln.
- [ ] AK-5: Eine Modulseite ohne Recht leitet auf `/` um.
- [ ] AK-6: `/personal` leitet auf `/` um.
- [ ] AK-7: Kacheln und Menüpunkte kommen aus derselben Liste (`NAVIGATION`).

## 13. Teststrategie

| Ebene | Was | Wo |
|---|---|---|
| Unit | Sichtbarkeitsregel „mindestens ein Bereich freigegeben“ als reine Funktion, mit Tests | `#[cfg(test)]` im Modul der gemeinsamen Navigation (`src/navigation.rs`) |
| Datenbank | Entfällt, weil keine Datenbankänderung. | – |
| Build | Skill `build` (Server und WASM) | Cloud-Session oder lokal, sobald Defender es zulässt |
| Manuell / UI | AK-1 bis AK-6 im Browser | Checkliste unten |

Manuelle Prüfliste:

- [ ] Als Admin: alle zehn Kacheln
- [ ] Als Benutzer mit nur `termine.sehen`: nur Weiterbildung
- [ ] Ohne Recht: Leer-Hinweis

## 14. Betrieb und Einführung

- Neue Umgebungsvariablen: Entfällt.
- Migration läuft beim Start: Entfällt.
- Datenübernahme / Initialdaten: Entfällt.
- Rückfallplan: Die Route `/` wieder auf die Weiterleitung setzen, also den Commit zurücknehmen.
- Information an die Benutzer: Nach dem Login erscheint die Modulübersicht.

## 15. Offene Fragen und Entscheidungen

| # | Frage / Entscheidung | Status | Antwort / Begründung | Datum |
|---|---|---|---|---|
| 1 | Was ist ein Modul? | entschieden | Ein Eintrag der gemeinsamen Navigationsliste, also zehn Kacheln. Weiterbildung und Projekte sind je eine Kachel. Einstellungen ist keine Kachel. | 2026-10-30 |
| 2 | Wann ist ein Modul freigegeben? | entschieden | Recht `<bereich>.sehen` für mindestens einen Bereich des Moduls. Admins haben alle Module. Keine Lizenzprüfung, weil es sie im Code nicht gibt. | 2026-10-30 |
| 3 | Was zeigt `/`, wenn keine Kachel sichtbar ist? | entschieden | Leer-Hinweis, keine Umleitung nach `/kein-zugriff`. | 2026-10-30 |
| 4 | Wohin leitet der Bereichsschutz ohne Recht? | entschieden | Auf `/`. `/personal` leitet ebenfalls auf `/`. Die Komponente `Startseite` entfällt. `Benutzer::startseite()` bleibt, weil die Reiterleiste es nutzt (`src/components/gemeinsam/gemeinsam.rs`). | 2026-10-30 |
| 5 | ID-Konflikt: F-010 ist als Platzhalter für Einkauf eingetragen. | entschieden | F-010 gehört zu dieser Startansicht. Einkauf bekommt die ID F-014. | 2026-10-30 |
| 6 | Sollen Kacheln und Menü aus derselben Liste gebaut werden? | entschieden | Ja. `Ziel` und `NAVIGATION` wandern aus `layout.rs` in `src/navigation.rs`. | 2026-10-30 |

## 16. Aufgaben

Werden in [task.md](../task.md) übernommen (T-011 bis T-014).

- [ ] Gemeinsame Navigationsliste: `Ziel` und `NAVIGATION` aus `layout.rs` nach `src/navigation.rs` verschieben, Modul in `src/lib.rs` eintragen, Layout nutzt die Liste weiter. Unit-Test für die Sichtbarkeitsregel.
- [ ] Komponente `src/components/startansicht/`: Kachelraster (DaisyUI `card`), Leer-Zustand, Ladeanzeige.
- [ ] Route und Weiterleitungen in `src/app.rs`: `/` zeigt die Startansicht, `Startseite` entfällt, `/personal` leitet auf `/`, Bereichsschutz leitet auf `/`.
- [ ] Prüfung: Skill `build` und Skill `test` laufen ohne Fehler, AK-1 bis AK-7 sind geprüft.

## 17. Änderungsverlauf

| Datum | Version | Änderung | Wer |
|---|---|---|---|
| 2026-10-30 | 0.1 | Erster Entwurf | Benutzer |
| 2026-10-30 | 0.2 | Offene Fragen entschieden, Abschnitte vervollständigt, Aufgaben abgeleitet | Claude |
| 2026-10-30 | 0.3 | Menüpunkt und Kachel „Artikel“ (F-011) dazu: elf statt zehn Module | Claude |
| 2026-10-30 | 0.4 | Lager als Gruppe wie Personal, Kachel „Lager“ entfällt: wieder zehn Module | Claude |
