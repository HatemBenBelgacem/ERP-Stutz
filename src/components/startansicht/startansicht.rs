use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use crate::app::{Sitzung, use_sitzung};
use crate::components::gemeinsam::{Laden, Leer};
use crate::navigation::{NAVIGATION, Ziel};

/// Startseite nach dem Login: eine Kachel je Modul, das der Benutzer sehen darf.
#[component]
pub fn Startansicht() -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <h1 class="mb-4 text-xl font-bold">"Module"</h1>
        <Transition>
            {move || match sitzung.angemeldet() {
                None => view! { <Laden/> }.into_any(),
                Some(false) => ().into_any(),
                Some(true) => {
                    let sichtbare = sichtbare_module(sitzung);
                    if sichtbare.is_empty() {
                        view! {
                            <Leer
                                symbol=icondata::LuLock
                                text="Noch keine Module freigegeben. Bitte an die Administration wenden."
                            />
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                                {sichtbare.into_iter().map(kachel).collect_view()}
                            </div>
                        }
                        .into_any()
                    }
                }
            }}
        </Transition>
    }
}

/// Alle Module in Menüreihenfolge, die der angemeldete Benutzer sehen darf.
fn sichtbare_module(sitzung: Sitzung) -> Vec<Ziel> {
    NAVIGATION
        .iter()
        .flat_map(|(_, eintraege)| eintraege.iter().copied())
        .filter(|ziel| ziel.sichtbar(|b| sitzung.sieht(b) == Some(true)))
        .collect()
}

/// Kachel mit Icon und Name; führt zum Modul.
fn kachel(ziel: Ziel) -> impl IntoView {
    view! {
        <A href=ziel.href attr:class="card card-border bg-base-100 shadow-sm transition hover:shadow-md">
            <div class="card-body flex-row items-center gap-4">
                <span class="rounded-box bg-primary/10 p-3 text-primary">
                    <Icon icon=ziel.icon width="1.5rem" height="1.5rem"/>
                </span>
                <span class="card-title">{ziel.text}</span>
            </div>
        </A>
    }
}
