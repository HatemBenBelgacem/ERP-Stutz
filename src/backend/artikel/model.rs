//! Datentypen und Prüfregeln für Artikel. Die Prüfung läuft im Browser für
//! sofortige Meldungen und auf dem Server zur Sicherheit, mit demselben Code.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kleidergrösse, z. B. „M“ (Tabelle `kleidergroesse`)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Kleidergroesse {
    pub id: Uuid,
    pub bezeichnung: String,
}

/// Artikel, wie er in der Liste steht (ohne Bilddaten)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artikel {
    pub id: Uuid,
    pub bezeichnung: String,
    /// Dateiname des Bildes, sofern eins hochgeladen ist
    pub bild: Option<String>,
    pub kleidergroesse_id: Uuid,
    /// Bezeichnung der Kleidergrösse
    pub groesse: String,
    pub erstellt_am: NaiveDateTime,
    pub geaendert_am: NaiveDateTime,
}

/// Höchstgrösse eines Bildes: 2 MB
pub const BILD_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Ein Bild aus dem Dateidialog
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BildUpload {
    pub name: String,
    pub daten: Vec<u8>,
}

impl BildUpload {
    pub fn pruefen(&self) -> Result<(), String> {
        pruefe_bildtyp(&self.name, &self.daten)?;
        if self.daten.len() > BILD_MAX_BYTES {
            return Err("Das Bild ist grösser als 2 MB.".into());
        }
        Ok(())
    }
}

/// Was beim Speichern mit dem Bild geschehen soll
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum BildAenderung {
    /// Bild bleibt, wie es ist
    #[default]
    Behalten,
    Neu(BildUpload),
    Entfernen,
}

/// Eingaben aus dem Dialog „Neuer Artikel“ bzw. „Artikel bearbeiten“
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ArtikelEingabe {
    pub bezeichnung: String,
    pub kleidergroesse_id: Option<Uuid>,
    pub bild: BildAenderung,
}

impl ArtikelEingabe {
    /// Fachliche Prüfung; läuft im Browser und auf dem Server.
    pub fn pruefen(&self) -> Result<(), String> {
        if self.bezeichnung.trim().is_empty() {
            return Err("Bitte eine Bezeichnung eingeben.".into());
        }
        if self.kleidergroesse_id.is_none() {
            return Err("Bitte eine Grösse wählen.".into());
        }
        if let BildAenderung::Neu(bild) = &self.bild {
            bild.pruefen()?;
        }
        Ok(())
    }
}

/// Erlaubt sind JPG und PNG. Geprüft werden Endung und Dateikopf.
pub fn pruefe_bildtyp(name: &str, daten: &[u8]) -> Result<(), String> {
    let endung = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    let passt = matches!(
        (endung.as_deref(), daten),
        (Some("jpg" | "jpeg"), [0xFF, 0xD8, 0xFF, ..])
            | (
                Some("png"),
                [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..]
            )
    );
    if passt {
        Ok(())
    } else {
        Err("Das Bild muss ein JPG oder PNG sein.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00];

    fn eingabe(
        bezeichnung: &str,
        kleidergroesse_id: Option<Uuid>,
        bild: BildAenderung,
    ) -> ArtikelEingabe {
        ArtikelEingabe {
            bezeichnung: bezeichnung.into(),
            kleidergroesse_id,
            bild,
        }
    }

    fn upload(name: &str, daten: &[u8]) -> BildAenderung {
        BildAenderung::Neu(BildUpload {
            name: name.into(),
            daten: daten.to_vec(),
        })
    }

    #[test]
    fn leere_bezeichnung_wird_abgelehnt() {
        let e = eingabe("   ", Some(Uuid::nil()), BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Bitte eine Bezeichnung eingeben.".to_string())
        );
    }

    #[test]
    fn fehlende_groesse_wird_abgelehnt() {
        let e = eingabe("Schraube", None, BildAenderung::Behalten);
        assert_eq!(e.pruefen(), Err("Bitte eine Grösse wählen.".to_string()));
    }

    #[test]
    fn artikel_ohne_bild_ist_gueltig() {
        let e = eingabe("Schraube", Some(Uuid::nil()), BildAenderung::Entfernen);
        assert_eq!(e.pruefen(), Ok(()));
    }

    #[test]
    fn jpg_und_png_sind_erlaubt() {
        assert_eq!(
            eingabe("A", Some(Uuid::nil()), upload("bild.jpg", JPG)).pruefen(),
            Ok(())
        );
        assert_eq!(
            eingabe("A", Some(Uuid::nil()), upload("bild.PNG", PNG)).pruefen(),
            Ok(())
        );
    }

    #[test]
    fn gif_wird_abgelehnt() {
        let e = eingabe("A", Some(Uuid::nil()), upload("bild.gif", JPG));
        assert_eq!(
            e.pruefen(),
            Err("Das Bild muss ein JPG oder PNG sein.".to_string())
        );
    }

    #[test]
    fn falscher_inhalt_wird_abgelehnt() {
        let e = eingabe("A", Some(Uuid::nil()), upload("bild.jpg", b"hallo"));
        assert_eq!(
            e.pruefen(),
            Err("Das Bild muss ein JPG oder PNG sein.".to_string())
        );
    }

    #[test]
    fn bild_mit_genau_zwei_mb_ist_erlaubt() {
        let mut daten = JPG.to_vec();
        daten.resize(BILD_MAX_BYTES, 0);
        assert_eq!(
            eingabe("A", Some(Uuid::nil()), upload("bild.jpg", &daten)).pruefen(),
            Ok(())
        );
    }

    #[test]
    fn bild_ueber_zwei_mb_wird_abgelehnt() {
        let mut daten = JPG.to_vec();
        daten.resize(BILD_MAX_BYTES + 1, 0);
        let e = eingabe("A", Some(Uuid::nil()), upload("bild.jpg", &daten));
        assert_eq!(
            e.pruefen(),
            Err("Das Bild ist grösser als 2 MB.".to_string())
        );
    }
}
