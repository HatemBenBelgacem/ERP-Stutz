use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::rollen::{RolleLoeschen, RolleSpeichern, rollen_laden};
use crate::components::benutzer::RollenBadge;
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen, bestaetigt,
    ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich, Rolle, alle_rechte, recht};

#[component]
pub fn RollenSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Rollen, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Rollen, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Rollen, Aktion::Loeschen);

    let speichern = ServerAction::<RolleSpeichern>::new();
    let loeschen = ServerAction::<RolleLoeschen>::new();
    let rollen = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| rollen_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let ist_admin = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let beschreibung = RwSignal::new(String::new());
    let rechte = RwSignal::new(Vec::<String>::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        ist_admin.set(false);
        name.set(String::new());
        beschreibung.set(String::new());
        rechte.set(Vec::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    let rolle_bearbeiten = move |r: Rolle| {
        bearbeite.set(Some(r.id));
        ist_admin.set(r.ist_admin);
        name.set(r.name);
        beschreibung.set(r.beschreibung);
        rechte.set(r.rechte);
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(RolleSpeichern {
            id: bearbeite.get_untracked(),
            name: name.get_untracked(),
            beschreibung: beschreibung.get_untracked(),
            rechte: rechte.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Rolle bearbeiten" } else { "Neue Rolle" }.to_string()
    });

    view! {
        <Seitenkopf titel="Rollen" untertitel="Welche Bereiche und Funktionen eine Rolle nutzen darf">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neue Rolle"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match rollen.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuShieldCheck text="Noch keine Rollen."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table max-sm:table-sm">
                                    <thead>
                                        <tr>
                                            <th>"Rolle"</th>
                                            <th class="hidden xl:table-cell">"Beschreibung"</th>
                                            <th class="hidden sm:table-cell">"Rechte"</th>
                                            <th>"Benutzer"</th>
                                            <th></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {liste.into_iter().map(|r| {
                                            let id = r.id;
                                            let fuer_doppelklick = r.clone();
                                            let fuer_button = r.clone();
                                            let rechte_text = if r.ist_admin {
                                                "Alle".to_string()
                                            } else {
                                                format!("{} von {}", r.rechte.len(), alle_rechte().len())
                                            };
                                            let admin = r.ist_admin;
                                            view! {
                                                <tr class="select-none hover:bg-base-200/50"
                                                    class:cursor-pointer=move || darf_bearbeiten.get()
                                                    title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                    on:dblclick=move |_| if darf_bearbeiten.get_untracked() {
                                                        rolle_bearbeiten(fuer_doppelklick.clone())
                                                    }>
                                                    <td>
                                                        <RollenBadge ist_admin=r.ist_admin name=Some(r.name.clone())/>
                                                        <div class="mt-1 max-w-[14rem] truncate text-xs text-base-content/60 xl:hidden">{r.beschreibung.clone()}</div>
                                                    </td>
                                                    <td class="hidden max-w-xs truncate text-base-content/70 xl:table-cell">{r.beschreibung}</td>
                                                    <td class="hidden whitespace-nowrap sm:table-cell">{rechte_text}</td>
                                                    <td>{r.anzahl_benutzer}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=move || rolle_bearbeiten(fuer_button.clone())
                                                            loeschen=move || {
                                                                if bestaetigt("Rolle wirklich löschen?") {
                                                                    loeschen.dispatch(RolleLoeschen { id });
                                                                }
                                                            }
                                                            darf_bearbeiten=move || darf_bearbeiten.get()
                                                            // Die Admin-Rolle lässt sich nicht löschen
                                                            darf_loeschen=move || darf_loeschen.get() && !admin
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

        <Dialog offen=dialog_offen titel=dialog_titel breite="max-w-2xl">
            <form on:submit=absenden>
                <Fehlermeldung meldung=formular_meldung/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Name"</legend>
                    <input class="input w-full" required disabled=move || ist_admin.get() bind:value=name/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Beschreibung"</legend>
                    <input class="input w-full" bind:value=beschreibung/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Rechte"</legend>
                    <Show
                        when=move || !ist_admin.get()
                        fallback=|| view! {
                            <p class="text-sm text-base-content/60">"Die Administrator-Rolle hat immer alle Rechte."</p>
                        }
                    >
                        // Typ hier abschneiden, sonst wird der Dialog-Typ für den Compiler zu tief
                        {view! { <RechteMatrix rechte/> }.into_any()}
                    </Show>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}

/// Tabelle Bereich × Funktion mit je einem Häkchen.
#[component]
fn RechteMatrix(rechte: RwSignal<Vec<String>>) -> impl IntoView {
    // Anlegen/Bearbeiten/Löschen setzen „Sehen“ voraus; ohne „Sehen“ fällt der Rest weg.
    let umschalten = move |bereich: Bereich, aktion: Aktion, an: bool| {
        rechte.update(|liste| {
            let schluessel = recht(bereich, aktion);
            let sehen = recht(bereich, Aktion::Sehen);
            if an {
                if !liste.contains(&schluessel) {
                    liste.push(schluessel);
                }
                if !liste.contains(&sehen) {
                    liste.push(sehen);
                }
            } else if aktion == Aktion::Sehen {
                let praefix = format!("{}.", bereich.schluessel());
                liste.retain(|r| !r.starts_with(&praefix));
            } else {
                liste.retain(|r| *r != schluessel);
            }
        });
    };

    view! {
        <div class="overflow-x-auto rounded-box border border-base-300">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Bereich"</th>
                        {Aktion::ALLE.into_iter().map(|a| view! { <th class="px-1 text-center">{a.titel()}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {Bereich::ALLE.into_iter().map(|bereich| view! {
                        <tr>
                            <td>
                                <div class="whitespace-nowrap">{bereich.titel()}</div>
                                {bereich.gruppe().map(|gruppe| view! {
                                    <div class="text-xs text-base-content/50">{gruppe}</div>
                                })}
                            </td>
                            {Aktion::ALLE.into_iter().map(|aktion| {
                                if bereich.aktionen().contains(&aktion) {
                                    let schluessel = recht(bereich, aktion);
                                    view! {
                                        <td class="px-1 text-center">
                                            <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                                                aria-label=format!("{} {}", bereich.titel(), aktion.titel())
                                                prop:checked=move || rechte.with(|r| r.contains(&schluessel))
                                                on:change:target=move |ev| umschalten(bereich, aktion, ev.target().checked())/>
                                        </td>
                                    }.into_any()
                                } else {
                                    view! { <td class="text-center text-base-content/30">"–"</td> }.into_any()
                                }
                            }).collect_view()}
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
        <div class="mt-1 flex gap-2">
            <button type="button" class="btn btn-ghost btn-xs" on:click=move |_| rechte.set(alle_rechte())>
                <Icon icon=icondata::LuCheck/>
                "Alle"
            </button>
            <button type="button" class="btn btn-ghost btn-xs" on:click=move |_| rechte.set(Vec::new())>
                <Icon icon=icondata::LuX/>
                "Keine"
            </button>
        </div>
    }
}
