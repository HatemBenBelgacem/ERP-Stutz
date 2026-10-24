//! Server-Funktionen: werden im Browser aufgerufen und laufen auf dem Server.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::models::{Aktion, Bereich};
use crate::models::{Adresse, Geschaeftsbereich, Kurs, Mitarbeiter, Termin, TerminEingabe};

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

// -------------------------------------------------------------------- Kurse

#[server(input = Json)]
pub async fn kurse_laden() -> Result<Vec<Kurs>, ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Kurse, Aktion::Sehen), (Bereich::Termine, Aktion::Sehen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as(
        "SELECT k.id, k.titel, k.beschreibung, k.dauer_stunden,
                COALESCE(array_agg(kg.geschaeftsbereich_id) FILTER (WHERE kg.geschaeftsbereich_id IS NOT NULL), '{}')
                    AS geschaeftsbereich_ids
         FROM kurs k LEFT JOIN kurs_geschaeftsbereich kg ON kg.kurs_id = k.id
         GROUP BY k.id
         ORDER BY k.titel",
    )
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
    geschaeftsbereich_ids: Vec<Uuid>,
) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Kurse, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
    use crate::backend::db::{fehler, pool};
    let titel = hilfe::pflicht(titel, "Titel")?;
    if dauer_stunden < 0 {
        return Err(ServerFnError::new("Die Dauer darf nicht negativ sein."));
    }

    let mut tx = pool().begin().await.map_err(fehler)?;

    let abfrage = match id {
        Some(id) => sqlx::query_scalar(
            "UPDATE kurs SET titel = $2, beschreibung = $3, dauer_stunden = $4 WHERE id = $1 RETURNING id",
        )
        .bind(id),
        None => sqlx::query_scalar(
            "INSERT INTO kurs (titel, beschreibung, dauer_stunden) VALUES ($1, $2, $3) RETURNING id",
        ),
    };
    let kurs_id: Uuid = abfrage
        .bind(titel)
        .bind(beschreibung.trim().to_string())
        .bind(dauer_stunden)
        .fetch_one(&mut *tx)
        .await
        .map_err(fehler)?;

    sqlx::query("DELETE FROM kurs_geschaeftsbereich WHERE kurs_id = $1")
        .bind(kurs_id)
        .execute(&mut *tx)
        .await
        .map_err(fehler)?;
    sqlx::query(
        "INSERT INTO kurs_geschaeftsbereich (kurs_id, geschaeftsbereich_id)
         SELECT $1, UNNEST($2::uuid[]) ON CONFLICT DO NOTHING",
    )
    .bind(kurs_id)
    .bind(geschaeftsbereich_ids)
    .execute(&mut *tx)
    .await
    .map_err(fehler)?;

    tx.commit().await.map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn kurs_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Kurse, Aktion::Loeschen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query("DELETE FROM kurs WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// -------------------------------------------------------- Geschäftsbereiche

#[server(input = Json)]
pub async fn geschaeftsbereiche_laden() -> Result<Vec<Geschaeftsbereich>, ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Geschaeftsbereiche, Aktion::Sehen), (Bereich::Kurse, Aktion::Sehen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as("SELECT id, beschreibung FROM geschaeftsbereich ORDER BY beschreibung")
        .fetch_all(pool())
        .await
        .map_err(fehler)
}

#[server(input = Json)]
pub async fn geschaeftsbereich_speichern(id: Option<Uuid>, beschreibung: String) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Geschaeftsbereiche, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
    use crate::backend::db::{fehler, pool};
    let beschreibung = hilfe::pflicht(beschreibung, "Beschreibung")?;
    let abfrage = match id {
        Some(id) => sqlx::query("UPDATE geschaeftsbereich SET beschreibung = $2 WHERE id = $1").bind(id),
        None => sqlx::query("INSERT INTO geschaeftsbereich (beschreibung) VALUES ($1)"),
    };
    abfrage.bind(beschreibung).execute(pool()).await.map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn geschaeftsbereich_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Geschaeftsbereiche, Aktion::Loeschen)]).await?;
    use crate::backend::db::{fehler, pool};
    // Zuordnungen zu Kursen werden per ON DELETE CASCADE mitgelöscht.
    sqlx::query("DELETE FROM geschaeftsbereich WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// ----------------------------------------------------------------- Adressen

#[server(input = Json)]
pub async fn adressen_laden() -> Result<Vec<Adresse>, ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Adressen, Aktion::Sehen), (Bereich::Termine, Aktion::Sehen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query_as("SELECT id, bezeichnung, strasse, plz, ort FROM adresse ORDER BY bezeichnung, ort")
        .fetch_all(pool())
        .await
        .map_err(fehler)
}

#[server(input = Json)]
pub async fn adresse_speichern(
    id: Option<Uuid>,
    bezeichnung: String,
    strasse: String,
    plz: String,
    ort: String,
) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Adressen, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
    use crate::backend::db::{fehler, pool};
    let bezeichnung = hilfe::pflicht(bezeichnung, "Bezeichnung")?;
    let abfrage = match id {
        Some(id) => sqlx::query(
            "UPDATE adresse SET bezeichnung = $2, strasse = $3, plz = $4, ort = $5 WHERE id = $1",
        )
        .bind(id),
        None => sqlx::query("INSERT INTO adresse (bezeichnung, strasse, plz, ort) VALUES ($1, $2, $3, $4)"),
    };
    abfrage
        .bind(bezeichnung)
        .bind(strasse.trim().to_string())
        .bind(plz.trim().to_string())
        .bind(ort.trim().to_string())
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn adresse_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Adressen, Aktion::Loeschen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query("DELETE FROM adresse WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}

// -------------------------------------------------------------- Mitarbeiter

#[server(input = Json)]
pub async fn mitarbeiter_laden() -> Result<Vec<Mitarbeiter>, ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Mitarbeiter, Aktion::Sehen), (Bereich::Termine, Aktion::Sehen), (Bereich::Anmeldung, Aktion::Sehen)]).await?;
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
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Mitarbeiter, if id.is_some() { Aktion::Bearbeiten } else { Aktion::Anlegen })]).await?;
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
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Mitarbeiter, Aktion::Loeschen)]).await?;
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
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Termine, Aktion::Sehen)]).await?;
    termine_abfragen(false).await
}

/// Termine, die noch nicht beendet sind und nicht in der Vergangenheit liegen.
#[server(input = Json)]
pub async fn aktive_termine_laden() -> Result<Vec<Termin>, ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Anmeldung, Aktion::Sehen), (Bereich::Termine, Aktion::Sehen)]).await?;
    termine_abfragen(true).await
}

/// Mitarbeitende zu einem Termin anmelden. Bisherige Teilnehmende bleiben,
/// alle anderen Angaben zum Termin auch.
#[server(input = Json)]
pub async fn teilnehmende_anmelden(termin_id: Uuid, mitarbeiter_ids: Vec<Uuid>) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[
        (Bereich::Anmeldung, Aktion::TeilnehmendeHinzufuegen),
        (Bereich::Termine, Aktion::TeilnehmendeHinzufuegen),
        (Bereich::Termine, Aktion::Bearbeiten),
    ])
    .await?;
    teilnehmende_hinzufuegen(termin_id, mitarbeiter_ids).await
}

#[cfg(feature = "ssr")]
async fn termine_abfragen(nur_aktive: bool) -> Result<Vec<Termin>, ServerFnError> {
    use std::collections::HashMap;

    use chrono::NaiveDateTime;

    use crate::backend::db::{fehler, pool};

    type Zeile = (
        Uuid, Uuid, String, NaiveDateTime, NaiveDateTime, i32, String,
        Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<String>,
    );
    let zeilen: Vec<Zeile> = sqlx::query_as(
        "SELECT t.id, t.kurs_id, k.titel, t.beginn, t.ende, t.max_teilnehmer, t.status::text,
                a.id, a.bezeichnung, a.strasse, a.plz, a.ort
         FROM termin t
         JOIN kurs k ON k.id = t.kurs_id
         LEFT JOIN adresse a ON a.id = t.adresse_id
         WHERE NOT $1 OR (t.status <> 'beendet' AND t.ende >= localtimestamp)
         ORDER BY t.beginn",
    )
    .bind(nur_aktive)
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
        .map(|(id, kurs_id, kurs_titel, beginn, ende, max_teilnehmer, status, adresse_id, bezeichnung, strasse, plz, ort)| Termin {
            mitarbeiter: pro_termin.remove(&id).unwrap_or_default(),
            adresse: adresse_id.map(|adresse_id| Adresse {
                id: adresse_id,
                bezeichnung: bezeichnung.unwrap_or_default(),
                strasse: strasse.unwrap_or_default(),
                plz: plz.unwrap_or_default(),
                ort: ort.unwrap_or_default(),
            }),
            id,
            kurs_id,
            kurs_titel,
            beginn,
            ende,
            max_teilnehmer,
            status: crate::models::TerminStatus::aus_schluessel(&status).unwrap_or_default(),
        })
        .collect())
}

#[server(input = Json)]
pub async fn termin_speichern(id: Option<Uuid>, eingabe: TerminEingabe) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};

    // Wer bestehende Termine nicht bearbeiten, aber Teilnehmende hinzufügen
    // darf, ergänzt nur Teilnehmende; alle anderen Angaben bleiben unverändert.
    let rechte: &[(Bereich, Aktion)] = if id.is_some() {
        &[(Bereich::Termine, Aktion::Bearbeiten), (Bereich::Termine, Aktion::TeilnehmendeHinzufuegen)]
    } else {
        &[(Bereich::Termine, Aktion::Anlegen)]
    };
    let ich = crate::backend::auth::server::recht_pflicht(rechte).await?;
    if let Some(id) = id.filter(|_| !ich.darf(Bereich::Termine, Aktion::Bearbeiten)) {
        return teilnehmende_hinzufuegen(id, eingabe.mitarbeiter_ids).await;
    }

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
                "UPDATE termin SET kurs_id = $2, beginn = $3, ende = $4, adresse_id = $5, max_teilnehmer = $6,
                                   status = $7::termin_status
                 WHERE id = $1",
            )
                .bind(id)
                .bind(eingabe.kurs_id)
                .bind(beginn)
                .bind(ende)
                .bind(eingabe.adresse_id)
                .bind(eingabe.max_teilnehmer)
                .bind(eingabe.status.schluessel())
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
            "INSERT INTO termin (kurs_id, beginn, ende, adresse_id, max_teilnehmer, status)
             VALUES ($1, $2, $3, $4, $5, $6::termin_status) RETURNING id",
        )
        .bind(eingabe.kurs_id)
        .bind(beginn)
        .bind(ende)
        .bind(eingabe.adresse_id)
        .bind(eingabe.max_teilnehmer)
        .bind(eingabe.status.schluessel())
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

/// Fügt einem Termin Teilnehmende hinzu, ohne bestehende zu entfernen.
#[cfg(feature = "ssr")]
async fn teilnehmende_hinzufuegen(termin_id: Uuid, mitarbeiter_ids: Vec<Uuid>) -> Result<(), ServerFnError> {
    use crate::backend::db::{fehler, pool};

    let mut tx = pool().begin().await.map_err(fehler)?;
    // Termin sperren, damit gleichzeitige Anmeldungen die Obergrenze nicht überschreiten
    let max: Option<i32> = sqlx::query_scalar("SELECT max_teilnehmer FROM termin WHERE id = $1 FOR UPDATE")
        .bind(termin_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(fehler)?;
    let max = max.ok_or_else(|| ServerFnError::new("Den Termin gibt es nicht mehr."))?;

    sqlx::query(
        "INSERT INTO termin_mitarbeiter (termin_id, mitarbeiter_id)
         SELECT $1, UNNEST($2::uuid[]) ON CONFLICT DO NOTHING",
    )
    .bind(termin_id)
    .bind(mitarbeiter_ids)
    .execute(&mut *tx)
    .await
    .map_err(fehler)?;

    let belegt: i64 = sqlx::query_scalar("SELECT count(*) FROM termin_mitarbeiter WHERE termin_id = $1")
        .bind(termin_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(fehler)?;
    if belegt > max as i64 {
        return Err(ServerFnError::new(format!(
            "Der Termin hätte damit {belegt} Teilnehmende, erlaubt sind höchstens {max}."
        )));
    }
    tx.commit().await.map_err(fehler)?;
    Ok(())
}

#[server(input = Json)]
pub async fn termin_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    crate::backend::auth::server::recht_pflicht(&[(Bereich::Termine, Aktion::Loeschen)]).await?;
    use crate::backend::db::{fehler, pool};
    sqlx::query("DELETE FROM termin WHERE id = $1")
        .bind(id)
        .execute(pool())
        .await
        .map_err(fehler)?;
    Ok(())
}
