//! Datenbankverbindung (nur auf dem Server).

use std::sync::OnceLock;

use leptos::prelude::ServerFnError;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

static POOL: OnceLock<PgPool> = OnceLock::new();

/// Verbindet sich mit `DATABASE_URL` und führt ausstehende Migrationen aus.
pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL ist nicht gesetzt (siehe .env.example)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    POOL.set(pool).map_err(|_| "Datenbank wurde bereits initialisiert")?;
    Ok(())
}

pub fn pool() -> &'static PgPool {
    POOL.get().expect("db::init() wurde nicht aufgerufen")
}

/// Übersetzt typische Postgres-Fehler in verständliche Meldungen.
pub fn fehler(e: sqlx::Error) -> ServerFnError {
    if let Some(db) = e.as_database_error() {
        match db.code().as_deref() {
            Some("23505")
                if matches!(db.constraint(), Some("mitarbeiter_email_key" | "benutzer_email_key")) =>
            {
                return ServerFnError::new("Diese E-Mail-Adresse ist bereits vergeben.");
            }
            Some("23503") if db.constraint() == Some("termin_kurs_id_fkey") => {
                return ServerFnError::new(
                    "Der Kurs kann nicht gelöscht werden, solange noch Termine existieren.",
                );
            }
            Some("23505") if db.constraint() == Some("geschaeftsbereich_beschreibung_key") => {
                return ServerFnError::new("Einen Geschäftsbereich mit dieser Beschreibung gibt es bereits.");
            }
            Some("23503") if db.constraint() == Some("benutzer_rolle_id_fkey") => {
                return ServerFnError::new(
                    "Die Rolle kann nicht gelöscht werden, solange ihr noch Benutzer zugewiesen sind.",
                );
            }
            Some("23505") if db.constraint() == Some("rolle_name_key") => {
                return ServerFnError::new("Eine Rolle mit diesem Namen gibt es bereits.");
            }
            Some("23505") if db.constraint() == Some("projekt_projektnummer_key") => {
                return ServerFnError::new("Ein Projekt mit dieser Projektnummer gibt es bereits.");
            }
            Some("23503") if db.constraint() == Some("buchung_projekt_id_fkey") => {
                return ServerFnError::new(
                    "Das Projekt kann nicht gelöscht werden, solange noch Stunden darauf gebucht sind.",
                );
            }
            Some("23503") if db.constraint() == Some("projekt_bauobjekt_id_fkey") => {
                return ServerFnError::new(
                    "Das Bauobjekt kann nicht gelöscht werden, solange noch Projekte dazu existieren.",
                );
            }
            Some("23514") => {
                return ServerFnError::new("Die Eingaben verletzen eine Prüfregel der Datenbank.");
            }
            _ => {}
        }
    }
    leptos::logging::error!("Datenbankfehler: {e}");
    ServerFnError::new("Datenbankfehler – Details stehen im Server-Log.")
}
