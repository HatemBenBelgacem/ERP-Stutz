---
name: feature-spec
description: Neues Feature für ERP-Stutz planen – Spezifikation nach features/_vorlage.md anlegen, in die Roadmap features/index.md eintragen und Aufgaben in task.md ableiten. Verwenden, wenn ein Feature geplant, eine Spezifikation geprüft oder eine Spezifikation in Aufgaben zerlegt werden soll.
---

# Feature-Spezifikation

Der Benutzer schreibt Spezifikationen selbst. Hilf beim Gerüst, prüfe auf Lücken und
leite Aufgaben ab, aber erfinde keine fachlichen Entscheidungen: Unklares als offene Frage
in Abschnitt 15 der Spezifikation eintragen.

## Neues Feature anlegen

1. Nächste freie ID in `features/index.md` suchen (`F-XXX`).
2. `features/_vorlage.md` nach `features/F-XXX-<kurzname>.md` kopieren, Kopf ausfüllen, Status „Entwurf“.
3. Zeile in `features/index.md` unter „Geplant“ ergänzen und auf die Datei verlinken.

## Spezifikation prüfen

Abschnitt für Abschnitt gegen die Vorlage prüfen und dem Benutzer eine Liste geben:

- Fehlende oder leere Abschnitte (ohne „Entfällt, weil …“)
- Akzeptanzkriterien, die nicht prüfbar sind oder keiner Story zugeordnet sind
- Rechte: Ist für jede Aktion ein Recht `<bereich>.<aktion>` genannt? Passt es zu `Bereich`/`Aktion` in `src/models.rs`?
- Datenmodell: Widersprüche zu `docs/data-model.md`, fehlendes `ON DELETE`, fehlende Datenübernahme
- Randfälle: Löschen bei Verwendung, UNIQUE-Konflikte, leere Listen, abgelaufene Sitzung

## In Aufgaben zerlegen

Aus Abschnitt 16 der Spezifikation Zeilen in `task.md` unter „Offen“ anlegen:
fortlaufende ID `T-XXX`, Feature-ID, Priorität, Abhängigkeiten.
Typische Reihenfolge: Migration (Skill `migration`) → Modelle → Server-Funktionen → UI/Route → Tests (Skill `test`) → Doku.
Status in `features/index.md` auf „✅ Bereit“ setzen, wenn keine offenen Fragen mehr bestehen.
