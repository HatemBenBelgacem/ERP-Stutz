//! Artikel: Stammdaten mit Bezeichnung, Grösse und optionalem Bild.

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

/// Übersetzt die Prüfregel der Bezeichnung; alle anderen Fehler gehen an `db::fehler`.
#[cfg(feature = "ssr")]
fn fehler(e: sqlx::Error) -> leptos::prelude::ServerFnError {
    let bezeichnung_leer = e.as_database_error().is_some_and(|db| {
        db.code().as_deref() == Some("23514")
            && db.constraint() == Some("artikel_bezeichnung_check")
    });
    if bezeichnung_leer {
        return leptos::prelude::ServerFnError::new("Bitte eine Bezeichnung eingeben.");
    }
    crate::backend::db::fehler(e)
}
