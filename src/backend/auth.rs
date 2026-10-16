//! Anmeldung, Registrierung, Sitzungen und Benutzerverwaltung.
//!
//! Sitzungen laufen über ein HttpOnly-Cookie mit zufälligem Token. In der
//! Datenbank steht nur der SHA-256-Hash des Tokens, Passwörter werden mit
//! Argon2 gehasht. Was ein Benutzer darf, kommt aus seiner Rolle.

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
    use uuid::Uuid;

    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Benutzer, Bereich, PASSWORT_MIN_LAENGE};

    const COOKIE_NAME: &str = "weiterbildung_sitzung";
    const SITZUNG_TAGE: i64 = 7;

    /// Benutzer samt Rolle und Rechten; Tabellenalias `b` für `benutzer`.
    pub const BENUTZER_SPALTEN: &str = "b.id, b.name, b.email, b.aktiv, b.erstellt_am, b.rolle_id,
        r.name AS rolle_name, COALESCE(r.ist_admin, FALSE) AS ist_admin,
        COALESCE(r.rechte, '{}') AS rechte, (b.microsoft_oid IS NOT NULL) AS microsoft";

    /// Platzhalter-Hash für Konten ohne Passwort (z. B. nur Microsoft-Anmeldung).
    pub const KEIN_PASSWORT: &str = "!kein-passwort";

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

    pub fn zufall_hex(laenge: usize) -> String {
        let mut bytes = vec![0u8; laenge];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    fn token_hash(token: &str) -> String {
        hex::encode(Sha256::digest(token.as_bytes()))
    }

    /// In Produktion (HTTPS) Cookies nur verschlüsselt übertragen.
    pub fn cookie_secure() -> &'static str {
        if std::env::var("LEPTOS_ENV").is_ok_and(|e| e.eq_ignore_ascii_case("PROD")) {
            "; Secure"
        } else {
            ""
        }
    }

    /// Inhalt des `Set-Cookie`-Headers für das Sitzungs-Cookie.
    pub fn sitzungs_cookie(wert: &str, max_age: i64) -> String {
        format!("{COOKIE_NAME}={wert}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}", cookie_secure())
    }

    fn cookie_setzen(wert: &str, max_age: i64) {
        if let (Some(antwort), Ok(wert)) = (
            use_context::<leptos_axum::ResponseOptions>(),
            HeaderValue::from_str(&sitzungs_cookie(wert, max_age)),
        ) {
            antwort.append_header(SET_COOKIE, wert);
        }
    }

    /// Wert eines Cookies aus den Request-Headern.
    pub fn cookie_lesen(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(';'))
            .filter_map(|teil| teil.trim().split_once('='))
            .find(|(n, _)| *n == name)
            .map(|(_, wert)| wert.to_string())
    }

    async fn token_aus_cookie() -> Option<String> {
        let headers: HeaderMap = leptos_axum::extract().await.ok()?;
        cookie_lesen(&headers, COOKIE_NAME)
    }

    /// Legt eine Sitzung in der Datenbank an und liefert das Cookie dafür.
    pub async fn sitzung_anlegen(benutzer_id: Uuid) -> Result<String, ServerFnError> {
        let token = zufall_hex(32);
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
        Ok(sitzungs_cookie(&token, SITZUNG_TAGE * 24 * 60 * 60))
    }

    /// Legt eine neue Sitzung an und setzt das Cookie (für Server-Funktionen).
    pub async fn sitzung_starten(benutzer_id: Uuid) -> Result<(), ServerFnError> {
        let cookie = sitzung_anlegen(benutzer_id).await?;
        if let (Some(antwort), Ok(wert)) = (
            use_context::<leptos_axum::ResponseOptions>(),
            HeaderValue::from_str(&cookie),
        ) {
            antwort.append_header(SET_COOKIE, wert);
        }
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
        match token_aus_cookie().await {
            Some(token) => benutzer_zum_token(&token).await,
            None => Ok(None),
        }
    }

    /// Wie [`aktueller_benutzer`], aber für eigene Axum-Routen außerhalb von Leptos.
    pub async fn benutzer_aus_headers(headers: &HeaderMap) -> Result<Option<Benutzer>, ServerFnError> {
        match cookie_lesen(headers, COOKIE_NAME) {
            Some(token) => benutzer_zum_token(&token).await,
            None => Ok(None),
        }
    }

    async fn benutzer_zum_token(token: &str) -> Result<Option<Benutzer>, ServerFnError> {
        // Nur feste Spaltenliste eingesetzt, Werte gehen als Parameter
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {BENUTZER_SPALTEN}
             FROM sitzung s
             JOIN benutzer b ON b.id = s.benutzer_id
             LEFT JOIN rolle r ON r.id = b.rolle_id
             WHERE s.token_hash = $1 AND s.laeuft_ab > now() AND b.aktiv"
        )))
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

    /// Bricht ab, wenn der Benutzer keines der genannten Rechte hat.
    pub async fn recht_pflicht(rechte: &[(Bereich, Aktion)]) -> Result<Benutzer, ServerFnError> {
        let benutzer = benutzer_pflicht().await?;
        if rechte.iter().any(|(b, a)| benutzer.darf(*b, *a)) {
            Ok(benutzer)
        } else {
            Err(ServerFnError::new("Für diese Funktion fehlt dir die Berechtigung."))
        }
    }

    pub async fn admin_rolle_id() -> Result<Option<Uuid>, ServerFnError> {
        sqlx::query_scalar("SELECT id FROM rolle WHERE ist_admin ORDER BY erstellt_am LIMIT 1")
            .fetch_optional(pool())
            .await
            .map_err(fehler)
    }

    /// Rolle für ein neues Konto: Mit ADMIN_EMAIL wird nur diese Adresse
    /// Administrator, ohne die Variable das allererste Konto. Alle anderen
    /// starten ohne Rolle, bis ein Admin ihnen eine zuweist.
    pub async fn start_rolle(email: &str) -> Result<Option<Uuid>, ServerFnError> {
        let admin_email = std::env::var("ADMIN_EMAIL")
            .ok()
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty());
        let wird_admin = match admin_email {
            Some(admin) => admin == email,
            None => {
                let anzahl: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM benutzer")
                    .fetch_one(pool())
                    .await
                    .map_err(fehler)?;
                anzahl == 0
            }
        };
        if wird_admin { admin_rolle_id().await } else { Ok(None) }
    }

    /// Anzahl aktiver Admins ohne den angegebenen Benutzer.
    pub async fn andere_aktive_admins(ohne: Uuid) -> Result<i64, ServerFnError> {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM benutzer b JOIN rolle r ON r.id = b.rolle_id
             WHERE r.ist_admin AND b.aktiv AND b.id <> $1",
        )
        .bind(ohne)
        .fetch_one(pool())
        .await
        .map_err(fehler)
    }

    pub async fn ist_admin_rolle(rolle_id: Option<Uuid>) -> Result<bool, ServerFnError> {
        let Some(id) = rolle_id else { return Ok(false) };
        let admin: Option<bool> = sqlx::query_scalar("SELECT ist_admin FROM rolle WHERE id = $1")
            .bind(id)
            .fetch_optional(pool())
            .await
            .map_err(fehler)?;
        admin.ok_or_else(|| ServerFnError::new("Diese Rolle gibt es nicht."))
    }
}

// ------------------------------------------------------------ Anmeldung

#[server(input = Json)]
pub async fn aktueller_benutzer_laden() -> Result<Option<Benutzer>, ServerFnError> {
    server::aktueller_benutzer().await
}

/// Ob die Anmeldung mit Microsoft eingerichtet ist (für den Button).
#[server(input = Json)]
pub async fn microsoft_verfuegbar() -> Result<bool, ServerFnError> {
    Ok(crate::backend::microsoft::konfiguration().is_some())
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

    let rolle_id = server::start_rolle(&email).await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO benutzer (name, email, passwort_hash, rolle_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(name)
    .bind(email)
    .bind(server::passwort_hashen(&passwort)?)
    .bind(rolle_id)
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
    use crate::models::{Aktion, Bereich};

    server::recht_pflicht(&[(Bereich::Benutzer, Aktion::Sehen)]).await?;
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {} FROM benutzer b LEFT JOIN rolle r ON r.id = b.rolle_id ORDER BY b.name",
        server::BENUTZER_SPALTEN
    )))
    .fetch_all(pool())
    .await
    .map_err(fehler)
}

/// Legt einen Benutzer an (`id` = None) oder ändert ihn. Das Passwort ist
/// optional: leer lassen behält das bisherige bzw. erlaubt bei neuen Konten
/// nur die Anmeldung mit Microsoft.
#[server(input = Json)]
pub async fn benutzer_speichern(
    id: Option<Uuid>,
    name: String,
    email: String,
    rolle_id: Option<Uuid>,
    aktiv: bool,
    passwort: String,
) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    let aktion = if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen };
    let ich = server::recht_pflicht(&[(Bereich::Benutzer, aktion)]).await?;

    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();
    if name.is_empty() {
        return Err(ServerFnError::new("Bitte einen Namen angeben."));
    }
    if !email.contains('@') {
        return Err(ServerFnError::new("Bitte eine gültige E-Mail-Adresse angeben."));
    }
    let neues_passwort = if passwort.is_empty() {
        None
    } else {
        server::passwort_regeln(&passwort)?;
        Some(server::passwort_hashen(&passwort)?)
    };

    // Nur Admins dürfen die Admin-Rolle vergeben oder Admins ändern.
    let neue_rolle_admin = server::ist_admin_rolle(rolle_id).await?;
    if neue_rolle_admin && !ich.ist_admin {
        return Err(ServerFnError::new("Nur Administratoren dürfen die Administrator-Rolle vergeben."));
    }

    match id {
        None => {
            sqlx::query(
                "INSERT INTO benutzer (name, email, passwort_hash, rolle_id, aktiv) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(name)
            .bind(email)
            .bind(neues_passwort.unwrap_or_else(|| server::KEIN_PASSWORT.to_string()))
            .bind(rolle_id)
            .bind(aktiv)
            .execute(pool())
            .await
            .map_err(fehler)?;
        }
        Some(id) => {
            let bisher: Option<(Option<Uuid>, bool)> = sqlx::query_as(
                "SELECT b.rolle_id, COALESCE(r.ist_admin, FALSE)
                 FROM benutzer b LEFT JOIN rolle r ON r.id = b.rolle_id WHERE b.id = $1",
            )
            .bind(id)
            .fetch_optional(pool())
            .await
            .map_err(fehler)?;
            let (bisherige_rolle, war_admin) =
                bisher.ok_or_else(|| ServerFnError::new("Diesen Benutzer gibt es nicht."))?;

            if war_admin && !ich.ist_admin {
                return Err(ServerFnError::new("Administratoren können nur von Administratoren geändert werden."));
            }
            // Sich selbst nicht aussperren
            if id == ich.id && (bisherige_rolle != rolle_id || !aktiv) {
                return Err(ServerFnError::new(
                    "Du kannst weder deine eigene Rolle ändern noch dein Konto deaktivieren.",
                ));
            }
            if war_admin && (!neue_rolle_admin || !aktiv) && server::andere_aktive_admins(id).await? == 0 {
                return Err(ServerFnError::new("Es muss mindestens ein aktiver Administrator bleiben."));
            }

            let mut tx = pool().begin().await.map_err(fehler)?;
            sqlx::query("UPDATE benutzer SET name = $2, email = $3, rolle_id = $4, aktiv = $5 WHERE id = $1")
                .bind(id)
                .bind(name)
                .bind(email)
                .bind(rolle_id)
                .bind(aktiv)
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
    use crate::models::{Aktion, Bereich};

    let ich = server::recht_pflicht(&[(Bereich::Benutzer, Aktion::Loeschen)]).await?;
    if id == ich.id {
        return Err(ServerFnError::new("Du kannst dein eigenes Konto nicht löschen."));
    }
    let war_admin: bool = sqlx::query_scalar(
        "SELECT COALESCE(r.ist_admin, FALSE) FROM benutzer b LEFT JOIN rolle r ON r.id = b.rolle_id WHERE b.id = $1",
    )
    .bind(id)
    .fetch_optional(pool())
    .await
    .map_err(fehler)?
    .unwrap_or(false);
    if war_admin && !ich.ist_admin {
        return Err(ServerFnError::new("Administratoren können nur von Administratoren gelöscht werden."));
    }
    sqlx::query("DELETE FROM benutzer WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
