//! Anmeldung, Registrierung, Sitzungen und Benutzerverwaltung.
//!
//! Sitzungen laufen über ein HttpOnly-Cookie mit zufälligem Token. In der
//! Datenbank steht nur der SHA-256-Hash des Tokens, Passwörter werden mit
//! Argon2 gehasht.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

use crate::models::Benutzer;

#[cfg(feature = "ssr")]
pub mod server {
    use argon2::password_hash::rand_core::{OsRng, RngCore};
    use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
    use argon2::Argon2;
    use axum::http::header::{COOKIE, SET_COOKIE};
    use axum::http::{HeaderMap, HeaderValue};
    use leptos::prelude::*;
    use sha2::{Digest, Sha256};

    use crate::backend::db::{fehler, pool};
    use crate::models::{Benutzer, Bereich, PASSWORT_MIN_LAENGE};

    const COOKIE_NAME: &str = "weiterbildung_sitzung";
    const SITZUNG_TAGE: i64 = 7;

    pub fn passwort_hashen(passwort: &str) -> Result<String, ServerFnError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(passwort.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| ServerFnError::new("Passwort konnte nicht verarbeitet werden."))
    }

    pub fn passwort_pruefen(passwort: &str, hash: &str) -> bool {
        PasswordHash::new(hash)
            .map(|h| Argon2::default().verify_password(passwort.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    }

    pub fn passwort_regeln(passwort: &str) -> Result<(), ServerFnError> {
        if passwort.chars().count() < PASSWORT_MIN_LAENGE {
            return Err(ServerFnError::new(format!(
                "Das Passwort muss mindestens {PASSWORT_MIN_LAENGE} Zeichen lang sein."
            )));
        }
        Ok(())
    }

    fn token_hash(token: &str) -> String {
        hex::encode(Sha256::digest(token.as_bytes()))
    }

    fn cookie_setzen(wert: &str, max_age: i64) {
        // In Produktion (HTTPS) das Cookie nur verschlüsselt übertragen
        let secure = if std::env::var("LEPTOS_ENV").is_ok_and(|e| e.eq_ignore_ascii_case("PROD")) {
            "; Secure"
        } else {
            ""
        };
        let cookie = format!("{COOKIE_NAME}={wert}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}");
        if let (Some(antwort), Ok(wert)) = (
            use_context::<leptos_axum::ResponseOptions>(),
            HeaderValue::from_str(&cookie),
        ) {
            antwort.append_header(SET_COOKIE, wert);
        }
    }

    async fn token_aus_cookie() -> Option<String> {
        let headers: HeaderMap = leptos_axum::extract().await.ok()?;
        headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(';'))
            .filter_map(|teil| teil.trim().split_once('='))
            .find(|(name, _)| *name == COOKIE_NAME)
            .map(|(_, wert)| wert.to_string())
    }

    /// Legt eine neue Sitzung an und setzt das Cookie.
    pub async fn sitzung_starten(benutzer_id: uuid::Uuid) -> Result<(), ServerFnError> {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(bytes);

        sqlx::query("DELETE FROM sitzung WHERE laeuft_ab < now()")
            .execute(pool())
            .await
            .map_err(fehler)?;
        sqlx::query(
            "INSERT INTO sitzung (token_hash, benutzer_id, laeuft_ab)
             VALUES ($1, $2, now() + make_interval(days => $3))",
        )
        .bind(token_hash(&token))
        .bind(benutzer_id)
        .bind(SITZUNG_TAGE as i32)
        .execute(pool())
        .await
        .map_err(fehler)?;

        cookie_setzen(&token, SITZUNG_TAGE * 24 * 60 * 60);
        Ok(())
    }

    pub async fn sitzung_beenden() -> Result<(), ServerFnError> {
        if let Some(token) = token_aus_cookie().await {
            sqlx::query("DELETE FROM sitzung WHERE token_hash = $1")
                .bind(token_hash(&token))
                .execute(pool())
                .await
                .map_err(fehler)?;
        }
        cookie_setzen("", 0);
        Ok(())
    }

    /// Der angemeldete, aktive Benutzer – oder `None`.
    pub async fn aktueller_benutzer() -> Result<Option<Benutzer>, ServerFnError> {
        let Some(token) = token_aus_cookie().await else {
            return Ok(None);
        };
        sqlx::query_as(
            "SELECT b.id, b.name, b.email, b.rolle, b.aktiv, b.erstellt_am, b.berechtigungen
             FROM sitzung s JOIN benutzer b ON b.id = s.benutzer_id
             WHERE s.token_hash = $1 AND s.laeuft_ab > now() AND b.aktiv",
        )
        .bind(token_hash(&token))
        .fetch_optional(pool())
        .await
        .map_err(fehler)
    }

    /// Bricht ab, wenn niemand angemeldet ist.
    pub async fn benutzer_pflicht() -> Result<Benutzer, ServerFnError> {
        aktueller_benutzer()
            .await?
            .ok_or_else(|| ServerFnError::new("Bitte zuerst anmelden."))
    }

    /// Bricht ab, wenn der Benutzer keinen der genannten Bereiche sehen darf.
    pub async fn bereich_pflicht(bereiche: &[Bereich]) -> Result<Benutzer, ServerFnError> {
        let benutzer = benutzer_pflicht().await?;
        if bereiche.iter().any(|b| benutzer.darf(*b)) {
            Ok(benutzer)
        } else {
            Err(ServerFnError::new("Für diesen Bereich fehlt dir die Berechtigung."))
        }
    }

    /// Bricht ab, wenn der angemeldete Benutzer kein Admin ist.
    pub async fn admin_pflicht() -> Result<Benutzer, ServerFnError> {
        let benutzer = benutzer_pflicht().await?;
        if benutzer.ist_admin() {
            Ok(benutzer)
        } else {
            Err(ServerFnError::new("Dafür sind Administratorrechte nötig."))
        }
    }
}

// ------------------------------------------------------------ Anmeldung

#[server(input = Json)]
pub async fn aktueller_benutzer_laden() -> Result<Option<Benutzer>, ServerFnError> {
    server::aktueller_benutzer().await
}

#[server(input = Json)]
pub async fn anmelden(email: String, passwort: String) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};

    let zeile: Option<(Uuid, String, bool)> =
        sqlx::query_as("SELECT id, passwort_hash, aktiv FROM benutzer WHERE email = $1")
            .bind(email.trim().to_lowercase())
            .fetch_optional(pool())
            .await
            .map_err(fehler)?;

    // Gleiche Meldung für unbekannte E-Mail und falsches Passwort.
    let falsch = || ServerFnError::new("E-Mail oder Passwort ist falsch.");
    let (id, hash, aktiv) = zeile.ok_or_else(falsch)?;
    if !server::passwort_pruefen(&passwort, &hash) {
        return Err(falsch());
    }
    if !aktiv {
        return Err(ServerFnError::new("Dieses Konto ist deaktiviert."));
    }
    server::sitzung_starten(id).await
}

#[server(input = Json)]
pub async fn registrieren(
    name: String,
    email: String,
    passwort: String,
    passwort_wiederholt: String,
) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};
    use crate::models::{ROLLE_ADMIN, ROLLE_BENUTZER};

    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.is_empty() {
        return Err(ServerFnError::new("Bitte einen Namen angeben."));
    }
    if !email.contains('@') {
        return Err(ServerFnError::new("Bitte eine gültige E-Mail-Adresse angeben."));
    }
    server::passwort_regeln(&passwort)?;
    if passwort != passwort_wiederholt {
        return Err(ServerFnError::new("Die Passwörter stimmen nicht überein."));
    }

    // Mit ADMIN_EMAIL wird nur diese Adresse Administrator (für öffentliche
    // Installationen). Ohne die Variable wird das erste Konto Administrator.
    let admin_email = std::env::var("ADMIN_EMAIL")
        .ok()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty());
    let rolle = match admin_email {
        Some(admin) if admin == email => ROLLE_ADMIN,
        Some(_) => ROLLE_BENUTZER,
        None => {
            let anzahl: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM benutzer")
                .fetch_one(pool())
                .await
                .map_err(fehler)?;
            if anzahl == 0 { ROLLE_ADMIN } else { ROLLE_BENUTZER }
        }
    };

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO benutzer (name, email, passwort_hash, rolle) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(name)
    .bind(email)
    .bind(server::passwort_hashen(&passwort)?)
    .bind(rolle)
    .fetch_one(pool())
    .await
    .map_err(fehler)?;

    server::sitzung_starten(id).await
}

#[server(input = Json)]
pub async fn abmelden() -> Result<(), ServerFnError> {
    server::sitzung_beenden().await
}

// --------------------------------------------------- Benutzerverwaltung

#[server(input = Json)]
pub async fn benutzer_laden() -> Result<Vec<Benutzer>, ServerFnError> {
    use crate::backend::db::{fehler, pool};
    server::admin_pflicht().await?;
    sqlx::query_as("SELECT id, name, email, rolle, aktiv, erstellt_am, berechtigungen FROM benutzer ORDER BY name")
        .fetch_all(pool())
        .await
        .map_err(fehler)
}

/// Legt einen Benutzer an (`id` = None, Passwort Pflicht) oder ändert ihn
/// (Passwort optional – leer lassen behält das bisherige).
#[server(input = Json)]
pub async fn benutzer_speichern(
    id: Option<Uuid>,
    name: String,
    email: String,
    rolle: String,
    aktiv: bool,
    passwort: String,
    berechtigungen: Vec<String>,
) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};
    use crate::models::{ROLLE_ADMIN, ROLLE_BENUTZER};

    let ich = server::admin_pflicht().await?;
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.is_empty() {
        return Err(ServerFnError::new("Bitte einen Namen angeben."));
    }
    if !email.contains('@') {
        return Err(ServerFnError::new("Bitte eine gültige E-Mail-Adresse angeben."));
    }
    if rolle != ROLLE_ADMIN && rolle != ROLLE_BENUTZER {
        return Err(ServerFnError::new("Unbekannte Rolle."));
    }
    // Nur bekannte Bereiche übernehmen, jeden höchstens einmal
    let mut berechtigungen: Vec<String> = berechtigungen
        .iter()
        .filter_map(|b| crate::models::Bereich::aus_schluessel(b))
        .map(|b| b.schluessel().to_string())
        .collect();
    berechtigungen.sort();
    berechtigungen.dedup();

    let neues_passwort = if passwort.is_empty() {
        None
    } else {
        server::passwort_regeln(&passwort)?;
        Some(server::passwort_hashen(&passwort)?)
    };

    match id {
        None => {
            let hash = neues_passwort
                .ok_or_else(|| ServerFnError::new("Für neue Benutzer ist ein Passwort nötig."))?;
            sqlx::query(
                "INSERT INTO benutzer (name, email, passwort_hash, rolle, aktiv, berechtigungen)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(name)
            .bind(email)
            .bind(hash)
            .bind(rolle)
            .bind(aktiv)
            .bind(&berechtigungen)
            .execute(pool())
            .await
            .map_err(fehler)?;
        }
        Some(id) => {
            // Sich selbst nicht aussperren
            if id == ich.id && (rolle != ROLLE_ADMIN || !aktiv) {
                return Err(ServerFnError::new(
                    "Du kannst dir selbst weder die Admin-Rolle entziehen noch dein Konto deaktivieren.",
                ));
            }
            let mut tx = pool().begin().await.map_err(fehler)?;
            sqlx::query(
                "UPDATE benutzer SET name = $2, email = $3, rolle = $4, aktiv = $5, berechtigungen = $6
                 WHERE id = $1",
            )
                .bind(id)
                .bind(name)
                .bind(email)
                .bind(rolle)
                .bind(aktiv)
                .bind(&berechtigungen)
                .execute(&mut *tx)
                .await
                .map_err(fehler)?;
            if let Some(hash) = neues_passwort {
                sqlx::query("UPDATE benutzer SET passwort_hash = $2 WHERE id = $1")
                    .bind(id)
                    .bind(hash)
                    .execute(&mut *tx)
                    .await
                    .map_err(fehler)?;
            }
            if !aktiv {
                // Deaktivierte Konten sofort abmelden
                sqlx::query("DELETE FROM sitzung WHERE benutzer_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await
                    .map_err(fehler)?;
            }
            tx.commit().await.map_err(fehler)?;
        }
    }
    Ok(())
}

#[server(input = Json)]
pub async fn benutzer_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};
    let ich = server::admin_pflicht().await?;
    if id == ich.id {
        return Err(ServerFnError::new("Du kannst dein eigenes Konto nicht löschen."));
    }
    sqlx::query("DELETE FROM benutzer WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
