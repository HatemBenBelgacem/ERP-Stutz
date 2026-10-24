use leptos::prelude::*;

use crate::components::gemeinsam::{ReiterStart, Reiterseite};
use crate::models::Bereich;

const REITER: [(Bereich, icondata::Icon); 3] = [
    (Bereich::Termine, icondata::LuCalendar),
    (Bereich::Kurse, icondata::LuBookOpen),
    (Bereich::Anmeldung, icondata::LuUserPlus),
];

/// Rahmen für die Weiterbildung mit Reitern für Termine, Kurse und Anmeldung.
#[component]
pub fn WeiterbildungSeite() -> impl IntoView {
    view! { <Reiterseite titel="Weiterbildung" untertitel="Kurse und ihre Durchführungen" reiter=&REITER/> }
}

/// `/personal/weiterbildung` ohne Reiter: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn WeiterbildungStart() -> impl IntoView {
    view! { <ReiterStart bereiche=&Bereich::WEITERBILDUNG/> }
}
