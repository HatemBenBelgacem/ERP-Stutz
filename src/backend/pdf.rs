//! PDF-Dokumente: Teilnehmerliste pro Termin.
//!
//! Die PDF-Datei wird direkt geschrieben (ohne zusätzliche Bibliothek). Als
//! Schrift dienen die eingebauten Helvetica-Schnitte mit WinAnsi-Kodierung,
//! damit funktionieren Umlaute ohne eingebettete Schriftdatei.

use axum::extract::Path;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::backend::auth::server::benutzer_aus_headers;
use crate::backend::db::pool;
use crate::models::{Aktion, Bereich};

// A4 in Punkt
const BREITE: f32 = 595.28;
const HOEHE: f32 = 841.89;
const RAND: f32 = 50.0;

// ------------------------------------------------------------ Route

/// `GET /termine/{id}/teilnehmerliste.pdf`
pub async fn teilnehmerliste(headers: HeaderMap, Path(id): Path<Uuid>) -> Response {
    let benutzer = match benutzer_aus_headers(&headers).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::FOUND, [(LOCATION, "/anmelden")]).into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if !benutzer.darf(Bereich::Termine, Aktion::Sehen) {
        return (StatusCode::FORBIDDEN, "Für diese Funktion fehlt dir die Berechtigung.").into_response();
    }

    let daten = match termin_laden(id).await {
        Ok(Some(d)) => d,
        Ok(None) => return (StatusCode::NOT_FOUND, "Diesen Termin gibt es nicht.").into_response(),
        Err(e) => {
            leptos::logging::error!("PDF: Termin konnte nicht geladen werden: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let dateiname = format!(
        "Teilnehmerliste-{}-{}.pdf",
        dateiname_teil(&daten.kurs_titel),
        daten.beginn.format("%Y-%m-%d")
    );
    let pdf = teilnehmerliste_pdf(&daten);
    (
        [
            (CONTENT_TYPE, "application/pdf".to_string()),
            (CONTENT_DISPOSITION, format!("inline; filename=\"{dateiname}\"")),
        ],
        pdf,
    )
        .into_response()
}

struct TerminDaten {
    kurs_titel: String,
    kurs_beschreibung: String,
    dauer_stunden: i32,
    beginn: NaiveDateTime,
    ende: NaiveDateTime,
    ort: String,
    max_teilnehmer: i32,
    /// (Name, Abteilung, E-Mail)
    teilnehmende: Vec<(String, String, String)>,
}

async fn termin_laden(id: Uuid) -> Result<Option<TerminDaten>, sqlx::Error> {
    let kopf: Option<(String, String, i32, NaiveDateTime, NaiveDateTime, String, i32)> = sqlx::query_as(
        "SELECT k.titel, k.beschreibung, k.dauer_stunden, t.beginn, t.ende, t.ort, t.max_teilnehmer
         FROM termin t JOIN kurs k ON k.id = t.kurs_id WHERE t.id = $1",
    )
    .bind(id)
    .fetch_optional(pool())
    .await?;
    let Some((kurs_titel, kurs_beschreibung, dauer_stunden, beginn, ende, ort, max_teilnehmer)) = kopf else {
        return Ok(None);
    };
    let zeilen: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT m.vorname, m.nachname, m.abteilung, m.email
         FROM termin_mitarbeiter tm JOIN mitarbeiter m ON m.id = tm.mitarbeiter_id
         WHERE tm.termin_id = $1 ORDER BY m.nachname, m.vorname",
    )
    .bind(id)
    .fetch_all(pool())
    .await?;
    Ok(Some(TerminDaten {
        kurs_titel,
        kurs_beschreibung,
        dauer_stunden,
        beginn,
        ende,
        ort,
        max_teilnehmer,
        teilnehmende: zeilen
            .into_iter()
            .map(|(vorname, nachname, abteilung, email)| (format!("{nachname}, {vorname}"), abteilung, email))
            .collect(),
    }))
}

/// Nur Buchstaben, Ziffern und Bindestriche im Dateinamen.
fn dateiname_teil(text: &str) -> String {
    let ersetzt: String = text
        .chars()
        .map(|c| match c {
            'ä' => "ae".to_string(),
            'ö' => "oe".to_string(),
            'ü' => "ue".to_string(),
            'Ä' => "Ae".to_string(),
            'Ö' => "Oe".to_string(),
            'Ü' => "Ue".to_string(),
            'ß' => "ss".to_string(),
            c if c.is_ascii_alphanumeric() => c.to_string(),
            _ => "-".to_string(),
        })
        .collect();
    ersetzt.split('-').filter(|t| !t.is_empty()).collect::<Vec<_>>().join("-")
}

// ------------------------------------------------------- Teilnehmerliste

/// Tabellenspalten: (Überschrift, linker Rand, Breite)
const SPALTEN: [(&str, f32, f32); 5] = [
    ("Nr.", RAND, 28.0),
    ("Name", RAND + 28.0, 140.0),
    ("Abteilung", RAND + 168.0, 85.0),
    ("E-Mail", RAND + 253.0, 147.0),
    ("Unterschrift", RAND + 400.0, BREITE - 2.0 * RAND - 400.0),
];
const ZEILENHOEHE: f32 = 24.0;

fn teilnehmerliste_pdf(d: &TerminDaten) -> Vec<u8> {
    let mut doc = Dokument::new(format!("Teilnehmerliste {}", d.kurs_titel));
    let mut y = HOEHE - RAND;

    // Kopf
    doc.text(RAND, y, 9.0, Schrift::Normal, Grau::Mittel, "ERP-STUTZ AG");
    y -= 24.0;
    doc.text(RAND, y, 20.0, Schrift::Fett, Grau::Schwarz, "Teilnehmerliste");
    y -= 26.0;
    doc.text(RAND, y, 14.0, Schrift::Fett, Grau::Schwarz, &d.kurs_titel);
    y -= 8.0;
    for zeile in umbrechen(&d.kurs_beschreibung, 10.0, BREITE - 2.0 * RAND) {
        y -= 14.0;
        doc.text(RAND, y, 10.0, Schrift::Normal, Grau::Dunkel, &zeile);
    }
    y -= 22.0;

    // Angaben zum Termin in zwei Spalten
    let datum = if d.beginn.date() == d.ende.date() {
        d.beginn.format("%d.%m.%Y").to_string()
    } else {
        format!("{} – {}", d.beginn.format("%d.%m.%Y"), d.ende.format("%d.%m.%Y"))
    };
    let angaben = [
        ("Datum", datum),
        ("Zeit", format!("{} – {} Uhr", d.beginn.format("%H:%M"), d.ende.format("%H:%M"))),
        ("Ort", if d.ort.is_empty() { "–".to_string() } else { d.ort.clone() }),
        ("Dauer", format!("{} Stunden", d.dauer_stunden)),
        ("Belegung", format!("{} von {} Plätzen", d.teilnehmende.len(), d.max_teilnehmer)),
    ];
    let spalte_breite = (BREITE - 2.0 * RAND) / 2.0;
    for (i, (bezeichnung, wert)) in angaben.iter().enumerate() {
        let x = RAND + (i % 2) as f32 * spalte_breite;
        let zeile_y = y - (i / 2) as f32 * 18.0;
        doc.text(x, zeile_y, 9.0, Schrift::Fett, Grau::Mittel, bezeichnung);
        doc.text(x + 60.0, zeile_y, 10.0, Schrift::Normal, Grau::Schwarz, &kuerzen(wert, 10.0, spalte_breite - 70.0));
    }
    y -= ((angaben.len() + 1) / 2) as f32 * 18.0 + 16.0;

    // Tabelle der Teilnehmenden, bei Bedarf über mehrere Seiten
    y = tabellenkopf(&mut doc, y);
    if d.teilnehmende.is_empty() {
        doc.text(RAND + 6.0, y - 16.0, 10.0, Schrift::Normal, Grau::Mittel, "Noch keine Teilnehmenden.");
    }
    for (nr, (name, abteilung, email)) in d.teilnehmende.iter().enumerate() {
        if y - ZEILENHOEHE < RAND + 30.0 {
            doc.neue_seite();
            y = tabellenkopf(&mut doc, HOEHE - RAND);
        }
        let text_y = y - 15.5;
        let werte = [(nr + 1).to_string(), name.clone(), abteilung.clone(), email.clone()];
        for ((_, x, breite), wert) in SPALTEN.iter().zip(werte.iter()) {
            doc.text(x + 4.0, text_y, 9.5, Schrift::Normal, Grau::Schwarz, &kuerzen(wert, 9.5, breite - 8.0));
        }
        y -= ZEILENHOEHE;
        doc.linie(RAND, y, BREITE - RAND, y, 0.5, Grau::Hell);
    }

    let erstellt = chrono::Local::now().format("%d.%m.%Y %H:%M").to_string();
    doc.fertig(&format!("Erstellt am {erstellt}"))
}

fn tabellenkopf(doc: &mut Dokument, y: f32) -> f32 {
    let hoehe = 20.0;
    doc.flaeche(RAND, y - hoehe, BREITE - 2.0 * RAND, hoehe, 0.93);
    for (titel, x, _) in SPALTEN {
        doc.text(x + 4.0, y - 13.5, 9.0, Schrift::Fett, Grau::Dunkel, titel);
    }
    y - hoehe
}

// -------------------------------------------------- einfacher PDF-Schreiber

#[derive(Clone, Copy)]
enum Schrift {
    Normal,
    Fett,
}

#[derive(Clone, Copy)]
enum Grau {
    Schwarz,
    Dunkel,
    Mittel,
    Hell,
}

impl Grau {
    fn wert(self) -> f32 {
        match self {
            Grau::Schwarz => 0.0,
            Grau::Dunkel => 0.25,
            Grau::Mittel => 0.45,
            Grau::Hell => 0.82,
        }
    }
}

struct Dokument {
    titel: String,
    seiten: Vec<Vec<u8>>,
    ziel: Option<usize>,
}

impl Dokument {
    fn new(titel: String) -> Self {
        Dokument { titel, seiten: vec![Vec::new()], ziel: None }
    }

    fn neue_seite(&mut self) {
        self.seiten.push(Vec::new());
    }

    /// Befehle gehen an die zuletzt angelegte Seite, außer `ziel` ist gesetzt.
    fn seite(&mut self) -> &mut Vec<u8> {
        let nr = self.ziel.unwrap_or(self.seiten.len() - 1);
        &mut self.seiten[nr]
    }

    fn text(&mut self, x: f32, y: f32, groesse: f32, schrift: Schrift, farbe: Grau, text: &str) {
        let name = match schrift {
            Schrift::Normal => "F1",
            Schrift::Fett => "F2",
        };
        let seite = self.seite();
        seite.extend(format!("BT {} g /{name} {groesse:.1} Tf {x:.2} {y:.2} Td (", farbe.wert()).as_bytes());
        seite.extend(pdf_text(text));
        seite.extend(b") Tj ET\n");
    }

    fn linie(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, breite: f32, farbe: Grau) {
        let befehl = format!("q {} G {breite:.2} w {x1:.2} {y1:.2} m {x2:.2} {y2:.2} l S Q\n", farbe.wert());
        self.seite().extend(befehl.as_bytes());
    }

    fn flaeche(&mut self, x: f32, y: f32, breite: f32, hoehe: f32, grau: f32) {
        let befehl = format!("q {grau:.2} g {x:.2} {y:.2} {breite:.2} {hoehe:.2} re f Q\n");
        self.seite().extend(befehl.as_bytes());
    }

    /// Fusszeile auf jede Seite setzen und die PDF-Datei zusammenbauen.
    fn fertig(mut self, fuss_links: &str) -> Vec<u8> {
        let anzahl = self.seiten.len();
        let y = RAND - 20.0;
        for nr in 0..anzahl {
            self.ziel = Some(nr);
            let rechts = format!("Seite {} von {anzahl}", nr + 1);
            self.linie(RAND, y + 12.0, BREITE - RAND, y + 12.0, 0.5, Grau::Hell);
            self.text(RAND, y, 8.0, Schrift::Normal, Grau::Mittel, fuss_links);
            self.text(BREITE - RAND - textbreite(&rechts, 8.0), y, 8.0, Schrift::Normal, Grau::Mittel, &rechts);
        }

        let mut pdf: Vec<u8> = Vec::new();
        let mut versatz: Vec<usize> = Vec::new();
        pdf.extend(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

        let mut objekt = |pdf: &mut Vec<u8>, inhalt: &[u8]| {
            versatz.push(pdf.len());
            pdf.extend(format!("{} 0 obj\n", versatz.len()).as_bytes());
            pdf.extend(inhalt);
            pdf.extend(b"\nendobj\n");
        };

        // 1 Katalog, 2 Seitenbaum, 3/4 Schriften, 5 Info, danach je Seite: Seite + Inhalt
        let kinder: Vec<String> = (0..anzahl).map(|i| format!("{} 0 R", 6 + 2 * i)).collect();
        objekt(&mut pdf, b"<< /Type /Catalog /Pages 2 0 R >>");
        objekt(&mut pdf, format!("<< /Type /Pages /Kids [{}] /Count {anzahl} >>", kinder.join(" ")).as_bytes());
        objekt(&mut pdf, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
        objekt(&mut pdf, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>");
        let mut info = b"<< /Producer (ERP-Stutz AG) /Title (".to_vec();
        info.extend(pdf_text(&self.titel));
        info.extend(b") >>");
        objekt(&mut pdf, &info);

        for (i, inhalt) in self.seiten.iter().enumerate() {
            let inhalt_nr = 7 + 2 * i;
            objekt(
                &mut pdf,
                format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {BREITE} {HOEHE}] \
                     /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {inhalt_nr} 0 R >>"
                )
                .as_bytes(),
            );
            let mut strom = format!("<< /Length {} >>\nstream\n", inhalt.len()).into_bytes();
            strom.extend(inhalt);
            strom.extend(b"\nendstream");
            objekt(&mut pdf, &strom);
        }

        let xref = pdf.len();
        pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", versatz.len() + 1).as_bytes());
        for v in &versatz {
            pdf.extend(format!("{v:010} 00000 n \n").as_bytes());
        }
        pdf.extend(
            format!("trailer\n<< /Size {} /Root 1 0 R /Info 5 0 R >>\nstartxref\n{xref}\n%%EOF\n", versatz.len() + 1)
                .as_bytes(),
        );
        pdf
    }
}

/// Text in WinAnsi kodieren und für PDF-Strings maskieren.
fn pdf_text(text: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(text.len());
    for c in text.chars() {
        let b = match c {
            '(' | ')' | '\\' => {
                bytes.push(b'\\');
                c as u8
            }
            '€' => 0x80,
            '‚' => 0x82,
            '„' => 0x84,
            '…' => 0x85,
            '‘' => 0x91,
            '’' => 0x92,
            '“' => 0x93,
            '”' => 0x94,
            '•' => 0x95,
            '–' => 0x96,
            '—' => 0x97,
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => c as u8,
            c if (c as u32) >= 0xA0 && (c as u32) <= 0xFF => c as u32 as u8,
            _ => b'?',
        };
        bytes.push(b);
    }
    bytes
}

/// Geschätzte Breite in Punkt (Helvetica: im Mittel gut eine halbe Schriftgrösse pro Zeichen).
fn textbreite(text: &str, groesse: f32) -> f32 {
    text.chars().count() as f32 * groesse * 0.53
}

/// Kürzt Text mit „…“, damit er in die Breite passt.
fn kuerzen(text: &str, groesse: f32, breite: f32) -> String {
    if textbreite(text, groesse) <= breite {
        return text.to_string();
    }
    let max = ((breite / (groesse * 0.53)) as usize).saturating_sub(1);
    let gekuerzt: String = text.chars().take(max).collect();
    format!("{}…", gekuerzt.trim_end())
}

/// Bricht Text an Wortgrenzen in Zeilen um (höchstens 4 Zeilen).
fn umbrechen(text: &str, groesse: f32, breite: f32) -> Vec<String> {
    let mut zeilen: Vec<String> = Vec::new();
    let mut aktuell = String::new();
    for wort in text.split_whitespace() {
        let kandidat = if aktuell.is_empty() { wort.to_string() } else { format!("{aktuell} {wort}") };
        if textbreite(&kandidat, groesse) > breite && !aktuell.is_empty() {
            zeilen.push(std::mem::replace(&mut aktuell, wort.to_string()));
        } else {
            aktuell = kandidat;
        }
    }
    if !aktuell.is_empty() {
        zeilen.push(aktuell);
    }
    if zeilen.len() > 4 {
        zeilen.truncate(4);
        if let Some(letzte) = zeilen.last_mut() {
            *letzte = kuerzen(&format!("{letzte} …"), groesse, breite);
        }
    }
    zeilen
}
