//! Server-Funktionen: werden im Browser aufgerufen und laufen auf dem Server.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::models::Bereich;
use crate::models::{Kurs, Mitarbeiter, Termin, TerminEingabe};

#[cfg(feature = "ssr")]
mod hilfe {
    use chrono::NaiveDateTime;
    use leptos::prelude::ServerFnError;

    use crate::models::DATETIME_LOCAL;

    pub fn pflicht(wert: String, feld: &str) -> Result<String, ServerFnError> {
        let wert = wert.trim().to_string();
        if wert.is_empty() {
            Err(ServerFnError::new(format!("{feld} darf nicht leer sein.")))
        } else {
            Ok(wert)
        }
    }

    pub fn zeitpunkt(wert: &str, feld: &str) -> Result<NaiveDateTime, ServerFnError> {
        NaiveDateTime::parse_from_str(wert, DATETIME_LOCAL)
            .or_else(|_| NaiveDateTime::parse_from_str(wert, "%Y-%m-%dT%H:%M:%S"))
            .map_err(|_| ServerFnError::new(format!("{feld} ist kein gültiges Datum.")))
    }
}

// ---------------------------------------------------------------- Übersicht

#[server(input = Json)]
pub async fn statistik() -> Result<(i64, i64, i64), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Uebersicht]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM kurs), (SELECT COUNT(*) FROM mitarbeiter), (SELECT COUNT(*) FROM termin)",
    )
    .fetch_one(pool())
    .await
    .map_err(fehler)
}

// -------------------------------------------------------------------- Kurse

#[server(input = Json)]
pub async fn kurse_laden() -> Result<Vec<Kurs>, ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Kurse, Bereich::Termine]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as("SELECT id, titel, beschreibung, dauer_stunden FROM kurs ORDER BY titel")
        .fetch_all(pool())
        .await
        .map_err(fehler)
}

#[server(input = Json)]
pub async fn kurs_speichern(
    id: Option<Uuid>,
    titel: String,
    beschreibung: String,
    dauer_stunden: i32,
) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Kurse]).await?;
    use crate::backend::db::{fehler, pool};
    let titel = hilfe::pflicht(titel, "Titel")?;
    if dauer_stunden < 0 {
        return Err(ServerFnError::new("Die Dauer darf nicht negativ sein."));
    }
    let abfrage = match id {
        Some(id) => sqlx::query(
            "UPDATE kurs SET titel = $2, beschreibung = $3, dauer_stunden = $4 WHERE id = $1",
        )
        .bind(id),
        None => sqlx::query(
            "INSERT INTO kurs (titel, beschreibung, dauer_stunden) VALUES ($1, $2, $3)",
        ),
    };
    abfrage
        .bind(titel)
        .bind(beschreibung.trim().to_string())
        .bind(dauer_stunden)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn kurs_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Kurse]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query("DELETE FROM kurs WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// -------------------------------------------------------------- Mitarbeiter

#[server(input = Json)]
pub async fn mitarbeiter_laden() -> Result<Vec<Mitarbeiter>, ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Mitarbeiter, Bereich::Termine]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as(
        "SELECT id, vorname, nachname, email, abteilung FROM mitarbeiter ORDER BY nachname, vorname",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)
}

#[server(input = Json)]
pub async fn mitarbeiter_speichern(
    id: Option<Uuid>,
    vorname: String,
    nachname: String,
    email: String,
    abteilung: String,
) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Mitarbeiter]).await?;
    use crate::backend::db::{fehler, pool};
    let vorname = hilfe::pflicht(vorname, "Vorname")?;
    let nachname = hilfe::pflicht(nachname, "Nachname")?;
    let email = hilfe::pflicht(email, "E-Mail")?.to_lowercase();
    if !email.contains('@') {
        return Err(ServerFnError::new("Bitte eine gültige E-Mail-Adresse angeben."));
    }
    let abfrage = match id {
        Some(id) => sqlx::query(
            "UPDATE mitarbeiter SET vorname = $2, nachname = $3, email = $4, abteilung = $5 WHERE id = $1",
        )
        .bind(id),
        None => sqlx::query(
            "INSERT INTO mitarbeiter (vorname, nachname, email, abteilung) VALUES ($1, $2, $3, $4)",
        ),
    };
    abfrage
        .bind(vorname)
        .bind(nachname)
        .bind(email)
        .bind(abteilung.trim().to_string())
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn mitarbeiter_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Mitarbeiter]).await?;
    use crate::backend::db::{fehler, pool};
    // Teilnahmen werden per ON DELETE CASCADE mitgelöscht.
    sqlx::query("DELETE FROM mitarbeiter WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// ------------------------------------------------------------------ Termine

#[server(input = Json)]
pub async fn termine_laden() -> Result<Vec<Termin>, ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Termine]).await?;
    use std::collections::HashMap;

    use chrono::NaiveDateTime;

    use crate::backend::db::{fehler, pool};

    let zeilen: Vec<(Uuid, Uuid, String, NaiveDateTime, NaiveDateTime, String, i32)> = sqlx::query_as(
        "SELECT t.id, t.kurs_id, k.titel, t.beginn, t.ende, t.ort, t.max_teilnehmer
         FROM termin t JOIN kurs k ON k.id = t.kurs_id
         ORDER BY t.beginn",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)?;

    let teilnahmen: Vec<(Uuid, Uuid, String, String, String, String)> = sqlx::query_as(
        "SELECT tm.termin_id, m.id, m.vorname, m.nachname, m.email, m.abteilung
         FROM termin_mitarbeiter tm JOIN mitarbeiter m ON m.id = tm.mitarbeiter_id
         ORDER BY m.nachname, m.vorname",
    )
    .fetch_all(pool())
    .await
    .map_err(fehler)?;

    let mut pro_termin: HashMap<Uuid, Vec<Mitarbeiter>> = HashMap::new();
    for (termin_id, id, vorname, nachname, email, abteilung) in teilnahmen {
        pro_termin.entry(termin_id).or_default().push(Mitarbeiter {
            id,
            vorname,
            nachname,
            email,
            abteilung,
        });
    }

    Ok(zeilen
        .into_iter()
        .map(|(id, kurs_id, kurs_titel, beginn, ende, ort, max_teilnehmer)| Termin {
            mitarbeiter: pro_termin.remove(&id).unwrap_or_default(),
            id,
            kurs_id,
            kurs_titel,
            beginn,
            ende,
            ort,
            max_teilnehmer,
        })
        .collect())
}

#[server(input = Json)]
pub async fn termin_speichern(id: Option<Uuid>, eingabe: TerminEingabe) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Termine]).await?;
    use crate::backend::db::{fehler, pool};

    let beginn = hilfe::zeitpunkt(&eingabe.beginn, "Beginn")?;
    let ende = hilfe::zeitpunkt(&eingabe.ende, "Ende")?;
    if ende <= beginn {
        return Err(ServerFnError::new("Das Ende muss nach dem Beginn liegen."));
    }
    if eingabe.max_teilnehmer < 1 {
        return Err(ServerFnError::new("Die maximale Teilnehmerzahl muss mindestens 1 sein."));
    }
    if eingabe.mitarbeiter_ids.len() > eingabe.max_teilnehmer as usize {
        return Err(ServerFnError::new(format!(
            "Es sind {} Teilnehmende ausgewählt, erlaubt sind höchstens {}.",
            eingabe.mitarbeiter_ids.len(),
            eingabe.max_teilnehmer
        )));
    }

    let mut tx = pool().begin().await.map_err(fehler)?;

    let termin_id: Uuid = match id {
        Some(id) => {
            sqlx::query(
                "UPDATE termin SET kurs_id = $2, beginn = $3, ende = $4, ort = $5, max_teilnehmer = $6
                 WHERE id = $1",
            )
                .bind(id)
                .bind(eingabe.kurs_id)
                .bind(beginn)
                .bind(ende)
                .bind(eingabe.ort.trim().to_string())
                .bind(eingabe.max_teilnehmer)
                .execute(&mut *tx)
                .await
                .map_err(fehler)?;
            sqlx::query("DELETE FROM termin_mitarbeiter WHERE termin_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(fehler)?;
            id
        }
        None => sqlx::query_scalar(
            "INSERT INTO termin (kurs_id, beginn, ende, ort, max_teilnehmer) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(eingabe.kurs_id)
        .bind(beginn)
        .bind(ende)
        .bind(eingabe.ort.trim().to_string())
        .bind(eingabe.max_teilnehmer)
        .fetch_one(&mut *tx)
        .await
        .map_err(fehler)?,
    };

    sqlx::query(
        "INSERT INTO termin_mitarbeiter (termin_id, mitarbeiter_id)
         SELECT $1, UNNEST($2::uuid[]) ON CONFLICT DO NOTHING",
    )
    .bind(termin_id)
    .bind(eingabe.mitarbeiter_ids)
    .execute(&mut *tx)
    .await
    .map_err(fehler)?;

    tx.commit().await.map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn termin_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::bereich_pflicht(&[Bereich::Termine]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query("DELETE FROM termin WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
