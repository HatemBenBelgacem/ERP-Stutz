use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::{
    MitarbeiterLoeschen, MitarbeiterSpeichern, adressen_laden, geschaeftsbereiche_laden, mitarbeiter_laden,
};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich};

#[component]
pub fn MitarbeiterSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Mitarbeiter, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Mitarbeiter, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Mitarbeiter, Aktion::Loeschen);
    let speichern = ServerAction::<MitarbeiterSpeichern>::new();
    let loeschen = ServerAction::<MitarbeiterLoeschen>::new();
    let mitarbeiter = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| mitarbeiter_laden(),
    );
    // Auswahllisten für Adresse (1:1) und Geschäftsbereich (N:1)
    let adressen = Resource::new(|| (), |_| adressen_laden());
    let geschaeftsbereiche = Resource::new(|| (), |_| geschaeftsbereiche_laden());

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let vorname = RwSignal::new(String::new());
    let nachname = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let abteilung = RwSignal::new(String::new());
    let adresse_id = RwSignal::new(String::new());
    let geschaeftsbereich_id = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        vorname.set(String::new());
        nachname.set(String::new());
        email.set(String::new());
        abteilung.set(String::new());
        adresse_id.set(String::new());
        geschaeftsbereich_id.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(MitarbeiterSpeichern {
            id: bearbeite.get_untracked(),
            vorname: vorname.get_untracked(),
            nachname: nachname.get_untracked(),
            email: email.get_untracked(),
            abteilung: abteilung.get_untracked(),
            adresse_id: adresse_id.get_untracked().parse().ok(),
            geschaeftsbereich_id: geschaeftsbereich_id.get_untracked().parse().ok(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Mitarbeiter bearbeiten" } else { "Neuer Mitarbeiter" }.to_string()
    });

    view! {
        <Seitenkopf titel="Mitarbeiter" untertitel="Mitarbeitende der Stutz AG verwalten">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neuer Mitarbeiter"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        let adress_liste = adressen.await.unwrap_or_default();
                        let bereich_liste = geschaeftsbereiche.await.unwrap_or_default();
                        match mitarbeiter.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuUsers text="Noch keine Mitarbeiter erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead><tr><th>"Name"</th><th class="hidden md:table-cell">"E-Mail"</th><th class="hidden sm:table-cell">"Abteilung"</th><th class="hidden lg:table-cell">"Geschäftsbereich"</th><th class="hidden xl:table-cell">"Adresse"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|m| {
                                            let id = m.id;
                                            let name = m.name();
                                            let bereich = bereich_liste
                                                .iter()
                                                .find(|b| Some(b.id) == m.geschaeftsbereich_id)
                                                .map(|b| b.beschreibung.clone());
                                            let adresse = adress_liste
                                                .iter()
                                                .find(|a| Some(a.id) == m.adresse_id)
                                                .map(|a| a.anzeige())
                                                .unwrap_or_default();
                                            let zum_bearbeiten = m.clone();
                                            let bearbeiten = move || {
                                                let m = zum_bearbeiten.clone();
                                                bearbeite.set(Some(m.id));
                                                vorname.set(m.vorname);
                                                nachname.set(m.nachname);
                                                email.set(m.email);
                                                abteilung.set(m.abteilung);
                                                adresse_id.set(m.adresse_id.map(|a| a.to_string()).unwrap_or_default());
                                                geschaeftsbereich_id.set(m.geschaeftsbereich_id.map(|g| g.to_string()).unwrap_or_default());
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| if darf_bearbeiten.get_untracked() { bearbeiten() }}>
                                                    <td>
                                                        <div class="font-medium">{name}</div>
                                                        <div class="text-xs text-base-content/60 break-all md:hidden">{m.email.clone()}</div>
                                                    </td>
                                                    <td class="hidden text-base-content/70 md:table-cell">{m.email}</td>
                                                    <td class="hidden sm:table-cell">{m.abteilung}</td>
                                                    <td class="hidden lg:table-cell">
                                                        {bereich.map(|b| view! { <span class="badge badge-sm badge-primary badge-soft whitespace-nowrap">{b}</span> })}
                                                    </td>
                                                    <td class="hidden max-w-xs truncate text-base-content/70 xl:table-cell">{adresse}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Mitarbeiter wirklich löschen? Die Teilnahmen an Terminen werden ebenfalls entfernt.") {
                                                                    loeschen.dispatch(MitarbeiterLoeschen { id });
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
                <div class="grid gap-x-4 sm:grid-cols-2">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Vorname"</legend>
                        <input class="input w-full" required bind:value=vorname/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Nachname"</legend>
                        <input class="input w-full" required bind:value=nachname/>
                    </fieldset>
                </div>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"E-Mail"</legend>
                    <input class="input w-full" type="email" required bind:value=email/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Abteilung"</legend>
                    <input class="input w-full" bind:value=abteilung/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Geschäftsbereich"</legend>
                    <select class="select w-full"
                        on:change:target=move |ev| geschaeftsbereich_id.set(ev.target().value())>
                        <option value="" selected=move || geschaeftsbereich_id.get().is_empty()>"Kein Geschäftsbereich"</option>
                        <Suspense>
                            {move || Suspend::new(async move {
                                geschaeftsbereiche.await.unwrap_or_default().into_iter().map(|g| {
                                    let id = g.id.to_string();
                                    let wert = id.clone();
                                    view! {
                                        <option value=id selected=move || geschaeftsbereich_id.get() == wert>{g.beschreibung}</option>
                                    }
                                }).collect_view()
                            })}
                        </Suspense>
                    </select>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Adresse"</legend>
                    // 1:1 – Adressen anderer Mitarbeiter sind nicht wählbar
                    <select class="select w-full"
                        on:change:target=move |ev| adresse_id.set(ev.target().value())>
                        <option value="" selected=move || adresse_id.get().is_empty()>"Keine Adresse"</option>
                        <Suspense>
                            {move || Suspend::new(async move {
                                let vergeben: Vec<(Uuid, Uuid)> = mitarbeiter
                                    .await
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter_map(|m| m.adresse_id.map(|a| (a, m.id)))
                                    .collect();
                                adressen.await.unwrap_or_default().into_iter().map(|a| {
                                    let besitzer = vergeben.iter().find(|(adresse, _)| *adresse == a.id).map(|(_, m)| *m);
                                    let id = a.id.to_string();
                                    let wert = id.clone();
                                    view! {
                                        <option value=id
                                            disabled=move || besitzer.is_some_and(|m| bearbeite.get() != Some(m))
                                            selected=move || adresse_id.get() == wert>
                                            {a.anzeige()}
                                        </option>
                                    }
                                }).collect_view()
                            })}
                        </Suspense>
                    </select>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
