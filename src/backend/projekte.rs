//! Bauobjekte und ihre Projekte (Bauobjekt 1:N Projekt). Bauherr und
//! Architekt sind Adressen. Auf Projekte werden Stunden gebucht
//! (Projekt 1:N Buchung).

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

use crate::models::{Bauobjekt, BauobjektEingabe, Buchung, Projekt, ProjektEingabe};

#[cfg(feature = "ssr")]
mod server {
    use uuid::Uuid;

    use crate::models::Adresse;

    /// Spalten einer Adresse aus einem LEFT JOIN (alle leer, wenn keine verknüpft ist).
    pub type AdressSpalten = (Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>);

    pub fn adresse((id, bezeichnung, strasse, plz, ort): AdressSpalten) -> Option<Adresse> {
        id.map(|id| Adresse {
            id,
            bezeichnung: bezeichnung.unwrap_or_default(),
            strasse: strasse.unwrap_or_default(),
            plz: plz.unwrap_or_default(),
            ort: ort.unwrap_or_default(),
        })
    }

    pub fn pflichttext(wert: &str, feld: &str) -> Result<String, leptos::prelude::ServerFnError> {
        let wert = wert.trim();
        if wert.is_empty() {
            return Err(leptos::prelude::ServerFnError::new(format!("Bitte {feld} angeben.")));
        }
        Ok(wert.to_string())
    }
}

// ---------------------------------------------------------------- Bauobjekte

#[server(input = Json)]
pub async fn bauobjekte_laden() -> Result<Vec<Bauobjekt>, ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    // Die Projekte brauchen die Liste für die Auswahl des Bauobjekts
    recht_pflicht(&[(Bereich::Bauobjekte, Aktion::Sehen), (Bereich::Projekte, Aktion::Sehen)]).await?;

    let zeilen: Vec<(Uuid, String, i64, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>,
                     Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT o.id, o.bezeichnung, (SELECT COUNT(*) FROM projekt p WHERE p.bauobjekt_id = o.id),
                b.id, b.bezeichnung, b.strasse, b.plz, b.ort,
                a.id, a.bezeichnung, a.strasse, a.plz, a.ort
         FROM bauobjekt o
         LEFT JOIN adresse b ON b.id = o.bauherr_id
         LEFT JOIN adresse a ON a.id = o.architekt_id
         ORDER BY o.bezeichnung",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)?;

    Ok(zeilen
        .into_iter()
        .map(|(id, bezeichnung, anzahl_projekte, b1, b2, b3, b4, b5, a1, a2, a3, a4, a5)| Bauobjekt {
            id,
            bezeichnung,
            anzahl_projekte,
            bauherr: server::adresse((b1, b2, b3, b4, b5)),
            architekt: server::adresse((a1, a2, a3, a4, a5)),
        })
        .collect())
}

#[server(input = Json)]
pub async fn bauobjekt_speichern(id: Option<Uuid>, eingabe: BauobjektEingabe) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Bauobjekte, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
    let bezeichnung = server::pflichttext(&eingabe.bezeichnung, "eine Bezeichnung")?;

    match id {
        Some(id) => sqlx::query(
            "UPDATE bauobjekt SET bezeichnung = $2, bauherr_id = $3, architekt_id = $4 WHERE id = $1",
        )
        .bind(id),
        None => sqlx::query("INSERT INTO bauobjekt (bezeichnung, bauherr_id, architekt_id) VALUES ($1, $2, $3)"),
    }
    .bind(bezeichnung)
    .bind(eingabe.bauherr_id)
    .bind(eingabe.architekt_id)
    .execute(pool())
    .await
    .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn bauobjekt_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Bauobjekte, Aktion::Loeschen)]).await?;
    sqlx::query("DELETE FROM bauobjekt WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// ------------------------------------------------------------------ Projekte

#[server(input = Json)]
pub async fn projekte_laden() -> Result<Vec<Projekt>, ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    // Die Stundenerfassung braucht die Liste für die Projektauswahl
    recht_pflicht(&[
        (Bereich::Projekte, Aktion::Sehen),
        (Bereich::Bauobjekte, Aktion::Sehen),
        (Bereich::Buchungen, Aktion::Sehen),
        (Bereich::Stunden, Aktion::Sehen),
    ])
    .await?;

    let zeilen: Vec<(Uuid, String, String, Uuid, String,
                     Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>,
                     Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT p.id, p.projektnummer, p.bezeichnung, o.id, o.bezeichnung,
                b.id, b.bezeichnung, b.strasse, b.plz, b.ort,
                a.id, a.bezeichnung, a.strasse, a.plz, a.ort
         FROM projekt p
         JOIN bauobjekt o ON o.id = p.bauobjekt_id
         LEFT JOIN adresse b ON b.id = p.bauherr_id
         LEFT JOIN adresse a ON a.id = p.architekt_id
         ORDER BY p.projektnummer",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)?;

    Ok(zeilen
        .into_iter()
        .map(|(id, projektnummer, bezeichnung, bauobjekt_id, bauobjekt_bezeichnung, b1, b2, b3, b4, b5, a1, a2, a3, a4, a5)| {
            Projekt {
                id,
                projektnummer,
                bezeichnung,
                bauobjekt_id,
                bauobjekt_bezeichnung,
                bauherr: server::adresse((b1, b2, b3, b4, b5)),
                architekt: server::adresse((a1, a2, a3, a4, a5)),
            }
        })
        .collect())
}

#[server(input = Json)]
pub async fn projekt_speichern(id: Option<Uuid>, eingabe: ProjektEingabe) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Projekte, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
    let projektnummer = server::pflichttext(&eingabe.projektnummer, "eine Projektnummer")?;
    let bezeichnung = server::pflichttext(&eingabe.bezeichnung, "eine Bezeichnung")?;

    match id {
        Some(id) => sqlx::query(
            "UPDATE projekt SET projektnummer = $2, bezeichnung = $3, bauobjekt_id = $4, bauherr_id = $5, architekt_id = $6
             WHERE id = $1",
        )
        .bind(id),
        None => sqlx::query(
            "INSERT INTO projekt (projektnummer, bezeichnung, bauobjekt_id, bauherr_id, architekt_id)
             VALUES ($1, $2, $3, $4, $5)",
        ),
    }
    .bind(projektnummer)
    .bind(bezeichnung)
    .bind(eingabe.bauobjekt_id)
    .bind(eingabe.bauherr_id)
    .bind(eingabe.architekt_id)
    .execute(pool())
    .await
    .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn projekt_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Projekte, Aktion::Loeschen)]).await?;
    sqlx::query("DELETE FROM projekt WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// ----------------------------------------------------------------- Buchungen

#[server(input = Json)]
pub async fn buchungen_laden() -> Result<Vec<Buchung>, ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Buchungen, Aktion::Sehen), (Bereich::Stunden, Aktion::Sehen)]).await?;
    sqlx::query_as(
        "SELECT b.id, b.projekt_id, p.projektnummer, p.bezeichnung AS projekt_bezeichnung, b.zeit, b.erfasst_am
         FROM buchung b JOIN projekt p ON p.id = b.projekt_id
         ORDER BY b.erfasst_am DESC",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)
}

#[server(input = Json)]
pub async fn buchung_speichern(id: Option<Uuid>, projekt_id: Uuid, zeit: f64) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    match id {
        Some(_) => recht_pflicht(&[(Bereich::Buchungen, Aktion::Bearbeiten)]).await?,
        None => recht_pflicht(&[(Bereich::Buchungen, Aktion::Anlegen), (Bereich::Stunden, Aktion::Sehen)]).await?,
    };
    if !zeit.is_finite() || zeit <= 0.0 {
        return Err(ServerFnError::new("Die Zeit muss grösser als 0 sein."));
    }

    match id {
        Some(id) => sqlx::query("UPDATE buchung SET projekt_id = $2, zeit = $3 WHERE id = $1").bind(id),
        None => sqlx::query("INSERT INTO buchung (projekt_id, zeit) VALUES ($1, $2)"),
    }
    .bind(projekt_id)
    .bind(zeit)
    .execute(pool())
    .await
    .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn buchung_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Buchungen, Aktion::Loeschen)]).await?;
    sqlx::query("DELETE FROM buchung WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
