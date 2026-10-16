use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::{A, Redirect};

use crate::app::use_sitzung;
use crate::backend::auth::{Anmelden, Registrieren};
use crate::components::gemeinsam::{Fehlermeldung, ergebnis_beobachten};
use crate::models::PASSWORT_MIN_LAENGE;

/// Zentrierte Karte mit Logo für Anmelden und Registrieren. Wer schon
/// angemeldet ist, wird direkt zur Übersicht weitergeleitet.
#[component]
fn KontoRahmen(titel: &'static str, untertitel: &'static str, children: Children) -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <Transition>
            {move || (sitzung.angemeldet() == Some(true)).then(|| view! { <Redirect path="/"/> })}
        </Transition>
        <div class="flex min-h-screen items-center justify-center p-4">
            <div class="card w-full max-w-sm border border-base-300 bg-base-100 shadow-sm">
                <div class="card-body gap-4">
                    <div class="flex flex-col items-center gap-2 text-center">
                        <span class="rounded-box bg-primary p-3 text-primary-content">
                            <Icon icon=icondata::LuGraduationCap width="1.75rem" height="1.75rem"/>
                        </span>
                        <h1 class="text-xl font-bold">{titel}</h1>
                        <p class="text-sm text-base-content/60">{untertitel}</p>
                    </div>
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn AnmeldeSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let email = RwSignal::new(String::new());
    let passwort = RwSignal::new(String::new());
    let meldung = RwSignal::new(None::<String>);
    ergebnis_beobachten(sitzung.anmelden.value(), meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        sitzung.anmelden.dispatch(Anmelden {
            email: email.get_untracked(),
            passwort: passwort.get_untracked(),
        });
    };

    view! {
        <KontoRahmen titel="Anmelden" untertitel="Willkommen zurück bei der Weiterbildungsverwaltung">
            <form class="flex flex-col gap-3" on:submit=absenden>
                <Fehlermeldung meldung/>
                <label class="input w-full">
                    <Icon icon=icondata::LuMail attr:class="opacity-50"/>
                    <input type="email" class="grow" placeholder="E-Mail" autocomplete="email" required bind:value=email/>
                </label>
                <label class="input w-full">
                    <Icon icon=icondata::LuLock attr:class="opacity-50"/>
                    <input type="password" class="grow" placeholder="Passwort" autocomplete="current-password"
                        required bind:value=passwort/>
                </label>
                <button type="submit" class="btn btn-primary w-full" disabled=move || sitzung.anmelden.pending().get()>
                    <Icon icon=icondata::LuLogIn/>
                    "Anmelden"
                </button>
            </form>
            <p class="text-center text-sm">
                "Noch kein Konto? "
                <A href="/registrieren" attr:class="link link-primary">"Registrieren"</A>
            </p>
        </KontoRahmen>
    }
}

#[component]
pub fn RegistrierSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let passwort = RwSignal::new(String::new());
    let passwort_wiederholt = RwSignal::new(String::new());
    let meldung = RwSignal::new(None::<String>);
    ergebnis_beobachten(sitzung.registrieren.value(), meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        sitzung.registrieren.dispatch(Registrieren {
            name: name.get_untracked(),
            email: email.get_untracked(),
            passwort: passwort.get_untracked(),
            passwort_wiederholt: passwort_wiederholt.get_untracked(),
        });
    };

    view! {
        <KontoRahmen titel="Registrieren" untertitel="Neues Konto für die Weiterbildungsverwaltung">
            <form class="flex flex-col gap-3" on:submit=absenden>
                <Fehlermeldung meldung/>
                <label class="input w-full">
                    <Icon icon=icondata::LuUserRound attr:class="opacity-50"/>
                    <input class="grow" placeholder="Name" autocomplete="name" required bind:value=name/>
                </label>
                <label class="input w-full">
                    <Icon icon=icondata::LuMail attr:class="opacity-50"/>
                    <input type="email" class="grow" placeholder="E-Mail" autocomplete="email" required bind:value=email/>
                </label>
                <label class="input w-full">
                    <Icon icon=icondata::LuLock attr:class="opacity-50"/>
                    <input type="password" class="grow" placeholder="Passwort" autocomplete="new-password"
                        minlength=PASSWORT_MIN_LAENGE.to_string() required bind:value=passwort/>
                </label>
                <label class="input w-full">
                    <Icon icon=icondata::LuLock attr:class="opacity-50"/>
                    <input type="password" class="grow" placeholder="Passwort wiederholen" autocomplete="new-password"
                        required bind:value=passwort_wiederholt/>
                </label>
                <p class="text-xs text-base-content/60">
                    {format!("Mindestens {PASSWORT_MIN_LAENGE} Zeichen.")}
                </p>
                <button type="submit" class="btn btn-primary w-full" disabled=move || sitzung.registrieren.pending().get()>
                    <Icon icon=icondata::LuUserPlus/>
                    "Konto erstellen"
                </button>
            </form>
            <p class="text-center text-sm">
                "Schon registriert? "
                <A href="/anmelden" attr:class="link link-primary">"Anmelden"</A>
            </p>
        </KontoRahmen>
    }
}
