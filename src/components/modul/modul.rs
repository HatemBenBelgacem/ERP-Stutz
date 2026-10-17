use leptos::prelude::*;

use crate::components::gemeinsam::{Leer, Seitenkopf};
use crate::models::Bereich;

/// Platzhalter für Module, die noch keine Funktionen haben (Einkauf, Lager, Projekte, Werkhof, Finanzen).
#[component]
pub fn ModulSeite(bereich: Bereich) -> impl IntoView {
    let (untertitel, symbol) = match bereich {
        Bereich::Einkauf => ("Bestellungen, Lieferanten und Material", icondata::LuShoppingCart),
        Bereich::Lager => ("Bestände, Lagerorte und Materialbewegungen", icondata::LuWarehouse),
        Bereich::Projekte => ("Baustellen und Projekte im Überblick", icondata::LuConstruction),
        Bereich::Werkhof => ("Fahrzeuge, Maschinen und Geräte", icondata::LuTruck),
        _ => ("Rechnungen, Budgets und Zahlungen", icondata::LuWallet),
    };
    view! {
        <Seitenkopf titel=bereich.titel() untertitel>
            {()}
        </Seitenkopf>
        <div class="card border border-base-300 bg-base-100">
            <Leer symbol text="Dieses Modul ist in Vorbereitung."/>
        </div>
    }
}
