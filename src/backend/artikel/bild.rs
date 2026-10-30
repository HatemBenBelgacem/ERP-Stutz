//! Bild eines Artikels als Axum-Route (`GET /artikel/{id}/bild`).

use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::repository;
use crate::backend::auth::server::benutzer_aus_headers;
use crate::backend::db::pool;
use crate::models::{Aktion, Bereich};

/// Liefert das Bild nur an angemeldete Benutzer mit dem Recht „Sehen“ für Lager.
pub async fn bild_ausliefern(headers: HeaderMap, Path(id): Path<Uuid>) -> Response {
    let benutzer = match benutzer_aus_headers(&headers).await {
        Ok(Some(b)) => b,
        Ok(None) => return StatusCode::FORBIDDEN.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if !benutzer.darf(Bereich::Lager, Aktion::Sehen) {
        return StatusCode::FORBIDDEN.into_response();
    }

    match repository::bild(pool(), id).await {
        Ok(Some((name, daten))) => {
            let typ = if name.to_ascii_lowercase().ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };
            ([(CONTENT_TYPE, typ)], daten).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            leptos::logging::error!("Artikelbild konnte nicht geladen werden: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
