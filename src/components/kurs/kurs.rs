use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::{KursLoeschen, KursSpeichern, TerminSpeichern, geschaeftsbereiche_laden, kurse_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::components::termin::{TerminDialog, TerminMaske};
use crate::models::{Aktion, Bereich, Geschaeftsbereich};

#[component]
pub fn KursSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Kurse, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Kurse, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Kurse, Aktion::Loeschen);
    let darf_termin_anlegen = sitzung.recht(Bereich::Termine, Aktion::Anlegen);
    let speichern = ServerAction::<KursSpeichern>::new();
    let loeschen = ServerAction::<KursLoeschen>::new();
    let kurse = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| kurse_laden(),
    );
    // Geschäftsbereiche (M:N) für Anzeige und Auswahl
    let geschaeftsbereiche = Resource::new(|| (), |_| geschaeftsbereiche_laden());

    // Termin direkt zum Kurs erfassen
    let termin_maske = TerminMaske::new();
    let termin_speichern = ServerAction::<TerminSpeichern>::new();

    // Formularzustand; `bearbeite` ist gesetzt, wenn ein bestehender Kurs geändert wird.
    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let titel = RwSignal::new(String::new());
    let beschreibung = RwSignal::new(String::new());
    let dauer = RwSignal::new(String::from("8"));
    let gewaehlte_bereiche = RwSignal::new(Vec::<Uuid>::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        titel.set(String::new());
        beschreibung.set(String::new());
        dauer.set(String::from("8"));
        gewaehlte_bereiche.set(Vec::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(dauer_stunden) = dauer.get_untracked().trim().parse::<i32>() else {
            formular_meldung.set(Some("Die Dauer muss eine ganze Zahl sein.".into()));
            return;
        };
        speichern.dispatch(KursSpeichern {
            id: bearbeite.get_untracked(),
            titel: titel.get_untracked(),
            beschreibung: beschreibung.get_untracked(),
            dauer_stunden,
            geschaeftsbereich_ids: gewaehlte_bereiche.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Kurs bearbeiten" } else { "Neuer Kurs" }.to_string()
    });

    view! {
        <Seitenkopf titel="Kurse" untertitel="Angebotene Kurse verwalten">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neuer Kurs"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        let bereiche = geschaeftsbereiche.await.unwrap_or_default();
                        match kurse.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuBookOpen text="Noch keine Kurse erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead><tr><th>"Titel"</th><th class="hidden md:table-cell">"Beschreibung"</th><th class="hidden lg:table-cell">"Geschäftsbereiche"</th><th>"Dauer"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|k| {
                                            let id = k.id;
                                            let zum_bearbeiten = k.clone();
                                            let bereich_namen: Vec<String> = bereiche
                                                .iter()
                                                .filter(|b| k.geschaeftsbereich_ids.contains(&b.id))
                                                .map(|b| b.beschreibung.clone())
                                                .collect();
                                            let bearbeiten = move || {
                                                let k = zum_bearbeiten.clone();
                                                bearbeite.set(Some(k.id));
                                                titel.set(k.titel);
                                                beschreibung.set(k.beschreibung);
                                                dauer.set(k.dauer_stunden.to_string());
                                                gewaehlte_bereiche.set(k.geschaeftsbereich_ids);
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| if darf_bearbeiten.get_untracked() { bearbeiten() }}>
                                                    <td>
                                                        <div class="font-medium">{k.titel}</div>
                                                        <div class="max-w-[14rem] truncate text-xs text-base-content/60 md:hidden">{k.beschreibung.clone()}</div>
                                                    </td>
                                                    <td class="hidden max-w-xs truncate text-base-content/70 md:table-cell">{k.beschreibung}</td>
                                                    <td class="hidden lg:table-cell">
                                                        <div class="flex flex-wrap gap-1">
                                                            {bereich_namen.into_iter().map(|n| view! {
                                                                <span class="badge badge-sm badge-primary badge-soft whitespace-nowrap">{n}</span>
                                                            }).collect_view()}
                                                        </div>
                                                    </td>
                                                    <td><span class="badge badge-ghost whitespace-nowrap">{k.dauer_stunden}" h"</span></td>
                                                    <td>
                                                        <div class="flex justify-end gap-1">
                                                        <Show when=move || darf_termin_anlegen.get()>
                                                            <button class="btn btn-ghost btn-sm btn-square"
                                                                title="Termin zu diesem Kurs erstellen"
                                                                aria-label="Termin zu diesem Kurs erstellen"
                                                                on:dblclick=|ev| ev.stop_propagation()
                                                                on:click=move |_| termin_maske.neu(Some(id))>
                                                                <Icon icon=icondata::LuCalendarPlus/>
                                                            </button>
                                                        </Show>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Kurs wirklich löschen?") {
                                                                    loeschen.dispatch(KursLoeschen { id });
                                                                }
                                                            }
                                                            darf_bearbeiten=move || darf_bearbeiten.get()
                                                            darf_loeschen=move || darf_loeschen.get()
                                                        />
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_any(),
                            Err(e) => view! { <div class="alert alert-error m-4">{meldung_aus(e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </div>

        <Dialog offen=dialog_offen titel=dialog_titel>
            <form on:submit=absenden>
                <Fehlermeldung meldung=formular_meldung/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Titel"</legend>
                    <input class="input w-full" required bind:value=titel/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Beschreibung"</legend>
                    <textarea class="textarea w-full" rows="3" bind:value=beschreibung></textarea>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Dauer (Stunden)"</legend>
                    <input class="input w-full" type="number" min="0" bind:value=dauer/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Geschäftsbereiche"</legend>
                    <Suspense fallback=Laden>
                        {move || Suspend::new(async move {
                            let liste = geschaeftsbereiche.await.unwrap_or_default();
                            view! { <BereichAuswahl liste gewaehlt=gewaehlte_bereiche/> }
                        })}
                    </Suspense>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>

        <TerminDialog maske=termin_maske speichern=termin_speichern/>
    }
}

/// Mehrfachauswahl der Geschäftsbereiche eines Kurses.
#[component]
fn BereichAuswahl(liste: Vec<Geschaeftsbereich>, gewaehlt: RwSignal<Vec<Uuid>>) -> impl IntoView {
    if liste.is_empty() {
        return view! { <p class="p-2 text-base-content/60">"Noch keine Geschäftsbereiche erfasst."</p> }.into_any();
    }
    view! {
        <div class="max-h-48 overflow-y-auto rounded-box border border-base-300 p-1">
            {liste.into_iter().map(|b| {
                let id = b.id;
                view! {
                    <label class="label w-full cursor-pointer rounded-field px-2 py-1.5 hover:bg-base-200">
                        <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                            prop:checked=move || gewaehlt.with(|v| v.contains(&id))
                            on:change:target=move |ev| {
                                let an = ev.target().checked();
                                gewaehlt.update(|v| {
                                    v.retain(|x| *x != id);
                                    if an {
                                        v.push(id);
                                    }
                                });
                            }/>
                        <span class="text-base-content">{b.beschreibung}</span>
                    </label>
                }
            }).collect_view()}
        </div>
    }
    .into_any()
}
