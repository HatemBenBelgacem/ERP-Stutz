use leptos::prelude::*;

use crate::components::gemeinsam::{ReiterStart, Reiterseite};
use crate::models::Bereich;

const REITER: [(Bereich, icondata::Icon); 3] = [
    (Bereich::Benutzer, icondata::LuUsers),
    (Bereich::Rollen, icondata::LuShieldCheck),
    (Bereich::Geschaeftsbereiche, icondata::LuBuilding2),
];

/// Rahmen für die Einstellungen mit Reitern für Benutzer, Rollen und Geschäftsbereiche.
#[component]
pub fn EinstellungenSeite() -> impl IntoView {
    view! { <Reiterseite titel="Einstellungen" untertitel="Benutzer, Rollen, Berechtigungen und Geschäftsbereiche" reiter=&REITER/> }
}

/// `/einstellungen` ohne Reiter: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn EinstellungenStart() -> impl IntoView {
    view! { <ReiterStart bereiche=&Bereich::EINSTELLUNGEN/> }
}
