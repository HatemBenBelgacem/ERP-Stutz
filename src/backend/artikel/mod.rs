//! Artikel: Stammdaten mit Bezeichnung, Grössen und optionalem Bild.

#[cfg(feature = "ssr")]
mod bild;
mod model;
#[cfg(feature = "ssr")]
mod repository;
mod server_fns;

#[cfg(feature = "ssr")]
pub use bild::bild_ausliefern;
pub use model::*;
pub use server_fns::*;

/// Übersetzt die eigenen Prüfregeln (Bezeichnung, doppelte Grösse); alle anderen Fehler gehen an `db::fehler`.
#[cfg(feature = "ssr")]
fn fehler(e: sqlx::Error) -> leptos::prelude::ServerFnError {
    let meldung = e.as_database_error().and_then(|db| {
        let code = db.code();
        let constraint = db.constraint();
        match (code.as_deref(), constraint) {
            (Some("23514"), Some("artikel_bezeichnung_check")) => {
                Some("Bitte eine Bezeichnung eingeben.")
            }
            (Some("23505"), Some("kleidergroesse_artikel_id_bezeichnung_key")) => {
                Some("Eine Grösse ist doppelt angegeben.")
            }
            _ => None,
        }
    });
    match meldung {
        Some(text) => leptos::prelude::ServerFnError::new(text),
        None => crate::backend::db::fehler(e),
    }
}
