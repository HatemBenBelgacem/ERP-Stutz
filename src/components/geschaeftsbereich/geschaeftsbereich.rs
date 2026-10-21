use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::{GeschaeftsbereichLoeschen, GeschaeftsbereichSpeichern, geschaeftsbereiche_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich};

#[component]
pub fn GeschaeftsbereichSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Geschaeftsbereiche, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Geschaeftsbereiche, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Geschaeftsbereiche, Aktion::Loeschen);
    let speichern = ServerAction::<GeschaeftsbereichSpeichern>::new();
    let loeschen = ServerAction::<GeschaeftsbereichLoeschen>::new();
    let geschaeftsbereiche = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| geschaeftsbereiche_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let beschreibung = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        beschreibung.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(GeschaeftsbereichSpeichern {
            id: bearbeite.get_untracked(),
            beschreibung: beschreibung.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Geschäftsbereich bearbeiten" } else { "Neuer Geschäftsbereich" }.to_string()
    });

    view! {
        <Seitenkopf titel="Geschäftsbereiche" untertitel="Geschäftsbereiche, denen Kurse zugeordnet werden">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neuer Geschäftsbereich"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match geschaeftsbereiche.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuBuilding2 text="Noch keine Geschäftsbereiche erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead><tr><th>"Beschreibung"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|g| {
                                            let id = g.id;
                                            let zum_bearbeiten = g.clone();
                                            let bearbeiten = move || {
                                                let g = zum_bearbeiten.clone();
                                                bearbeite.set(Some(g.id));
                                                beschreibung.set(g.beschreibung);
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| if darf_bearbeiten.get_untracked() { bearbeiten() }}>
                                                    <td class="font-medium">{g.beschreibung}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Geschäftsbereich wirklich löschen? Er wird auch bei allen Kursen entfernt.") {
                                                                    loeschen.dispatch(GeschaeftsbereichLoeschen { id });
                                                                }
                                                            }
                                                            darf_bearbeiten=move || darf_bearbeiten.get()
                                                            darf_loeschen=move || darf_loeschen.get()
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
                    <legend class="fieldset-legend">"Beschreibung"</legend>
                    <input class="input w-full" required bind:value=beschreibung/>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
