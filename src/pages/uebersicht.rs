use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

use crate::api::statistik;
use crate::components::{Laden, Seitenkopf, meldung_aus};

#[component]
pub fn Uebersicht() -> impl IntoView {
    let zahlen = Resource::new(|| (), |_| statistik());

    view! {
        <Seitenkopf titel="Übersicht" untertitel="Stand der Weiterbildungen auf einen Blick">
            {()}
        </Seitenkopf>
        <Suspense fallback=Laden>
            {move || Suspend::new(async move {
                match zahlen.await {
                    Ok((kurse, mitarbeiter, termine)) => view! {
                        <div class="grid gap-4 sm:grid-cols-3">
                            <Kachel titel="Termine" wert=termine href="/termine" symbol=icondata::LuCalendar/>
                            <Kachel titel="Kurse" wert=kurse href="/kurse" symbol=icondata::LuBookOpen/>
                            <Kachel titel="Mitarbeiter" wert=mitarbeiter href="/mitarbeiter" symbol=icondata::LuUsers/>
                        </div>
                    }
                    .into_any(),
                    Err(e) => view! { <div class="alert alert-error">{meldung_aus(e)}</div> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn Kachel(titel: &'static str, wert: i64, href: &'static str, symbol: icondata::Icon) -> impl IntoView {
    view! {
        <div class="card border border-base-300 bg-base-100">
            <div class="card-body">
                <div class="flex items-center justify-between">
                    <span class="text-sm text-base-content/60">{titel}</span>
                    <span class="rounded-field bg-primary/10 p-2 text-primary">
                        <Icon icon=symbol width="1.25rem" height="1.25rem"/>
                    </span>
                </div>
                <div class="text-3xl font-bold">{wert}</div>
                <div class="card-actions justify-end">
                    <A href=href attr:class="btn btn-sm btn-ghost text-primary">
                        "Verwalten"
                        <Icon icon=icondata::LuArrowRight/>
                    </A>
                </div>
            </div>
        </div>
    }
}
