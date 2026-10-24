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
    /// Zugeordnete Geschäftsbereiche (M:N)
    pub geschaeftsbereich_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Geschaeftsbereich {
    pub id: Uuid,
    pub beschreibung: String,
}

/// Durchführungsort; eine Adresse kann zu vielen Terminen gehören (1:N).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Adresse {
    pub id: Uuid,
    pub bezeichnung: String,
    pub strasse: String,
    pub plz: String,
    pub ort: String,
}

impl Adresse {
    /// Einzeilige Anzeige, z. B. „Werkhof, Hauptstrasse 1, 8000 Zürich“.
    pub fn anzeige(&self) -> String {
        self.zeilen().join(", ")
    }

    /// Bezeichnung, Strasse und „PLZ Ort“ als eigene Zeilen (leere entfallen).
    pub fn zeilen(&self) -> Vec<String> {
        let plz_ort = format!("{} {}", self.plz, self.ort).trim().to_string();
        [self.bezeichnung.clone(), self.strasse.clone(), plz_ort]
            .into_iter()
            .filter(|t| !t.is_empty())
            .collect()
    }
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

/// Termin inkl. zugehörigem Kurs (N:1), Adresse (N:1) und Teilnehmenden (M:N).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Termin {
    pub id: Uuid,
    pub kurs_id: Uuid,
    pub kurs_titel: String,
    pub beginn: NaiveDateTime,
    pub ende: NaiveDateTime,
    pub adresse: Option<Adresse>,
    pub max_teilnehmer: i32,
    pub status: TerminStatus,
    pub mitarbeiter: Vec<Mitarbeiter>,
}

/// Bearbeitungsstand eines Termins, in der Reihenfolge des Ablaufs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminStatus {
    #[default]
    Neu,
    TeamchefsInformiert,
    Laufend,
    Beendet,
}

impl TerminStatus {
    pub const ALLE: [TerminStatus; 4] =
        [TerminStatus::Neu, TerminStatus::TeamchefsInformiert, TerminStatus::Laufend, TerminStatus::Beendet];

    /// Wert in der Datenbank (Typ `termin_status`)
    pub fn schluessel(self) -> &'static str {
        match self {
            TerminStatus::Neu => "neu",
            TerminStatus::TeamchefsInformiert => "teamchefs_informiert",
            TerminStatus::Laufend => "laufend",
            TerminStatus::Beendet => "beendet",
        }
    }

    pub fn aus_schluessel(schluessel: &str) -> Option<TerminStatus> {
        TerminStatus::ALLE.into_iter().find(|s| s.schluessel() == schluessel)
    }

    pub fn titel(self) -> &'static str {
        match self {
            TerminStatus::Neu => "Neu",
            TerminStatus::TeamchefsInformiert => "Teamchefs informiert",
            TerminStatus::Laufend => "Laufend",
            TerminStatus::Beendet => "Beendet",
        }
    }

    /// Farbe des Badges (Klassen ausgeschrieben, damit Tailwind sie findet)
    pub fn badge_klasse(self) -> &'static str {
        match self {
            TerminStatus::Neu => "badge-success",
            TerminStatus::TeamchefsInformiert => "badge-info",
            TerminStatus::Laufend => "badge-warning",
            TerminStatus::Beendet => "badge-error",
        }
    }

    /// Text- und Hintergrundfarbe für die Timeline
    pub fn text_klasse(self) -> &'static str {
        match self {
            TerminStatus::Neu => "text-success",
            TerminStatus::TeamchefsInformiert => "text-info",
            TerminStatus::Laufend => "text-warning",
            TerminStatus::Beendet => "text-error",
        }
    }

    pub fn bg_klasse(self) -> &'static str {
        match self {
            TerminStatus::Neu => "bg-success",
            TerminStatus::TeamchefsInformiert => "bg-info",
            TerminStatus::Laufend => "bg-warning",
            TerminStatus::Beendet => "bg-error",
        }
    }

    /// Position im Ablauf (0 = Neu)
    pub fn schritt(self) -> usize {
        TerminStatus::ALLE.iter().position(|s| *s == self).unwrap_or_default()
    }
}

/// Eingabedaten aus dem Termin-Formular. Datumswerte kommen im Format
/// von `<input type="datetime-local">` (z. B. `2026-10-16T09:00`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerminEingabe {
    pub kurs_id: Uuid,
    pub beginn: String,
    pub ende: String,
    pub adresse_id: Option<Uuid>,
    pub max_teilnehmer: i32,
    pub status: TerminStatus,
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

/// Bereiche der App. Benutzer und Rollen liegen unter „Einstellungen“.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bereich {
    Termine,
    Kurse,
    Anmeldung,
    Geschaeftsbereiche,
    Mitarbeiter,
    Adressen,
    Einkauf,
    Lager,
    Projekte,
    Werkhof,
    Finanzen,
    Benutzer,
    Rollen,
}

impl Bereich {
    /// Bereiche der Gruppe „Personal“
    pub const PERSONAL: [Bereich; 4] = [Bereich::Termine, Bereich::Kurse, Bereich::Anmeldung, Bereich::Mitarbeiter];
    /// Reiter unter „Personal › Weiterbildung“
    pub const WEITERBILDUNG: [Bereich; 3] = [Bereich::Termine, Bereich::Kurse, Bereich::Anmeldung];
    pub const HAUPTBEREICHE: [Bereich; 10] = [
        Bereich::Termine,
        Bereich::Kurse,
        Bereich::Anmeldung,
        Bereich::Mitarbeiter,
        Bereich::Adressen,
        Bereich::Einkauf,
        Bereich::Lager,
        Bereich::Projekte,
        Bereich::Werkhof,
        Bereich::Finanzen,
    ];
    pub const EINSTELLUNGEN: [Bereich; 3] = [Bereich::Benutzer, Bereich::Rollen, Bereich::Geschaeftsbereiche];
    pub const ALLE: [Bereich; 13] = [
        Bereich::Termine,
        Bereich::Kurse,
        Bereich::Anmeldung,
        Bereich::Mitarbeiter,
        Bereich::Adressen,
        Bereich::Einkauf,
        Bereich::Lager,
        Bereich::Projekte,
        Bereich::Werkhof,
        Bereich::Finanzen,
        Bereich::Benutzer,
        Bereich::Rollen,
        Bereich::Geschaeftsbereiche,
    ];

    /// Wert in der Datenbank
    pub fn schluessel(self) -> &'static str {
        match self {
            Bereich::Termine => "termine",
            Bereich::Kurse => "kurse",
            Bereich::Anmeldung => "anmeldung",
            Bereich::Geschaeftsbereiche => "geschaeftsbereiche",
            Bereich::Mitarbeiter => "mitarbeiter",
            Bereich::Adressen => "adressen",
            Bereich::Einkauf => "einkauf",
            Bereich::Lager => "lager",
            Bereich::Projekte => "projekte",
            Bereich::Werkhof => "werkhof",
            Bereich::Finanzen => "finanzen",
            Bereich::Benutzer => "benutzer",
            Bereich::Rollen => "rollen",
        }
    }

    pub fn titel(self) -> &'static str {
        match self {
            Bereich::Termine => "Termine",
            Bereich::Kurse => "Kurse",
            Bereich::Anmeldung => "Anmeldung",
            Bereich::Geschaeftsbereiche => "Geschäftsbereiche",
            Bereich::Mitarbeiter => "Mitarbeiter",
            Bereich::Adressen => "Adressen",
            Bereich::Einkauf => "Einkauf",
            Bereich::Lager => "Lager",
            Bereich::Projekte => "Projekte",
            Bereich::Werkhof => "Werkhof",
            Bereich::Finanzen => "Finanzen",
            Bereich::Benutzer => "Benutzer",
            Bereich::Rollen => "Rollen",
        }
    }

    pub fn pfad(self) -> &'static str {
        match self {
            Bereich::Termine => "/personal/weiterbildung/termine",
            Bereich::Kurse => "/personal/weiterbildung/kurse",
            Bereich::Anmeldung => "/personal/weiterbildung/anmeldung",
            Bereich::Geschaeftsbereiche => "/einstellungen/geschaeftsbereiche",
            Bereich::Mitarbeiter => "/personal/mitarbeiter",
            Bereich::Adressen => "/adressen",
            Bereich::Einkauf => "/einkauf",
            Bereich::Lager => "/lager",
            Bereich::Projekte => "/projekte",
            Bereich::Werkhof => "/werkhof",
            Bereich::Finanzen => "/finanzen",
            Bereich::Benutzer => "/einstellungen/benutzer",
            Bereich::Rollen => "/einstellungen/rollen",
        }
    }

    /// Gruppe in Navigation und Rechteverwaltung
    pub fn gruppe(self) -> Option<&'static str> {
        if Bereich::PERSONAL.contains(&self) {
            Some("Personal")
        } else if Bereich::EINSTELLUNGEN.contains(&self) {
            Some("Einstellungen")
        } else {
            None
        }
    }

    /// Funktionen, die sich in diesem Bereich berechtigen lassen.
    pub fn aktionen(self) -> &'static [Aktion] {
        match self {
            // Noch ohne Inhalte, daher nur sichtbar schalten
            Bereich::Einkauf | Bereich::Lager | Bereich::Projekte | Bereich::Werkhof | Bereich::Finanzen => {
                &[Aktion::Sehen]
            }
            // Bei Terminen lassen sich Teilnehmende auch ohne „Bearbeiten“ hinzufügen
            Bereich::Termine => &Aktion::ALLE,
            // Anmeldung: aktive Termine sehen und Mitarbeitende anmelden
            Bereich::Anmeldung => &[Aktion::Sehen, Aktion::TeilnehmendeHinzufuegen],
            _ => &Aktion::STANDARD,
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
    /// Nur Teilnehmende zu einem Termin hinzufügen (sonst nichts ändern)
    TeilnehmendeHinzufuegen,
}

impl Aktion {
    /// Funktionen, die es in jedem Bereich gibt
    pub const STANDARD: [Aktion; 4] = [Aktion::Sehen, Aktion::Anlegen, Aktion::Bearbeiten, Aktion::Loeschen];
    pub const ALLE: [Aktion; 5] =
        [Aktion::Sehen, Aktion::Anlegen, Aktion::Bearbeiten, Aktion::Loeschen, Aktion::TeilnehmendeHinzufuegen];

    pub fn schluessel(self) -> &'static str {
        match self {
            Aktion::Sehen => "sehen",
            Aktion::Anlegen => "anlegen",
            Aktion::Bearbeiten => "bearbeiten",
            Aktion::Loeschen => "loeschen",
            Aktion::TeilnehmendeHinzufuegen => "teilnehmende_hinzufuegen",
        }
    }

    pub fn titel(self) -> &'static str {
        match self {
            Aktion::Sehen => "Sehen",
            Aktion::Anlegen => "Anlegen",
            Aktion::Bearbeiten => "Bearbeiten",
            Aktion::Loeschen => "Löschen",
            Aktion::TeilnehmendeHinzufuegen => "Teilnehmende hinzufügen",
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
