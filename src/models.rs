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

/// Benutzerkonto (ohne Passwort-Hash – der verlässt den Server nie).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Benutzer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub rolle: String,
    pub aktiv: bool,
    pub erstellt_am: NaiveDateTime,
    /// Schlüssel der freigegebenen Bereiche, siehe [`Bereich`]
    pub berechtigungen: Vec<String>,
}

impl Benutzer {
    pub fn ist_admin(&self) -> bool {
        self.rolle == ROLLE_ADMIN
    }

    /// Admins dürfen alles, alle anderen nur ihre freigegebenen Bereiche.
    pub fn darf(&self, bereich: Bereich) -> bool {
        self.ist_admin() || self.berechtigungen.iter().any(|b| b == bereich.schluessel())
    }

    /// Erste Seite, die der Benutzer öffnen darf.
    pub fn startseite(&self) -> &'static str {
        Bereich::ALLE
            .into_iter()
            .find(|b| self.darf(*b))
            .map(Bereich::pfad)
            .unwrap_or(PFAD_KEIN_ZUGRIFF)
    }
}

/// Bereiche der App, die sich pro Benutzer freigeben lassen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bereich {
    Uebersicht,
    Termine,
    Kurse,
    Mitarbeiter,
}

impl Bereich {
    pub const ALLE: [Bereich; 4] = [Bereich::Uebersicht, Bereich::Termine, Bereich::Kurse, Bereich::Mitarbeiter];

    /// Wert in der Datenbank
    pub fn schluessel(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "uebersicht",
            Bereich::Termine => "termine",
            Bereich::Kurse => "kurse",
            Bereich::Mitarbeiter => "mitarbeiter",
        }
    }

    pub fn titel(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "Übersicht",
            Bereich::Termine => "Termine",
            Bereich::Kurse => "Kurse",
            Bereich::Mitarbeiter => "Mitarbeiter",
        }
    }

    pub fn pfad(self) -> &'static str {
        match self {
            Bereich::Uebersicht => "/",
            Bereich::Termine => "/termine",
            Bereich::Kurse => "/kurse",
            Bereich::Mitarbeiter => "/mitarbeiter",
        }
    }

    pub fn aus_schluessel(schluessel: &str) -> Option<Bereich> {
        Bereich::ALLE.into_iter().find(|b| b.schluessel() == schluessel)
    }
}

pub const PFAD_KEIN_ZUGRIFF: &str = "/kein-zugriff";

pub const ROLLE_ADMIN: &str = "admin";
pub const ROLLE_BENUTZER: &str = "benutzer";
pub const PASSWORT_MIN_LAENGE: usize = 8;
