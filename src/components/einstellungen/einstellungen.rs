use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::{A, Outlet, Redirect};

use crate::app::use_sitzung;
use crate::models::Bereich;

/// Rahmen für die Einstellungen mit Reitern für Benutzer und Rollen.
#[component]
pub fn EinstellungenSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let reiter = [(Bereich::Benutzer, icondata::LuUsers), (Bereich::Rollen, icondata::LuShieldCheck)];

    view! {
        <div class="mb-4">
            <h1 class="text-xl font-bold sm:text-2xl">"Einstellungen"</h1>
            <p class="text-sm text-base-content/60">"Benutzer, Rollen und Berechtigungen"</p>
        </div>
        <Transition>
            <div role="tablist" class="tabs tabs-border mb-6">
                {reiter
                    .into_iter()
                    .map(|(bereich, icon)| view! {
                        <Show when=move || sitzung.sieht(bereich) == Some(true)>
                            <A
                                href=bereich.pfad()
                                attr:role="tab"
                                attr:class="tab gap-2 aria-[current=page]:tab-active"
                            >
                                <Icon icon/>
                                {bereich.titel()}
                            </A>
                        </Show>
                    })
                    .collect_view()}
            </div>
        </Transition>
        <Outlet/>
    }
}

/// `/einstellungen` ohne Reiter: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn EinstellungenStart() -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <Transition>
            {move || sitzung.benutzer().map(|b| {
                let ziel = Bereich::EINSTELLUNGEN
                    .into_iter()
                    .find(|e| b.sieht(*e))
                    .map(Bereich::pfad)
                    .unwrap_or_else(|| b.startseite());
                view! { <Redirect path=ziel/> }
            })}
        </Transition>
    }
}
