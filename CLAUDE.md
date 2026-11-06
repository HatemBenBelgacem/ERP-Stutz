# CLAUDE.md – ERP-Stutz

ERP-System der Stutz AG. Fullstack-Webanwendung in Rust mit **Leptos 0.8** (SSR + Hydration),
**Axum 0.8**, **sqlx 0.9 / PostgreSQL**, **Tailwind CSS v4 + DaisyUI v5**.

## Sprache und Stil

- Mit dem Benutzer **Deutsch** sprechen, einfach und klar.
- Code-Bezeichner, Kommentare, Commit-Nachrichten, Tabellen und Testnamen auf **Deutsch**
  (z. B. `bild_ausliefern`, `geschaeftsbereich`, `db_termin_ende_vor_beginn_wird_abgelehnt`).
- **Keine Umlaute in Modul- und Dateinamen** (`loeschen`, `groesse`); in Texten für die Oberfläche schon.
- Schweizer Schreibweise: „ss“ statt „ß“ (`Grösse`, `heisst`).
- Commit-Nachrichten: `F-XXX <was>` für Features (z. B. `F-011 Bild-Route für Artikel`),
  sonst kurz beschreibend.

## Skills (unter `.claude/skills/`)

Diese Skills zuerst verwenden, statt eigene Abläufe zu erfinden:

| Skill | Wann |
|---|---|
| `build` | Nach jeder Code-Änderung: kompiliert es? |
| `test` | Vor jedem Commit: Formatierung, Clippy, Tests |
| `migration` | Jede Änderung am Datenbankschema |
| `feature-spec` | Feature planen, Spezifikation prüfen, in Aufgaben zerlegen |

## Wichtige Befehle

Immer ein Feature angeben – ohne `ssr` oder `hydrate` kompiliert die Crate nicht sinnvoll.

```bash
cargo check --features ssr                                              # Server
cargo check --lib --features hydrate --target wasm32-unknown-unknown   # Browser (WASM)
cargo clippy --features ssr
cargo test --features ssr --lib -- --skip db_                           # ohne Datenbank
cargo leptos watch                                                      # lokal starten (Port 3000)
```

- Für `cargo check`/Build ist **keine Datenbank** nötig (`sqlx::query` zur Laufzeit, keine `query!`-Makros).
- Zum Starten und für `db_`-Tests braucht es `DATABASE_URL` (siehe `.env.example`).
- `cargo-leptos` in Version **0.3.10** (wie im `Dockerfile`).
- `wasm-bindgen` ist auf `=0.2.129` festgelegt – nicht ohne Grund ändern.

## Aufbau

```text
src/
├── main.rs         Axum-Router, Sonderrouten (Microsoft-OAuth, PDF, Artikelbild)
├── lib.rs          Module, hydrate()
├── app.rs          Routen, Sitzung (use_sitzung), Bereichsschutz
├── models.rs       Bereich, Aktion, Benutzer, Rechte (alte Struktur, wird abgelöst)
├── navigation.rs   Menü- und Kachelliste
├── backend/        Server-Funktionen, SQL (db.rs, auth.rs, artikel/, …)
└── components/     Leptos-Komponenten pro Modell
migrations/         SQL-Migrationen, laufen beim Start automatisch
docs/               system-design.md (Ziel-Architektur), data-model.md (Datenmodell)
features/           Roadmap (index.md), Spezifikationen F-XXX, Vorlage _vorlage.md
task.md             Aufgabenliste T-XXX
```

**Ziel-Architektur** (siehe `docs/system-design.md`): pro Modell
`backend/<modell>/{mod,model,server_fns,repository}.rs` und
`components/<modell>/{liste_,formular_,neu_,bearbeiten_,loeschen_}<modell>.rs`.
`backend/artikel/` ist schon so aufgebaut – als Vorbild nehmen. Ältere Modelle liegen noch in
`models.rs` und `backend/api.rs`; bei neuem Code die neue Struktur verwenden.

## Regeln

- **Bestehende Migrationen nie ändern** – immer eine neue Datei (Skill `migration`).
  Danach `docs/data-model.md` nachführen.
- **Rechte**: jede Aktion braucht ein Recht `<bereich>.<aktion>` (`Bereich`/`Aktion` in `src/models.rs`).
  Server-Funktionen prüfen Anmeldung und Recht selbst; Seiten in `app.rs` mit `Bereichsschutz` schützen.
- Neue Datenbank-Fehler (UNIQUE, FK) in `backend/db.rs::fehler` in verständliche Meldungen übersetzen.
- Code ist noch nicht durchgehend formatiert (T-005) und hat Clippy-Warnungen (T-006):
  nur **geänderte Dateien** mit `rustfmt --edition 2024` formatieren, **keine neuen Warnungen**.
  Nicht `cargo fmt` auf das ganze Projekt anwenden.
- Tests nie deaktivieren oder abschwächen. Rechte mit drei Fällen testen: Admin, mit Recht, ohne Recht.
- „Fertig“ heisst: `build` und `test` grün, Akzeptanzkriterien geprüft, `task.md` nachgeführt.

## Cloud-Umgebung (Claude Code im Web)

- Rust, Node 22, `psql` und Docker sind installiert. Das WASM-Ziel ist evtl. nicht installiert:
  `rustup target add wasm32-unknown-unknown`.
- Es läuft standardmässig **kein PostgreSQL-Server**. `db_`-Tests überspringen und das melden,
  oder eine Datenbank per Docker starten (siehe Skill `test`).
- Der Container ist flüchtig: Arbeit committen und auf den vorgegebenen Branch pushen.
- Deployment: Docker-Image (`Dockerfile`) auf Railway, Port über `PORT`.
