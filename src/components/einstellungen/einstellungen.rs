use leptos::prelude::*;

use crate::components::gemeinsam::{ReiterStart, Reiterseite};
use crate::models::Bereich;

const REITER: [(Bereich, icondata::Icon); 2] = [(Bereich::Benutzer, icondata::LuUsers), (Bereich::Rollen, icondata::LuShieldCheck)];

/// Rahmen für die Einstellungen mit Reitern für Benutzer und Rollen.
#[component]
pub fn EinstellungenSeite() -> impl IntoView {
    view! { <Reiterseite titel="Einstellungen" untertitel="Benutzer, Rollen und Berechtigungen" reiter=&REITER/> }
}

/// `/einstellungen` ohne Reiter: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn EinstellungenStart() -> impl IntoView {
    view! { <ReiterStart bereiche=&Bereich::EINSTELLUNGEN/> }
}
