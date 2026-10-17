//! Wiederverwendbare UI-Bausteine und Hilfsfunktionen.

use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::{A, Outlet, Redirect};

use crate::app::use_sitzung;
use crate::models::Bereich;

// ----------------------------------------------------------------- Bausteine

/// Seitentitel mit Untertitel und Platz für Aktionen (z. B. „Neu“-Button).
#[component]
pub fn Seitenkopf(titel: &'static str, untertitel: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="mb-6 flex flex-wrap items-end justify-between gap-3">
            <div>
                <h1 class="text-xl font-bold sm:text-2xl">{titel}</h1>
                <p class="text-sm text-base-content/60">{untertitel}</p>
            </div>
            <div class="flex gap-2">{children()}</div>
        </div>
    }
}

/// Modales Fenster; sichtbar, solange `offen` true ist.
#[component]
pub fn Dialog(
    offen: RwSignal<bool>,
    #[prop(into)] titel: Signal<String>,
    /// Maximale Breite des Fensters (Tailwind-Klasse)
    #[prop(default = "max-w-lg")]
    breite: &'static str,
    children: Children,
) -> impl IntoView {
    // Escape schliesst den Dialog (nur im Browser).
    #[cfg(feature = "hydrate")]
    {
        let taste = window_event_listener(leptos::ev::keydown, move |ev| {
            if ev.key() == "Escape" {
                offen.set(false);
            }
        });
        on_cleanup(move || taste.remove());
    }

    view! {
        <div class="modal modal-bottom sm:modal-middle" class:modal-open=move || offen.get() role="dialog" aria-modal="true">
            <div class=format!("modal-box max-h-[90vh] w-full {breite}")>
                <button
                    type="button"
                    class="btn btn-sm btn-circle btn-ghost absolute right-3 top-3"
                    aria-label="Schliessen"
                    on:click=move |_| offen.set(false)
                >
                    <Icon icon=icondata::LuX/>
                </button>
                <h3 class="mb-4 text-lg font-bold">{titel}</h3>
                {children()}
            </div>
            <div class="modal-backdrop bg-base-content/20" on:click=move |_| offen.set(false)></div>
        </div>
    }
}

/// Fusszeile eines Formulars im Dialog: Abbrechen + Speichern.
#[component]
pub fn DialogAktionen(offen: RwSignal<bool>, #[prop(into)] beschaeftigt: Signal<bool>) -> impl IntoView {
    view! {
        <div class="modal-action">
            <button type="button" class="btn btn-ghost" on:click=move |_| offen.set(false)>
                <Icon icon=icondata::LuX/>
                "Abbrechen"
            </button>
            <button type="submit" class="btn btn-primary" disabled=move || beschaeftigt.get()>
                <Show when=move || beschaeftigt.get() fallback=|| view! { <Icon icon=icondata::LuSave/> }>
                    <span class="loading loading-spinner loading-xs"></span>
                </Show>
                "Speichern"
            </button>
        </div>
    }
}

/// Icon-Buttons „Bearbeiten“ und „Löschen“ für eine Tabellenzeile; jeder
/// Button erscheint nur mit dem passenden Recht.
#[component]
pub fn ZeilenAktionen(
    bearbeiten: impl Fn() + Send + Sync + 'static,
    loeschen: impl Fn() + Send + Sync + 'static,
    darf_bearbeiten: impl Fn() -> bool + Copy + Send + Sync + 'static,
    darf_loeschen: impl Fn() -> bool + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let bearbeiten = std::sync::Arc::new(bearbeiten);
    let loeschen = std::sync::Arc::new(loeschen);
    view! {
        // Doppelklick auf die Buttons soll nicht zusätzlich die Zeile öffnen
        <div class="flex justify-end gap-1" on:dblclick=|ev| ev.stop_propagation()>
            <Show when=darf_bearbeiten>
                <button class="btn btn-ghost btn-sm btn-square" title="Bearbeiten" aria-label="Bearbeiten"
                    on:click={let bearbeiten = bearbeiten.clone(); move |_| bearbeiten()}>
                    <Icon icon=icondata::LuPencil/>
                </button>
            </Show>
            <Show when=darf_loeschen>
                <button class="btn btn-ghost btn-sm btn-square text-error" title="Löschen" aria-label="Löschen"
                    on:click={let loeschen = loeschen.clone(); move |_| loeschen()}>
                    <Icon icon=icondata::LuTrash2/>
                </button>
            </Show>
        </div>
    }
}

/// Leerer Zustand einer Liste mit Hinweis.
#[component]
pub fn Leer(symbol: icondata::Icon, text: &'static str) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-2 p-10 text-base-content/50">
            <Icon icon=symbol width="2.5rem" height="2.5rem"/>
            <p>{text}</p>
        </div>
    }
}

/// Rote Hinweisbox, sobald `meldung` gesetzt ist.
#[component]
pub fn Fehlermeldung(meldung: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || meldung.with(Option::is_some)>
            <div role="alert" class="alert alert-error alert-soft mb-4">
                <span>{move || meldung.get().unwrap_or_default()}</span>
                <button class="btn btn-ghost btn-xs btn-square" aria-label="Meldung schliessen"
                    on:click=move |_| meldung.set(None)>
                    <Icon icon=icondata::LuX/>
                </button>
            </div>
        </Show>
    }
}

/// Platzhalter, solange Daten geladen werden.
#[component]
pub fn Laden() -> impl IntoView {
    view! {
        <div class="flex justify-center p-8">
            <span class="loading loading-spinner loading-lg text-primary"></span>
        </div>
    }
}

/// Rahmen für einen Abschnitt mit Reitern; jeder Reiter nur mit dem Recht
/// „Sehen“ für seinen Bereich.
#[component]
pub fn Reiterseite(
    titel: &'static str,
    untertitel: &'static str,
    reiter: &'static [(Bereich, icondata::Icon)],
) -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <div class="mb-4">
            <h1 class="text-xl font-bold sm:text-2xl">{titel}</h1>
            <p class="text-sm text-base-content/60">{untertitel}</p>
        </div>
        <Transition>
            <div role="tablist" class="tabs tabs-border mb-6">
                {reiter
                    .iter()
                    .map(|&(bereich, icon)| view! {
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

/// Abschnitt ohne Reiter in der Adresse: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn ReiterStart(bereiche: &'static [Bereich]) -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <Transition>
            {move || sitzung.benutzer().map(|b| {
                let ziel = bereiche
                    .iter()
                    .find(|e| b.sieht(**e))
                    .map(|e| e.pfad())
                    .unwrap_or_else(|| b.startseite());
                view! { <Redirect path=ziel/> }
            })}
        </Transition>
    }
}

// ------------------------------------------------------------------- Helfer

/// Liefert die eigentliche Fehlermeldung ohne technisches Präfix.
pub fn meldung_aus(e: ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(m) => m,
        andere => andere.to_string(),
    }
}

/// Beobachtet das Ergebnis einer Server-Aktion: Fehler landen in `meldung`,
/// bei Erfolg wird die Meldung geleert und `bei_erfolg` aufgerufen.
pub fn ergebnis_beobachten(
    wert: impl Get<Value = Option<Result<(), ServerFnError>>> + 'static,
    meldung: RwSignal<Option<String>>,
    bei_erfolg: impl Fn() + 'static,
) {
    Effect::new(move |_| match wert.get() {
        Some(Ok(())) => {
            meldung.set(None);
            bei_erfolg();
        }
        Some(Err(e)) => meldung.set(Some(meldung_aus(e))),
        None => {}
    });
}

/// Fragt im Browser nach, bevor gelöscht wird.
pub fn bestaetigt(frage: &str) -> bool {
    window().confirm_with_message(frage).unwrap_or(false)
}
