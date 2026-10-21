use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::{AdresseLoeschen, AdresseSpeichern, adressen_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich};

#[component]
pub fn AdresseSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Adressen, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Adressen, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Adressen, Aktion::Loeschen);
    let speichern = ServerAction::<AdresseSpeichern>::new();
    let loeschen = ServerAction::<AdresseLoeschen>::new();
    let adressen = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| adressen_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let bezeichnung = RwSignal::new(String::new());
    let strasse = RwSignal::new(String::new());
    let plz = RwSignal::new(String::new());
    let ort = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        bezeichnung.set(String::new());
        strasse.set(String::new());
        plz.set(String::new());
        ort.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(AdresseSpeichern {
            id: bearbeite.get_untracked(),
            bezeichnung: bezeichnung.get_untracked(),
            strasse: strasse.get_untracked(),
            plz: plz.get_untracked(),
            ort: ort.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Adresse bearbeiten" } else { "Neue Adresse" }.to_string()
    });

    view! {
        <Seitenkopf titel="Adressen" untertitel="Orte, an denen Termine stattfinden">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neue Adresse"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match adressen.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuMapPin text="Noch keine Adressen erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead><tr><th>"Bezeichnung"</th><th class="hidden md:table-cell">"Strasse"</th><th class="hidden sm:table-cell">"PLZ / Ort"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|a| {
                                            let id = a.id;
                                            let plz_ort = format!("{} {}", a.plz, a.ort).trim().to_string();
                                            let plz_ort_kurz = plz_ort.clone();
                                            let zum_bearbeiten = a.clone();
                                            let bearbeiten = move || {
                                                let a = zum_bearbeiten.clone();
                                                bearbeite.set(Some(a.id));
                                                bezeichnung.set(a.bezeichnung);
                                                strasse.set(a.strasse);
                                                plz.set(a.plz);
                                                ort.set(a.ort);
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| if darf_bearbeiten.get_untracked() { bearbeiten() }}>
                                                    <td>
                                                        <div class="font-medium">{a.bezeichnung}</div>
                                                        // Auf kleinen Bildschirmen steht die Anschrift darunter
                                                        <div class="text-xs text-base-content/60 md:hidden">{a.strasse.clone()}</div>
                                                        <div class="text-xs text-base-content/60 sm:hidden">{plz_ort_kurz}</div>
                                                    </td>
                                                    <td class="hidden md:table-cell">{a.strasse}</td>
                                                    <td class="hidden sm:table-cell">{plz_ort}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Adresse wirklich löschen? Termine an dieser Adresse verlieren ihre Adresse.") {
                                                                    loeschen.dispatch(AdresseLoeschen { id });
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
                    <legend class="fieldset-legend">"Bezeichnung"</legend>
                    <input class="input w-full" required placeholder="z. B. Schulungsraum Werkhof" bind:value=bezeichnung/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Strasse"</legend>
                    <input class="input w-full" bind:value=strasse/>
                </fieldset>
                <div class="grid gap-x-4 sm:grid-cols-[8rem_1fr]">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"PLZ"</legend>
                        <input class="input w-full" bind:value=plz/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Ort"</legend>
                        <input class="input w-full" bind:value=ort/>
                    </fieldset>
                </div>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
