//! SQL-Zugriffe für Artikel und ihre Grössen (nur Server).

use chrono::NaiveDateTime;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::model::{Artikel, ArtikelEingabe, BildAenderung, BildUpload};

/// Alle Artikel mit ihren Grössen, ohne Bilddaten, nach Bezeichnung sortiert.
pub async fn alle(pool: &PgPool) -> sqlx::Result<Vec<Artikel>> {
    let zeilen: Vec<(
        Uuid,
        String,
        Option<String>,
        Vec<String>,
        NaiveDateTime,
        NaiveDateTime,
    )> = sqlx::query_as(
        "SELECT a.id, a.bezeichnung, a.bild,
                COALESCE(array_agg(k.bezeichnung ORDER BY k.sortierung)
                         FILTER (WHERE k.id IS NOT NULL), '{}') AS groessen,
                a.erstellt_am, a.geaendert_am
         FROM artikel a
         LEFT JOIN kleidergroesse k ON k.artikel_id = a.id
         GROUP BY a.id
         ORDER BY a.bezeichnung",
    )
    .fetch_all(pool)
    .await?;
    Ok(zeilen
        .into_iter()
        .map(
            |(id, bezeichnung, bild, groessen, erstellt_am, geaendert_am)| Artikel {
                id,
                bezeichnung,
                bild,
                groessen,
                erstellt_am,
                geaendert_am,
            },
        )
        .collect())
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

/// Legt einen Artikel mit seinen Grössen an und gibt seine Kennung zurück.
pub async fn einfuegen(tx: &mut PgConnection, e: &ArtikelEingabe) -> sqlx::Result<Uuid> {
    let neu = neues_bild(&e.bild);
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO artikel (bezeichnung, bild, bild_daten)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(e.bezeichnung.trim())
    .bind(neu.map(|b| b.name.trim()))
    .bind(neu.map(|b| b.daten.as_slice()))
    .fetch_one(&mut *tx)
    .await?;
    groessen_setzen(&mut *tx, id, &e.groessen_bereinigt()).await?;
    Ok(id)
}

/// Ändert einen Artikel und ersetzt seine Grössen. Ein Bild bleibt erhalten, außer es
/// wird ersetzt oder entfernt. Gibt die Zahl der geänderten Zeilen zurück; `0` heißt,
/// dass es den Artikel nicht (mehr) gibt. Dann werden auch keine Grössen geschrieben.
pub async fn aktualisieren(
    tx: &mut PgConnection,
    id: Uuid,
    e: &ArtikelEingabe,
) -> sqlx::Result<u64> {
    let neu = neues_bild(&e.bild);
    let bild_aendern = !matches!(e.bild, BildAenderung::Behalten);
    let zeilen = sqlx::query(
        "UPDATE artikel
         SET bezeichnung = $2,
             geaendert_am = now(),
             bild = CASE WHEN $3 THEN $4::text ELSE bild END,
             bild_daten = CASE WHEN $3 THEN $5::bytea ELSE bild_daten END
         WHERE id = $1",
    )
    .bind(id)
    .bind(e.bezeichnung.trim())
    .bind(bild_aendern)
    .bind(neu.map(|b| b.name.trim()))
    .bind(neu.map(|b| b.daten.as_slice()))
    .execute(&mut *tx)
    .await?;
    if zeilen.rows_affected() == 0 {
        return Ok(0);
    }
    groessen_setzen(&mut *tx, id, &e.groessen_bereinigt()).await?;
    Ok(1)
}

/// Löscht einen Artikel. Seine Grössen werden mitgelöscht. Gibt die Zahl der gelöschten Zeilen zurück.
pub async fn loeschen(pool: &PgPool, id: Uuid) -> sqlx::Result<u64> {
    let zeilen = sqlx::query("DELETE FROM artikel WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(zeilen.rows_affected())
}

/// Ersetzt die Grössen eines Artikels durch die gegebene Reihenfolge.
async fn groessen_setzen(
    tx: &mut PgConnection,
    artikel_id: Uuid,
    groessen: &[String],
) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM kleidergroesse WHERE artikel_id = $1")
        .bind(artikel_id)
        .execute(&mut *tx)
        .await?;
    for (nr, bezeichnung) in groessen.iter().enumerate() {
        sqlx::query(
            "INSERT INTO kleidergroesse (artikel_id, bezeichnung, sortierung)
             VALUES ($1, $2, $3)",
        )
        .bind(artikel_id)
        .bind(bezeichnung)
        .bind(nr as i32 + 1)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
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

    fn eingabe(groessen: &[&str], bild: BildAenderung) -> ArtikelEingabe {
        ArtikelEingabe {
            bezeichnung: "Schraube".into(),
            groessen: groessen.iter().map(|g| g.to_string()).collect(),
            bild,
        }
    }

    fn mit_bild() -> BildAenderung {
        BildAenderung::Neu(BildUpload {
            name: "schraube.jpg".into(),
            daten: JPG.to_vec(),
        })
    }

    /// Legt einen Artikel in eigener Transaktion an
    async fn anlegen(pool: &PgPool, e: &ArtikelEingabe) -> Uuid {
        let mut tx = pool.begin().await.unwrap();
        let id = einfuegen(&mut tx, e).await.unwrap();
        tx.commit().await.unwrap();
        id
    }

    /// Ändert einen Artikel in eigener Transaktion
    async fn aendern(pool: &PgPool, id: Uuid, e: &ArtikelEingabe) -> u64 {
        let mut tx = pool.begin().await.unwrap();
        let betroffen = aktualisieren(&mut tx, id, e).await.unwrap();
        tx.commit().await.unwrap();
        betroffen
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_artikel_mit_bild_und_grossen_wird_gespeichert_und_gelesen(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["S", "M", "L"], mit_bild())).await;

        let liste = alle(&pool).await.unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].groessen, ["S", "M", "L"]);
        assert_eq!(liste[0].bild.as_deref(), Some("schraube.jpg"));
        assert_eq!(
            bild(&pool, id).await.unwrap(),
            Some(("schraube.jpg".to_string(), JPG.to_vec()))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_artikel_ohne_bild_hat_kein_bild(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["M"], BildAenderung::Behalten)).await;

        assert_eq!(alle(&pool).await.unwrap()[0].bild, None);
        assert_eq!(bild(&pool, id).await.unwrap(), None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bearbeiten_ohne_neues_bild_behaelt_das_bild(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["M"], mit_bild())).await;

        let betroffen = aendern(&pool, id, &eingabe(&["M"], BildAenderung::Behalten)).await;

        assert_eq!(betroffen, 1);
        assert_eq!(
            bild(&pool, id).await.unwrap(),
            Some(("schraube.jpg".to_string(), JPG.to_vec()))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bearbeiten_mit_entfernen_loescht_das_bild(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["M"], mit_bild())).await;

        aendern(&pool, id, &eingabe(&["M"], BildAenderung::Entfernen)).await;

        assert_eq!(bild(&pool, id).await.unwrap(), None);
        assert_eq!(alle(&pool).await.unwrap()[0].bild, None);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bearbeiten_ersetzt_die_grossen(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["S", "M"], BildAenderung::Behalten)).await;

        aendern(&pool, id, &eingabe(&["L", "XL"], BildAenderung::Behalten)).await;

        assert_eq!(alle(&pool).await.unwrap()[0].groessen, ["L", "XL"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_unbekannter_artikel_wird_nicht_geaendert(pool: PgPool) {
        let betroffen = aendern(
            &pool,
            Uuid::nil(),
            &eingabe(&["M"], BildAenderung::Behalten),
        )
        .await;
        assert_eq!(betroffen, 0);
        assert!(alle(&pool).await.unwrap().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_grosse_je_artikel_ist_eindeutig(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["M"], BildAenderung::Behalten)).await;

        let ergebnis = sqlx::query(
            "INSERT INTO kleidergroesse (artikel_id, bezeichnung, sortierung) VALUES ($1, 'M', 2)",
        )
        .bind(id)
        .execute(&pool)
        .await;
        assert!(ergebnis.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_gleiche_grosse_bei_verschiedenen_artikeln_ist_erlaubt(pool: PgPool) {
        anlegen(&pool, &eingabe(&["M"], BildAenderung::Behalten)).await;
        anlegen(&pool, &eingabe(&["M"], BildAenderung::Behalten)).await;

        assert_eq!(alle(&pool).await.unwrap().len(), 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_loeschen_entfernt_die_grossen(pool: PgPool) {
        let id = anlegen(&pool, &eingabe(&["S", "M"], BildAenderung::Behalten)).await;

        loeschen(&pool, id).await.unwrap();

        let (anzahl,): (i64,) = sqlx::query_as("SELECT count(*) FROM kleidergroesse")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(anzahl, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_leere_bezeichnung_wird_abgelehnt(pool: PgPool) {
        let ergebnis = sqlx::query("INSERT INTO artikel (bezeichnung) VALUES ('   ')")
            .execute(&pool)
            .await;
        assert!(ergebnis.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn db_bild_und_bilddaten_nur_gemeinsam(pool: PgPool) {
        let ergebnis = sqlx::query(
            "INSERT INTO artikel (bezeichnung, bild, bild_daten) VALUES ('Schraube', 'a.jpg', NULL)",
        )
        .execute(&pool)
        .await;
        assert!(ergebnis.is_err());
    }
}
