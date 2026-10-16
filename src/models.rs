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
}

impl Benutzer {
    pub fn ist_admin(&self) -> bool {
        self.rolle == ROLLE_ADMIN
    }
}

pub const ROLLE_ADMIN: &str = "admin";
pub const ROLLE_BENUTZER: &str = "benutzer";
pub const PASSWORT_MIN_LAENGE: usize = 8;
