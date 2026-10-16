use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::backend::api::{KursLoeschen, KursSpeichern, kurse_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};

#[component]
pub fn KursSeite() -> impl IntoView {
    let speichern = ServerAction::<KursSpeichern>::new();
    let loeschen = ServerAction::<KursLoeschen>::new();
    let kurse = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| kurse_laden(),
    );

    // Formularzustand; `bearbeite` ist gesetzt, wenn ein bestehender Kurs geändert wird.
    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let titel = RwSignal::new(String::new());
    let beschreibung = RwSignal::new(String::new());
    let dauer = RwSignal::new(String::from("8"));
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        titel.set(String::new());
        beschreibung.set(String::new());
        dauer.set(String::from("8"));
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
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Kurs bearbeiten" } else { "Neuer Kurs" }.to_string()
    });

    view! {
        <Seitenkopf titel="Kurse" untertitel="Angebotene Weiterbildungen verwalten">
            <button class="btn btn-primary" on:click=move |_| neu()>
                <Icon icon=icondata::LuPlus/>
                "Neuer Kurs"
            </button>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match kurse.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuBookOpen text="Noch keine Kurse erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table max-sm:table-sm">
                                    <thead><tr><th>"Titel"</th><th class="hidden md:table-cell">"Beschreibung"</th><th>"Dauer"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|k| {
                                            let id = k.id;
                                            let zum_bearbeiten = k.clone();
                                            let bearbeiten = move || {
                                                let k = zum_bearbeiten.clone();
                                                bearbeite.set(Some(k.id));
                                                titel.set(k.titel);
                                                beschreibung.set(k.beschreibung);
                                                dauer.set(k.dauer_stunden.to_string());
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="cursor-pointer select-none hover:bg-base-200/50" title="Doppelklick zum Bearbeiten"
                                                    on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| bearbeiten()}>
                                                    <td>
                                                        <div class="font-medium">{k.titel}</div>
                                                        <div class="max-w-[14rem] truncate text-xs text-base-content/60 md:hidden">{k.beschreibung.clone()}</div>
                                                    </td>
                                                    <td class="hidden max-w-xs truncate text-base-content/70 md:table-cell">{k.beschreibung}</td>
                                                    <td><span class="badge badge-ghost whitespace-nowrap">{k.dauer_stunden}" h"</span></td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Kurs wirklich löschen?") {
                                                                    loeschen.dispatch(KursLoeschen { id });
                                                                }
                                                            }
                                                        />
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
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
