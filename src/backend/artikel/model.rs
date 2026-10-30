//! Datentypen und Prüfregeln für Artikel. Die Prüfung läuft im Browser für
//! sofortige Meldungen und auf dem Server zur Sicherheit, mit demselben Code.

use std::collections::HashSet;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Höchstlänge einer Grösse in Zeichen
pub const GROESSE_MAX_ZEICHEN: usize = 20;

/// Artikel, wie er in der Liste steht (ohne Bilddaten)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artikel {
    pub id: Uuid,
    pub bezeichnung: String,
    /// Dateiname des Bildes, sofern eins hochgeladen ist
    pub bild: Option<String>,
    /// Grössen des Artikels in ihrer Reihenfolge
    pub groessen: Vec<String>,
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
    /// Grössen als Text, in der Eingabereihenfolge
    pub groessen: Vec<String>,
    pub bild: BildAenderung,
}

impl ArtikelEingabe {
    /// Grössen ohne Leerzeichen am Rand und ohne leere Einträge (GR-2)
    pub fn groessen_bereinigt(&self) -> Vec<String> {
        self.groessen
            .iter()
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect()
    }

    /// Fachliche Prüfung; läuft im Browser und auf dem Server.
    pub fn pruefen(&self) -> Result<(), String> {
        if self.bezeichnung.trim().is_empty() {
            return Err("Bitte eine Bezeichnung eingeben.".into());
        }
        let groessen = self.groessen_bereinigt();
        if groessen.is_empty() {
            return Err("Bitte mindestens eine Grösse eingeben.".into());
        }
        if groessen
            .iter()
            .any(|g| g.chars().count() > GROESSE_MAX_ZEICHEN)
        {
            return Err("Eine Grösse darf höchstens 20 Zeichen haben.".into());
        }
        let mut gesehen = HashSet::new();
        if !groessen.iter().all(|g| gesehen.insert(g.as_str())) {
            return Err("Eine Grösse ist doppelt angegeben.".into());
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

    fn eingabe(bezeichnung: &str, groessen: &[&str], bild: BildAenderung) -> ArtikelEingabe {
        ArtikelEingabe {
            bezeichnung: bezeichnung.into(),
            groessen: groessen.iter().map(|g| g.to_string()).collect(),
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
        let e = eingabe("   ", &["M"], BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Bitte eine Bezeichnung eingeben.".to_string())
        );
    }

    #[test]
    fn ohne_grosse_wird_abgelehnt() {
        let e = eingabe("Schraube", &[], BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Bitte mindestens eine Grösse eingeben.".to_string())
        );
    }

    #[test]
    fn leere_felder_zaehlen_nicht_als_grosse() {
        let e = eingabe("Schraube", &["  ", ""], BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Bitte mindestens eine Grösse eingeben.".to_string())
        );
    }

    #[test]
    fn mehrere_grossen_sind_erlaubt_und_werden_bereinigt() {
        let e = eingabe("Shirt", &[" S ", "", "M", "L"], BildAenderung::Behalten);
        assert_eq!(e.groessen_bereinigt(), ["S", "M", "L"]);
        assert_eq!(e.pruefen(), Ok(()));
    }

    #[test]
    fn doppelte_grosse_wird_abgelehnt() {
        let e = eingabe("Shirt", &["M", "L", "M"], BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Eine Grösse ist doppelt angegeben.".to_string())
        );
    }

    #[test]
    fn grosse_mit_mehr_als_zwanzig_zeichen_wird_abgelehnt() {
        let zu_lang = "x".repeat(GROESSE_MAX_ZEICHEN + 1);
        let e = eingabe("Shirt", &[zu_lang.as_str()], BildAenderung::Behalten);
        assert_eq!(
            e.pruefen(),
            Err("Eine Grösse darf höchstens 20 Zeichen haben.".to_string())
        );
    }

    #[test]
    fn artikel_ohne_bild_ist_gueltig() {
        let e = eingabe("Schraube", &["M"], BildAenderung::Entfernen);
        assert_eq!(e.pruefen(), Ok(()));
    }

    #[test]
    fn jpg_und_png_sind_erlaubt() {
        assert_eq!(
            eingabe("A", &["M"], upload("bild.jpg", JPG)).pruefen(),
            Ok(())
        );
        assert_eq!(
            eingabe("A", &["M"], upload("bild.PNG", PNG)).pruefen(),
            Ok(())
        );
    }

    #[test]
    fn gif_wird_abgelehnt() {
        let e = eingabe("A", &["M"], upload("bild.gif", JPG));
        assert_eq!(
            e.pruefen(),
            Err("Das Bild muss ein JPG oder PNG sein.".to_string())
        );
    }

    #[test]
    fn falscher_inhalt_wird_abgelehnt() {
        let e = eingabe("A", &["M"], upload("bild.jpg", b"hallo"));
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
            eingabe("A", &["M"], upload("bild.jpg", &daten)).pruefen(),
            Ok(())
        );
    }

    #[test]
    fn bild_ueber_zwei_mb_wird_abgelehnt() {
        let mut daten = JPG.to_vec();
        daten.resize(BILD_MAX_BYTES + 1, 0);
        let e = eingabe("A", &["M"], upload("bild.jpg", &daten));
        assert_eq!(
            e.pruefen(),
            Err("Das Bild ist grösser als 2 MB.".to_string())
        );
    }
}
