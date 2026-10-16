//! Rollenverwaltung: Rollen bündeln Rechte pro Bereich und Funktion.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

use crate::models::Rolle;

#[server(input = Json)]
pub async fn rollen_laden() -> Result<Vec<Rolle>, ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    // Die Benutzerverwaltung braucht die Liste für die Rollenauswahl
    recht_pflicht(&[(Bereich::Rollen, Aktion::Sehen), (Bereich::Benutzer, Aktion::Sehen)]).await?;
    sqlx::query_as(
        "SELECT r.id, r.name, r.beschreibung, r.ist_admin, r.rechte,
                (SELECT COUNT(*) FROM benutzer b WHERE b.rolle_id = r.id) AS anzahl_benutzer
         FROM rolle r
         ORDER BY r.ist_admin DESC, r.name",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)
}

#[server(input = Json)]
pub async fn rolle_speichern(
    id: Option<Uuid>,
    name: String,
    beschreibung: String,
    rechte: Vec<String>,
) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich, alle_rechte, recht};

    let aktion = if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen };
    let ich = recht_pflicht(&[(Bereich::Rollen, aktion)]).await?;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Bitte einen Namen angeben."));
    }

    // Nur bekannte Rechte; wer anlegen, bearbeiten oder löschen darf, muss auch sehen dürfen
    let bekannt = alle_rechte();
    let mut rechte: Vec<String> = rechte.into_iter().filter(|r| bekannt.contains(r)).collect();
    for bereich in Bereich::ALLE {
        let sehen = recht(bereich, Aktion::Sehen);
        let praefix = format!("{}.", bereich.schluessel());
        if rechte.iter().any(|r| r.starts_with(&praefix)) && !rechte.contains(&sehen) {
            rechte.push(sehen);
        }
    }
    rechte.sort();
    rechte.dedup();

    // Niemand kann mehr vergeben, als er selbst darf
    if let Some(fremd) = rechte.iter().find(|r| !ich.ist_admin && !ich.rechte.contains(r)) {
        return Err(ServerFnError::new(format!(
            "Du kannst das Recht „{fremd}“ nicht vergeben, weil du es selbst nicht hast."
        )));
    }

    match id {
        None => {
            sqlx::query("INSERT INTO rolle (name, beschreibung, rechte) VALUES ($1, $2, $3)")
                .bind(name)
                .bind(beschreibung.trim().to_string())
                .bind(&rechte)
                .execute(pool())
                .await
                .map_err(fehler)?;
        }
        Some(id) => {
            let ist_admin: bool = sqlx::query_scalar("SELECT ist_admin FROM rolle WHERE id = $1")
                .bind(id)
                .fetch_optional(pool())
                .await
                .map_err(fehler)?
                .ok_or_else(|| ServerFnError::new("Diese Rolle gibt es nicht."))?;
            if ist_admin {
                // Die Admin-Rolle hat immer alle Rechte; nur die Beschreibung ist änderbar
                if !ich.ist_admin {
                    return Err(ServerFnError::new("Die Administrator-Rolle können nur Administratoren ändern."));
                }
                sqlx::query("UPDATE rolle SET beschreibung = $2 WHERE id = $1")
                    .bind(id)
                    .bind(beschreibung.trim().to_string())
                    .execute(pool())
                    .await
                    .map_err(fehler)?;
            } else {
                sqlx::query("UPDATE rolle SET name = $2, beschreibung = $3, rechte = $4 WHERE id = $1")
                    .bind(id)
                    .bind(name)
                    .bind(beschreibung.trim().to_string())
                    .bind(&rechte)
                    .execute(pool())
                    .await
                    .map_err(fehler)?;
            }
        }
    }
    Ok(())
}

#[server(input = Json)]
pub async fn rolle_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    use crate::backend::auth::server::recht_pflicht;
    use crate::backend::db::{fehler, pool};
    use crate::models::{Aktion, Bereich};

    recht_pflicht(&[(Bereich::Rollen, Aktion::Loeschen)]).await?;
    let ist_admin: Option<bool> = sqlx::query_scalar("SELECT ist_admin FROM rolle WHERE id = $1")
        .bind(id)
        .fetch_optional(pool())
        .await
        .map_err(fehler)?;
    if ist_admin == Some(true) {
        return Err(ServerFnError::new("Die Administrator-Rolle kann nicht gelöscht werden."));
    }
    // Rollen mit Benutzern sind per Fremdschlüssel geschützt (Meldung in db::fehler)
    sqlx::query("DELETE FROM rolle WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
