//! SQL-Zugriffe für Artikel und Kleidergrössen (nur Server).

use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Artikel, ArtikelEingabe, BildAenderung, BildUpload, Kleidergroesse};

/// Alle Artikel ohne Bilddaten, nach Bezeichnung sortiert.
pub async fn alle(pool: &PgPool) -> sqlx::Result<Vec<Artikel>> {
    let zeilen: Vec<(
        Uuid,
        String,
        Option<String>,
        Uuid,
        String,
        NaiveDateTime,
        NaiveDateTime,
    )> = sqlx::query_as(
        "SELECT a.id, a.bezeichnung, a.bild, a.kleidergroesse_id, k.bezeichnung,
                a.erstellt_am, a.geaendert_am
         FROM artikel a
         JOIN kleidergroesse k ON k.id = a.kleidergroesse_id
         ORDER BY a.bezeichnung",
    )
    .fetch_all(pool)
    .await?;
    Ok(zeilen
        .into_iter()
        .map(
            |(id, bezeichnung, bild, kleidergroesse_id, groesse, erstellt_am, geaendert_am)| {
                Artikel {
                    id,
                    bezeichnung,
                    bild,
                    kleidergroesse_id,
                    groesse,
                    erstellt_am,
                    geaendert_am,
                }
            },
        )
        .collect())
}

/// Alle Kleidergrössen in der festgelegten Reihenfolge.
pub async fn kleidergroessen(pool: &PgPool) -> sqlx::Result<Vec<Kleidergroesse>> {
    sqlx::query_as("SELECT id, bezeichnung FROM kleidergroesse ORDER BY sortierung")
        .fetch_all(pool)
        .await
}

/// Dateiname und Inhalt des Bildes; `None`, wenn es den Artikel oder ein Bild nicht gibt.
pub async fn bild(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<(String, Vec<u8>)>> {
    let zeile: Option<(Option<String>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT bild, bild_daten FROM artikel WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(match zeile {
        Some((Some(name), Some(daten))) => Some((name, daten)),
        _ => None,
    })
}

/// Legt einen Artikel an und gibt seine Kennung zurück.
pub async fn einfuegen(pool: &PgPool, e: &ArtikelEingabe) -> sqlx::Result<Uuid> {
    let neu = neues_bild(&e.bild);
    sqlx::query_scalar(
        "INSERT INTO artikel (bezeichnung, kleidergroesse_id, bild, bild_daten)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(e.bezeichnung.trim())
    .bind(e.kleidergroesse_id)
    .bind(neu.map(|b| b.name.trim()))
    .bind(neu.map(|b| b.daten.as_slice()))
    .fetch_one(pool)
    .await
}

/// Ändert einen Artikel. Ein Bild bleibt erhalten, außer es wird ersetzt oder entfernt.
/// Gibt die Zahl der geänderten Zeilen zurück; `0` heißt, dass es den Artikel nicht (mehr) gibt.
pub async fn aktualisieren(pool: &PgPool, id: Uuid, e: &ArtikelEingabe) -> sqlx::Result<u64> {
    let neu = neues_bild(&e.bild);
    let bild_aendern = !matches!(e.bild, BildAenderung::Behalten);
    let zeilen = sqlx::query(
        "UPDATE artikel
         SET bezeichnung = $2,
             kleidergroesse_id = $3,
             geaendert_am = now(),
             bild = CASE WHEN $4 THEN $5::text ELSE bild END,
             bild_daten = CASE WHEN $4 THEN $6::bytea ELSE bild_daten END
         WHERE id = $1",
    )
    .bind(id)
    .bind(e.bezeichnung.trim())
    .bind(e.kleidergroesse_id)
    .bind(bild_aendern)
    .bind(neu.map(|b| b.name.trim()))
    .bind(neu.map(|b| b.daten.as_slice()))
    .execute(pool)
    .await?;
    Ok(zeilen.rows_affected())
}

/// Löscht einen Artikel. Gibt die Zahl der gelöschten Zeilen zurück.
pub async fn loeschen(pool: &PgPool, id: Uuid) -> sqlx::Result<u64> {
    let zeilen = sqlx::query("DELETE FROM artikel WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(zeilen.rows_affected())
}

/// Das neu hochgeladene Bild, falls es eins gibt
fn neues_bild(bild: &BildAenderung) -> Option<&BildUpload> {
    match bild {
        BildAenderung::Neu(upload) => Some(upload),
        BildAenderung::Behalten | BildAenderung::Entfernen => None,
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    const JPG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];

    /// Kleidergrösse „XL“ aus den Startwerten der Migration
    async fn xl(pool: &PgPool) -> Uuid {
        sqlx::query_scalar("SELECT id FROM kleidergroesse WHERE bezeichnung = 'XL'")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn eingabe(pool: &PgPool, bild: BildAenderung) -> ArtikelEingabe {
        ArtikelEingabe {
            bezeichnung: "Schraube".into(),
            kleidergroesse_id: Some(xl(pool).await),
            bild,
        }
    }

    fn mit_bild() -> BildAenderung {
        BildAenderung::Neu(BildUpload {
            name: "schraube.jpg".into(),
            daten: JPG.to_vec(),
        })
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_artikel_mit_bild_wird_gespeichert_und_gelesen(pool: PgPool) {
        let id = einfuegen(&pool, &eingabe(&pool, mit_bild()).await)
            .await
            .unwrap();

        let liste = alle(&pool).await.unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].groesse, "XL");
        assert_eq!(liste[0].bild.as_deref(), Some("schraube.jpg"));
        assert_eq!(
            bild(&pool, id).await.unwrap(),
            Some(("schraube.jpg".to_string(), JPG.to_vec()))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_artikel_ohne_bild_hat_kein_bild(pool: PgPool) {
        let id = einfuegen(&pool, &eingabe(&pool, BildAenderung::Behalten).await)
            .await
            .unwrap();

        assert_eq!(alle(&pool).await.unwrap()[0].bild, None);
        assert_eq!(bild(&pool, id).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bearbeiten_ohne_neues_bild_behaelt_das_bild(pool: PgPool) {
        let id = einfuegen(&pool, &eingabe(&pool, mit_bild()).await)
            .await
            .unwrap();

        let betroffen = aktualisieren(&pool, id, &eingabe(&pool, BildAenderung::Behalten).await)
            .await
            .unwrap();

        assert_eq!(betroffen, 1);
        assert_eq!(
            bild(&pool, id).await.unwrap(),
            Some(("schraube.jpg".to_string(), JPG.to_vec()))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bearbeiten_mit_entfernen_loescht_das_bild(pool: PgPool) {
        let id = einfuegen(&pool, &eingabe(&pool, mit_bild()).await)
            .await
            .unwrap();

        aktualisieren(&pool, id, &eingabe(&pool, BildAenderung::Entfernen).await)
            .await
            .unwrap();

        assert_eq!(bild(&pool, id).await.unwrap(), None);
        assert_eq!(alle(&pool).await.unwrap()[0].bild, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_unbekannter_artikel_wird_nicht_geaendert(pool: PgPool) {
        let betroffen = aktualisieren(
            &pool,
            Uuid::nil(),
            &eingabe(&pool, BildAenderung::Behalten).await,
        )
        .await
        .unwrap();
        assert_eq!(betroffen, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_kleidergroessen_kommen_in_der_festgelegten_reihenfolge(pool: PgPool) {
        let namen: Vec<String> = kleidergroessen(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|k| k.bezeichnung)
            .collect();
        assert_eq!(namen, ["S", "M", "L", "XL"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_unbekannte_kleidergroesse_wird_abgelehnt(pool: PgPool) {
        let ergebnis = sqlx::query(
            "INSERT INTO artikel (bezeichnung, kleidergroesse_id) VALUES ('Schraube', gen_random_uuid())",
        )
        .execute(&pool)
        .await;
        assert!(ergebnis.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_kleidergroesse_mit_artikeln_laesst_sich_nicht_loeschen(pool: PgPool) {
        einfuegen(&pool, &eingabe(&pool, BildAenderung::Behalten).await)
            .await
            .unwrap();

        let ergebnis = sqlx::query("DELETE FROM kleidergroesse WHERE bezeichnung = 'XL'")
            .execute(&pool)
            .await;
        assert!(ergebnis.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_leere_bezeichnung_wird_abgelehnt(pool: PgPool) {
        let ergebnis = sqlx::query(
            "INSERT INTO artikel (bezeichnung, kleidergroesse_id) VALUES ('   ', (SELECT id FROM kleidergroesse WHERE bezeichnung = 'M'))",
        )
        .execute(&pool)
        .await;
        assert!(ergebnis.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bild_und_bilddaten_nur_gemeinsam(pool: PgPool) {
        let ergebnis = sqlx::query(
            "INSERT INTO artikel (bezeichnung, kleidergroesse_id, bild, bild_daten) VALUES ('Schraube', (SELECT id FROM kleidergroesse WHERE bezeichnung = 'M'), 'a.jpg', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(ergebnis.is_err());
    }
}
