//! Projekte und Bauobjekte (Bauobjekt 1:N Projekt) als Reiter unter „Projekte“.

use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::adressen_laden;
use crate::backend::projekte::{
    BauobjektLoeschen, BauobjektSpeichern, ProjektLoeschen, ProjektSpeichern, bauobjekte_laden, projekte_laden,
};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, ReiterStart, Reiterseite, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::components::termin::{AdresseAuswahl, AdresseZeilen};
use crate::models::{Aktion, Bauobjekt, BauobjektEingabe, Bereich, Projekt, ProjektEingabe};

const REITER: [(Bereich, icondata::Icon); 2] =
    [(Bereich::Projekte, icondata::LuFolderKanban), (Bereich::Bauobjekte, icondata::LuBuilding2)];

/// Rahmen für Projekte mit Reitern für Projekte und Bauobjekte.
#[component]
pub fn ProjekteSeite() -> impl IntoView {
    view! { <Reiterseite titel="Projekte" untertitel="Bauobjekte und ihre Projekte" reiter=&REITER/> }
}

/// `/projekte` ohne Reiter: zum ersten erlaubten Reiter weiterleiten.
#[component]
pub fn ProjekteStart() -> impl IntoView {
    view! { <ReiterStart bereiche=&Bereich::PROJEKTE/> }
}

/// Adresse in einer Tabellenzelle; ohne Adresse ein Strich.
#[component]
fn AdressZelle(adresse: Option<crate::models::Adresse>) -> impl IntoView {
    if adresse.is_some() {
        view! { <div class="text-sm"><AdresseZeilen adresse/></div> }.into_any()
    } else {
        view! { <span class="text-base-content/40">"–"</span> }.into_any()
    }
}

// ---------------------------------------------------------------- Bauobjekte

#[component]
pub fn BauobjektSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Bauobjekte, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Bauobjekte, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Bauobjekte, Aktion::Loeschen);
    let darf_projekt_anlegen = sitzung.recht(Bereich::Projekte, Aktion::Anlegen);

    let speichern = ServerAction::<BauobjektSpeichern>::new();
    let loeschen = ServerAction::<BauobjektLoeschen>::new();
    // Neue Projekte ändern die Anzahl in der Liste
    let projekt_speichern = ServerAction::<ProjektSpeichern>::new();
    let bauobjekte = Resource::new(
        move || (speichern.version().get(), loeschen.version().get(), projekt_speichern.version().get()),
        |_| bauobjekte_laden(),
    );
    let adressen = Resource::new(move || speichern.version().get(), |_| adressen_laden());

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let bezeichnung = RwSignal::new(String::new());
    let bauherr_id = RwSignal::new(String::new());
    let architekt_id = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);
    let projekt_maske = ProjektMaske::new();

    let oeffnen = move |o: Option<Bauobjekt>| {
        bearbeite.set(o.as_ref().map(|o| o.id));
        bezeichnung.set(o.as_ref().map(|o| o.bezeichnung.clone()).unwrap_or_default());
        bauherr_id.set(adress_id(o.as_ref().and_then(|o| o.bauherr.as_ref())));
        architekt_id.set(adress_id(o.as_ref().and_then(|o| o.architekt.as_ref())));
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(BauobjektSpeichern {
            id: bearbeite.get_untracked(),
            eingabe: BauobjektEingabe {
                bezeichnung: bezeichnung.get_untracked(),
                bauherr_id: bauherr_id.get_untracked().parse().ok(),
                architekt_id: architekt_id.get_untracked().parse().ok(),
            },
        });
    };
    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Bauobjekt bearbeiten" } else { "Neues Bauobjekt" }.to_string()
    });

    view! {
        <Seitenkopf titel="Bauobjekte" untertitel="Objekte mit Bauherr und Architekt">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| oeffnen(None)>
                    <Icon icon=icondata::LuPlus/>
                    "Neues Bauobjekt"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match bauobjekte.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuBuilding2 text="Noch keine Bauobjekte erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead>
                                        <tr>
                                            <th>"Bezeichnung"</th>
                                            <th class="hidden md:table-cell">"Bauherr"</th>
                                            <th class="hidden lg:table-cell">"Architekt"</th>
                                            <th>"Projekte"</th>
                                            <th></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {liste.into_iter().map(|o| {
                                            let id = o.id;
                                            let fuer_doppelklick = o.clone();
                                            let fuer_knopf = o.clone();
                                            let fuer_projekt = o.clone();
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                    class:cursor-pointer=move || darf_bearbeiten.get()
                                                    title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                    on:dblclick=move |_| if darf_bearbeiten.get_untracked() { oeffnen(Some(fuer_doppelklick.clone())) }>
                                                    <td class="font-medium">{o.bezeichnung}</td>
                                                    <td class="hidden md:table-cell"><AdressZelle adresse=o.bauherr/></td>
                                                    <td class="hidden lg:table-cell"><AdressZelle adresse=o.architekt/></td>
                                                    <td><span class="badge badge-ghost">{o.anzahl_projekte}</span></td>
                                                    <td>
                                                        <div class="flex justify-end gap-1">
                                                            <Show when=move || darf_projekt_anlegen.get()>
                                                                <button class="btn btn-ghost btn-sm btn-square"
                                                                    title="Neues Projekt zu diesem Bauobjekt"
                                                                    aria-label="Neues Projekt zu diesem Bauobjekt"
                                                                    on:dblclick=|ev| ev.stop_propagation()
                                                                    on:click={
                                                                        let o = fuer_projekt.clone();
                                                                        move |_| projekt_maske.neu(Some(&o))
                                                                    }>
                                                                    <Icon icon=icondata::LuFolderPlus/>
                                                                </button>
                                                            </Show>
                                                            <ZeilenAktionen
                                                                bearbeiten=move || oeffnen(Some(fuer_knopf.clone()))
                                                                loeschen=move || {
                                                                    if bestaetigt("Bauobjekt wirklich löschen?") {
                                                                        loeschen.dispatch(BauobjektLoeschen { id });
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
                    <legend class="fieldset-legend">"Bezeichnung"</legend>
                    <input class="input w-full" required placeholder="z. B. Überbauung Seepark" bind:value=bezeichnung/>
                </fieldset>
                <AdressFelder adressen bauherr_id architekt_id/>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>

        <ProjektDialog maske=projekt_maske speichern=projekt_speichern/>
    }
}

/// Suchbare Auswahl für Bauherr und Architekt.
#[component]
fn AdressFelder(
    adressen: Resource<Result<Vec<crate::models::Adresse>, ServerFnError>>,
    bauherr_id: RwSignal<String>,
    architekt_id: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="grid gap-x-4 sm:grid-cols-2">
            <fieldset class="fieldset min-w-0">
                <legend class="fieldset-legend">"Bauherr"</legend>
                <Suspense fallback=|| view! { <div class="skeleton h-10 w-full"></div> }>
                    {move || Suspend::new(async move {
                        let liste = adressen.await.unwrap_or_default();
                        view! { <AdresseAuswahl liste adresse_id=bauherr_id/> }
                    })}
                </Suspense>
            </fieldset>
            <fieldset class="fieldset min-w-0">
                <legend class="fieldset-legend">"Architekt"</legend>
                <Suspense fallback=|| view! { <div class="skeleton h-10 w-full"></div> }>
                    {move || Suspend::new(async move {
                        let liste = adressen.await.unwrap_or_default();
                        view! { <AdresseAuswahl liste adresse_id=architekt_id/> }
                    })}
                </Suspense>
            </fieldset>
        </div>
    }
}

fn adress_id(adresse: Option<&crate::models::Adresse>) -> String {
    adresse.map(|a| a.id.to_string()).unwrap_or_default()
}

// ------------------------------------------------------------------ Projekte

#[component]
pub fn ProjektSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Projekte, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Projekte, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Projekte, Aktion::Loeschen);

    let speichern = ServerAction::<ProjektSpeichern>::new();
    let loeschen = ServerAction::<ProjektLoeschen>::new();
    let projekte = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| projekte_laden(),
    );
    let maske = ProjektMaske::new();
    let listen_meldung = RwSignal::new(None::<String>);
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    view! {
        <Seitenkopf titel="Projekte" untertitel="Projekte der Bauobjekte">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| maske.neu(None)>
                    <Icon icon=icondata::LuPlus/>
                    "Neues Projekt"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match projekte.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuFolderKanban text="Noch keine Projekte erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead>
                                        <tr>
                                            <th>"Projekt"</th>
                                            <th class="hidden sm:table-cell">"Bauobjekt"</th>
                                            <th class="hidden md:table-cell">"Bauherr"</th>
                                            <th class="hidden lg:table-cell">"Architekt"</th>
                                            <th></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {liste.into_iter().map(|p| {
                                            let id = p.id;
                                            let fuer_doppelklick = p.clone();
                                            let fuer_knopf = p.clone();
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                    class:cursor-pointer=move || darf_bearbeiten.get()
                                                    title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                    on:dblclick=move |_| if darf_bearbeiten.get_untracked() { maske.bearbeiten(fuer_doppelklick.clone()) }>
                                                    <td>
                                                        <div class="font-mono text-xs text-base-content/60">{p.projektnummer}</div>
                                                        <div class="font-medium">{p.bezeichnung}</div>
                                                        // Auf dem Smartphone steht das Bauobjekt darunter
                                                        <div class="text-xs text-base-content/60 sm:hidden">{p.bauobjekt_bezeichnung.clone()}</div>
                                                    </td>
                                                    <td class="hidden sm:table-cell">{p.bauobjekt_bezeichnung}</td>
                                                    <td class="hidden md:table-cell"><AdressZelle adresse=p.bauherr/></td>
                                                    <td class="hidden lg:table-cell"><AdressZelle adresse=p.architekt/></td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=move || maske.bearbeiten(fuer_knopf.clone())
                                                            loeschen=move || {
                                                                if bestaetigt("Projekt wirklich löschen?") {
                                                                    loeschen.dispatch(ProjektLoeschen { id });
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

        <ProjektDialog maske speichern/>
    }
}

/// Zustand der Projekt-Eingabemaske; die Projekt- und die Bauobjektseite
/// nutzen dieselbe Maske.
#[derive(Clone, Copy)]
pub struct ProjektMaske {
    offen: RwSignal<bool>,
    bearbeite: RwSignal<Option<Uuid>>,
    projektnummer: RwSignal<String>,
    bezeichnung: RwSignal<String>,
    bauobjekt_id: RwSignal<String>,
    bauherr_id: RwSignal<String>,
    architekt_id: RwSignal<String>,
    meldung: RwSignal<Option<String>>,
}

impl ProjektMaske {
    pub fn new() -> Self {
        Self {
            offen: RwSignal::new(false),
            bearbeite: RwSignal::new(None),
            projektnummer: RwSignal::new(String::new()),
            bezeichnung: RwSignal::new(String::new()),
            bauobjekt_id: RwSignal::new(String::new()),
            bauherr_id: RwSignal::new(String::new()),
            architekt_id: RwSignal::new(String::new()),
            meldung: RwSignal::new(None),
        }
    }

    /// Neues Projekt, optional zu einem Bauobjekt; dessen Adressen werden übernommen.
    pub fn neu(&self, objekt: Option<&Bauobjekt>) {
        self.bearbeite.set(None);
        self.projektnummer.set(String::new());
        self.bezeichnung.set(String::new());
        self.bauobjekt_id.set(objekt.map(|o| o.id.to_string()).unwrap_or_default());
        self.adressen_von(objekt);
        self.meldung.set(None);
        self.offen.set(true);
    }

    pub fn bearbeiten(&self, p: Projekt) {
        self.bearbeite.set(Some(p.id));
        self.projektnummer.set(p.projektnummer);
        self.bezeichnung.set(p.bezeichnung);
        self.bauobjekt_id.set(p.bauobjekt_id.to_string());
        self.bauherr_id.set(adress_id(p.bauherr.as_ref()));
        self.architekt_id.set(adress_id(p.architekt.as_ref()));
        self.meldung.set(None);
        self.offen.set(true);
    }

    /// Bauherr und Architekt vom Bauobjekt übernehmen.
    fn adressen_von(&self, objekt: Option<&Bauobjekt>) {
        self.bauherr_id.set(adress_id(objekt.and_then(|o| o.bauherr.as_ref())));
        self.architekt_id.set(adress_id(objekt.and_then(|o| o.architekt.as_ref())));
    }
}

impl Default for ProjektMaske {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
fn ProjektDialog(maske: ProjektMaske, speichern: ServerAction<ProjektSpeichern>) -> impl IntoView {
    let ProjektMaske { offen, bearbeite, projektnummer, bezeichnung, bauobjekt_id, bauherr_id, architekt_id, meldung } =
        maske;
    // Bei jedem Öffnen neu laden, damit eben angelegte Bauobjekte und Adressen dabei sind
    let bauobjekte = Resource::new(move || offen.get(), |_| bauobjekte_laden());
    let adressen = Resource::new(move || offen.get(), |_| adressen_laden());
    ergebnis_beobachten(speichern.value(), meldung, move || offen.set(false));

    // Adressen des gewählten Bauobjekts übernehmen
    let vom_objekt = move || {
        let id = bauobjekt_id.get_untracked();
        let liste = bauobjekte.get_untracked().and_then(Result::ok).unwrap_or_default();
        maske.adressen_von(liste.iter().find(|o| o.id.to_string() == id));
    };

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(objekt) = bauobjekt_id.get_untracked().parse::<Uuid>() else {
            meldung.set(Some("Bitte ein Bauobjekt auswählen.".into()));
            return;
        };
        speichern.dispatch(ProjektSpeichern {
            id: bearbeite.get_untracked(),
            eingabe: ProjektEingabe {
                projektnummer: projektnummer.get_untracked(),
                bezeichnung: bezeichnung.get_untracked(),
                bauobjekt_id: objekt,
                bauherr_id: bauherr_id.get_untracked().parse().ok(),
                architekt_id: architekt_id.get_untracked().parse().ok(),
            },
        });
    };
    let titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Projekt bearbeiten" } else { "Neues Projekt" }.to_string()
    });

    view! {
        <Dialog offen titel breite="max-w-2xl">
            <form on:submit=absenden>
                <Fehlermeldung meldung/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Bauobjekt"</legend>
                    <select class="select w-full" required
                        on:change:target=move |ev| {
                            bauobjekt_id.set(ev.target().value());
                            vom_objekt();
                        }>
                        <option value="" disabled selected=move || bauobjekt_id.get().is_empty()>"Bauobjekt wählen …"</option>
                        <Suspense>
                            {move || Suspend::new(async move {
                                bauobjekte.await.unwrap_or_default().into_iter().map(|o| {
                                    let id = o.id.to_string();
                                    let wert = id.clone();
                                    view! { <option value=id selected=move || bauobjekt_id.get() == wert>{o.bezeichnung}</option> }
                                }).collect_view()
                            })}
                        </Suspense>
                    </select>
                </fieldset>
                <div class="grid gap-x-4 sm:grid-cols-[10rem_1fr]">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Projektnummer"</legend>
                        <input class="input w-full" required placeholder="z. B. P-2026-001" bind:value=projektnummer/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Bezeichnung"</legend>
                        <input class="input w-full" required bind:value=bezeichnung/>
                    </fieldset>
                </div>
                <AdressFelder adressen bauherr_id architekt_id/>
                <p class="label flex-wrap">
                    "Bauherr und Architekt kommen vom Bauobjekt und lassen sich pro Projekt anpassen."
                    <button type="button" class="link link-primary" on:click=move |_| vom_objekt()>
                        "Vom Bauobjekt übernehmen"
                    </button>
                </p>
                <DialogAktionen offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
