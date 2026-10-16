//! Datentypen, die zwischen Server und Browser ausgetauscht werden.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Kurs {
    pub id: Uuid,
    pub titel: String,
    pub beschreibung: String,
    pub dauer_stunden: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Mitarbeiter {
    pub id: Uuid,
    pub vorname: String,
    pub nachname: String,
    pub email: String,
    pub abteilung: String,
}

impl Mitarbeiter {
    pub fn name(&self) -> String {
        format!("{} {}", self.vorname, self.nachname)
    }
}

/// Termin inkl. zugehörigem Kurs (N:1) und Teilnehmenden (M:N).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Termin {
    pub id: Uuid,
    pub kurs_id: Uuid,
    pub kurs_titel: String,
    pub beginn: NaiveDateTime,
    pub ende: NaiveDateTime,
    pub ort: String,
    pub max_teilnehmer: i32,
    pub mitarbeiter: Vec<Mitarbeiter>,
}

/// Eingabedaten aus dem Termin-Formular. Datumswerte kommen im Format
/// von `<input type="datetime-local">` (z. B. `2026-10-16T09:00`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminEingabe {
    pub kurs_id: Uuid,
    pub beginn: String,
    pub ende: String,
    pub ort: String,
    pub max_teilnehmer: i32,
    pub mitarbeiter_ids: Vec<Uuid>,
}

/// Format für `<input type="datetime-local">`.
pub const DATETIME_LOCAL: &str = "%Y-%m-%dT%H:%M";

/// Benutzerkonto (ohne Passwort-Hash – der verlässt den Server nie),
/// inklusive der Rechte aus seiner Rolle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Benutzer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub aktiv: bool,
    pub erstellt_am: NaiveDateTime,
    pub rolle_id: Option<Uuid>,
    pub rolle_name: Option<String>,
    pub ist_admin: bool,
    /// Rechte der Rolle, z. B. `termine.bearbeiten`
    pub rechte: Vec<String>,
    /// Konto ist mit einem Microsoft-Konto verknüpft
    pub microsoft: bool,
}

impl Benutzer {
    /// Admins dürfen alles, alle anderen nur, was ihre Rolle erlaubt.
    pub fn darf(&self, bereich: Bereich, aktion: Aktion) -> bool {
        self.ist_admin || self.rechte.iter().any(|r| *r == recht(bereich, aktion))
    }

    pub fn sieht(&self, bereich: Bereich) -> bool {
        self.darf(bereich, Aktion::Sehen)
    }

    pub fn sieht_einstellungen(&self) -> bool {
        Bereich::EINSTELLUNGEN.into_iter().any(|b| self.sieht(b))
    }

    /// Erste Seite, die der Benutzer öffnen darf.
    pub fn startseite(&self) -> &'static str {
        Bereich::HAUPTBEREICHE
            .into_iter()
            .chain(Bereich::EINSTELLUNGEN)
            .find(|b| self.sieht(*b))
            .map(Bereich::pfad)
            .unwrap_or(PFAD_KEIN_ZUGRIFF)
    }
}

/// Rolle mit ihren Rechten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Rolle {
    pub id: Uuid,
    pub name: String,
    pub beschreibung: String,
    pub ist_admin: bool,
    pub rechte: Vec<String>,
    pub anzahl_benutzer: i64,
}

/// Bereiche der App. Die letzten beiden liegen unter „Einstellungen“.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bereich {
    Uebersicht,
    Termine,
    Kurse,
    Mitarbeiter,
    Benutzer,
    Rollen,
}

impl Bereich {
    pub const HAUPTBEREICHE: [Bereich; 4] = [Bereich::Uebersicht, Bereich::Termine, Bereich::Kurse, Bereich::Mitarbeiter];
    pub const EINSTELLUNGEN: [Bereich; 2] = [Bereich::Benutzer, Bereich::Rollen];
    pub const ALLE: [Bereich; 6] = [
        Bereich::Uebersicht,
        Bereich::Termine,
        Bereich::Kurse,
        Bereich::Mitarbeiter,
        Bereich::Benutzer,
        Bereich::Rollen,
    ];

    /// Wert in der Datenbank
    pub fn schluessel(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "uebersicht",
            Bereich::Termine => "termine",
            Bereich::Kurse => "kurse",
            Bereich::Mitarbeiter => "mitarbeiter",
            Bereich::Benutzer => "benutzer",
            Bereich::Rollen => "rollen",
        }
    }

    pub fn titel(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "Übersicht",
            Bereich::Termine => "Termine",
            Bereich::Kurse => "Kurse",
            Bereich::Mitarbeiter => "Mitarbeiter",
            Bereich::Benutzer => "Benutzer",
            Bereich::Rollen => "Rollen",
        }
    }

    pub fn pfad(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "/",
            Bereich::Termine => "/termine",
            Bereich::Kurse => "/kurse",
            Bereich::Mitarbeiter => "/mitarbeiter",
            Bereich::Benutzer => "/einstellungen/benutzer",
            Bereich::Rollen => "/einstellungen/rollen",
        }
    }

    /// Funktionen, die sich in diesem Bereich berechtigen lassen.
    pub fn aktionen(self) -> &'static [Aktion] {
        match self {
            Bereich::Uebersicht => &[Aktion::Sehen],
            _ => &Aktion::ALLE,
        }
    }
}

/// Funktionen, die pro Bereich berechtigt werden.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aktion {
    Sehen,
    Anlegen,
    Bearbeiten,
    Loeschen,
}

impl Aktion {
    pub const ALLE: [Aktion; 4] = [Aktion::Sehen, Aktion::Anlegen, Aktion::Bearbeiten, Aktion::Loeschen];

    pub fn schluessel(self) -> &'static str {
        match self {
            Aktion::Sehen => "sehen",
            Aktion::Anlegen => "anlegen",
            Aktion::Bearbeiten => "bearbeiten",
            Aktion::Loeschen => "loeschen",
        }
    }

    pub fn titel(self) -> &'static str {
        match self {
            Aktion::Sehen => "Sehen",
            Aktion::Anlegen => "Anlegen",
            Aktion::Bearbeiten => "Bearbeiten",
            Aktion::Loeschen => "Löschen",
        }
    }
}

/// Schlüssel eines Rechts, z. B. `termine.bearbeiten`.
pub fn recht(bereich: Bereich, aktion: Aktion) -> String {
    format!("{}.{}", bereich.schluessel(), aktion.schluessel())
}

/// Alle Rechte, die es gibt.
pub fn alle_rechte() -> Vec<String> {
    Bereich::ALLE
        .into_iter()
        .flat_map(|b| b.aktionen().iter().map(move |a| recht(b, *a)))
        .collect()
}

pub const PFAD_KEIN_ZUGRIFF: &str = "/kein-zugriff";
pub const PASSWORT_MIN_LAENGE: usize = 8;
