//! Anmeldung mit einem Microsoft-Konto (Microsoft Entra ID, OAuth 2.0 mit PKCE).
//!
//! Einrichtung über Umgebungsvariablen:
//! - `MICROSOFT_CLIENT_ID`, `MICROSOFT_CLIENT_SECRET`: aus der App-Registrierung
//! - `MICROSOFT_TENANT`: Mandanten-ID, oder `common` / `organizations` / `consumers`
//!   (Standard: `common`)
//! - `APP_URL`: öffentliche Adresse der App, z. B. `https://….up.railway.app`
//!   (Standard: `http://localhost:3000`). Als Redirect-URI in Azure eintragen:
//!   `<APP_URL>/auth/microsoft/callback`

/// Eingerichtete Microsoft-Anmeldung, `None` wenn die Variablen fehlen.
#[cfg(feature = "ssr")]
pub struct Konfiguration {
    pub client_id: String,
    pub client_secret: String,
    pub tenant: String,
    pub redirect_uri: String,
}

#[cfg(feature = "ssr")]
pub fn konfiguration() -> Option<Konfiguration> {
    let var = |name: &str| std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    let app_url = var("APP_URL").unwrap_or_else(|| "http://localhost:3000".to_string());
    Some(Konfiguration {
        client_id: var("MICROSOFT_CLIENT_ID")?,
        client_secret: var("MICROSOFT_CLIENT_SECRET")?,
        tenant: var("MICROSOFT_TENANT").unwrap_or_else(|| "common".to_string()),
        redirect_uri: format!("{}/auth/microsoft/callback", app_url.trim_end_matches('/')),
    })
}

#[cfg(feature = "ssr")]
pub use handler::{rueckruf, start};

#[cfg(feature = "ssr")]
mod handler {
    use axum::extract::Query;
    use axum::http::header::{LOCATION, SET_COOKIE};
    use axum::http::{HeaderMap, StatusCode};
    use axum::response::{IntoResponse, Response};
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde::Deserialize;
    use sha2::{Digest, Sha256};
    use uuid::Uuid;

    use super::{Konfiguration, konfiguration};
    use crate::backend::auth::server::{
        KEIN_PASSWORT, cookie_lesen, cookie_secure, sitzung_anlegen, start_rolle, zufall_hex,
    };
    use crate::backend::db::pool;

    const COOKIE_NAME: &str = "weiterbildung_ms";
    const COOKIE_PFAD: &str = "/auth/microsoft";

    fn umleiten(ziel: &str, cookies: Vec<String>) -> Response {
        let mut antwort = (StatusCode::FOUND, [(LOCATION, ziel.to_string())]).into_response();
        for cookie in cookies {
            if let Ok(wert) = cookie.parse() {
                antwort.headers_mut().append(SET_COOKIE, wert);
            }
        }
        antwort
    }

    fn fehlerseite(meldung: &str) -> Response {
        let ziel = reqwest::Url::parse_with_params("http://x/anmelden", &[("fehler", meldung)])
            .map(|u| format!("/anmelden?{}", u.query().unwrap_or_default()))
            .unwrap_or_else(|_| "/anmelden".to_string());
        umleiten(&ziel, vec![ms_cookie("", 0)])
    }

    fn ms_cookie(wert: &str, max_age: i64) -> String {
        format!("{COOKIE_NAME}={wert}; Path={COOKIE_PFAD}; HttpOnly; SameSite=Lax; Max-Age={max_age}{}", cookie_secure())
    }

    /// Schritt 1: zu Microsoft weiterleiten. `state` und PKCE-Verifier
    /// liegen bis zur Rückkehr in einem kurzlebigen Cookie.
    pub async fn start() -> Response {
        let Some(k) = konfiguration() else {
            return fehlerseite("Die Anmeldung mit Microsoft ist nicht eingerichtet.");
        };
        let state = zufall_hex(16);
        let verifier = zufall_hex(32);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

        let url = reqwest::Url::parse_with_params(
            &format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", k.tenant),
            &[
                ("client_id", k.client_id.as_str()),
                ("response_type", "code"),
                ("redirect_uri", k.redirect_uri.as_str()),
                ("response_mode", "query"),
                ("scope", "openid profile email"),
                ("state", state.as_str()),
                ("code_challenge", challenge.as_str()),
                ("code_challenge_method", "S256"),
                ("prompt", "select_account"),
            ],
        );
        match url {
            Ok(url) => umleiten(url.as_str(), vec![ms_cookie(&format!("{state}.{verifier}"), 600)]),
            Err(_) => fehlerseite("Die Microsoft-Anmeldung ist falsch eingerichtet."),
        }
    }

    #[derive(Deserialize)]
    pub struct Rueckruf {
        code: Option<String>,
        state: Option<String>,
        error: Option<String>,
    }

    /// Schritt 2: Microsoft kommt mit einem Code zurück.
    pub async fn rueckruf(headers: HeaderMap, Query(rueckruf): Query<Rueckruf>) -> Response {
        match anmelden(&headers, rueckruf).await {
            Ok(sitzung) => umleiten("/", vec![sitzung, ms_cookie("", 0)]),
            Err(meldung) => fehlerseite(&meldung),
        }
    }

    #[derive(Deserialize)]
    struct TokenAntwort {
        id_token: String,
    }

    /// Inhalt des ID-Tokens, den wir brauchen.
    #[derive(Deserialize)]
    struct Angaben {
        aud: String,
        exp: i64,
        oid: String,
        name: Option<String>,
        email: Option<String>,
        preferred_username: Option<String>,
    }

    async fn anmelden(headers: &HeaderMap, rueckruf: Rueckruf) -> Result<String, String> {
        let k = konfiguration().ok_or("Die Anmeldung mit Microsoft ist nicht eingerichtet.")?;
        if let Some(fehler) = rueckruf.error {
            return Err(if fehler == "access_denied" {
                "Die Anmeldung bei Microsoft wurde abgebrochen.".into()
            } else {
                "Microsoft hat die Anmeldung abgelehnt.".into()
            });
        }

        // state prüfen (Schutz gegen untergeschobene Anmeldungen)
        let gespeichert = cookie_lesen(headers, COOKIE_NAME).ok_or("Die Anmeldung ist abgelaufen. Bitte erneut versuchen.")?;
        let (state, verifier) = gespeichert.split_once('.').ok_or("Ungültige Anmeldung.")?;
        if rueckruf.state.as_deref() != Some(state) {
            return Err("Ungültige Anmeldung. Bitte erneut versuchen.".into());
        }
        let code = rueckruf.code.ok_or("Microsoft hat keinen Anmeldecode geliefert.")?;

        let angaben = token_holen(&k, &code, verifier).await?;
        let email = angaben
            .email
            .or(angaben.preferred_username)
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains('@'))
            .ok_or("Dein Microsoft-Konto liefert keine E-Mail-Adresse.")?;
        let name = angaben.name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| email.clone());

        let id = benutzer_finden_oder_anlegen(&k, &angaben.oid, &email, &name).await?;
        sitzung_anlegen(id).await.map_err(|_| "Die Sitzung konnte nicht angelegt werden.".to_string())
    }

    async fn token_holen(k: &Konfiguration, code: &str, verifier: &str) -> Result<Angaben, String> {
        let antwort = reqwest::Client::new()
            .post(format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", k.tenant))
            .form(&[
                ("client_id", k.client_id.as_str()),
                ("client_secret", k.client_secret.as_str()),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", k.redirect_uri.as_str()),
                ("code_verifier", verifier),
            ])
            .send()
            .await
            .map_err(|_| "Microsoft ist gerade nicht erreichbar.".to_string())?;
        if !antwort.status().is_success() {
            leptos::logging::error!("Microsoft-Token abgelehnt: {}", antwort.text().await.unwrap_or_default());
            return Err("Microsoft hat die Anmeldung nicht bestätigt.".into());
        }
        let token: TokenAntwort = antwort.json().await.map_err(|_| "Unerwartete Antwort von Microsoft.".to_string())?;

        // Das ID-Token kommt direkt vom Token-Endpunkt über TLS; damit genügt
        // es nach OpenID Connect, Empfänger und Ablauf zu prüfen.
        let inhalt = token.id_token.split('.').nth(1).ok_or("Ungültiges Token von Microsoft.")?;
        let bytes = URL_SAFE_NO_PAD.decode(inhalt).map_err(|_| "Ungültiges Token von Microsoft.".to_string())?;
        let angaben: Angaben = serde_json::from_slice(&bytes).map_err(|_| "Ungültiges Token von Microsoft.".to_string())?;
        if angaben.aud != k.client_id || angaben.exp < chrono::Utc::now().timestamp() {
            return Err("Das Token von Microsoft ist ungültig.".into());
        }
        Ok(angaben)
    }

    /// Findet das Konto zur Microsoft-ID; sonst wird es angelegt oder – nur bei
    /// einem festen Mandanten – ein bestehendes Konto mit gleicher E-Mail verknüpft.
    async fn benutzer_finden_oder_anlegen(k: &Konfiguration, oid: &str, email: &str, name: &str) -> Result<Uuid, String> {
        let db_fehler = |_| "Datenbankfehler bei der Anmeldung.".to_string();

        let verknuepft: Option<(Uuid, bool)> = sqlx::query_as("SELECT id, aktiv FROM benutzer WHERE microsoft_oid = $1")
            .bind(oid)
            .fetch_optional(pool())
            .await
            .map_err(db_fehler)?;
        if let Some((id, aktiv)) = verknuepft {
            return if aktiv { Ok(id) } else { Err("Dieses Konto ist deaktiviert.".into()) };
        }

        let gleiche_email: Option<(Uuid, bool)> = sqlx::query_as("SELECT id, aktiv FROM benutzer WHERE email = $1")
            .bind(email)
            .fetch_optional(pool())
            .await
            .map_err(db_fehler)?;
        if let Some((id, aktiv)) = gleiche_email {
            // Bei „common“ & Co. könnte jeder Mandant beliebige E-Mail-Adressen
            // ausstellen – dort verknüpfen wir nicht automatisch.
            let fester_mandant = !matches!(k.tenant.as_str(), "common" | "organizations" | "consumers");
            if !fester_mandant {
                return Err("Für diese E-Mail gibt es schon ein Konto. Bitte melde dich mit Passwort an.".into());
            }
            if !aktiv {
                return Err("Dieses Konto ist deaktiviert.".into());
            }
            sqlx::query("UPDATE benutzer SET microsoft_oid = $2 WHERE id = $1")
                .bind(id)
                .bind(oid)
                .execute(pool())
                .await
                .map_err(db_fehler)?;
            return Ok(id);
        }

        let rolle_id = start_rolle(email).await.map_err(|_| "Datenbankfehler bei der Anmeldung.".to_string())?;
        sqlx::query_scalar(
            "INSERT INTO benutzer (name, email, passwort_hash, rolle_id, microsoft_oid)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(name)
        .bind(email)
        .bind(KEIN_PASSWORT)
        .bind(rolle_id)
        .bind(oid)
        .fetch_one(pool())
        .await
        .map_err(db_fehler)
    }
}
