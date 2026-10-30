use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

use crate::app::use_sitzung;
#[cfg(feature = "hydrate")]
use crate::backend::artikel::BildUpload;
use crate::backend::artikel::{
    ArtikelAnlegen, ArtikelBearbeiten, ArtikelEingabe, ArtikelLoeschen, BildAenderung,
    artikel_laden,
};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen, bestaetigt,
    ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich};

#[component]
pub fn ArtikelSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Lager, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Lager, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Lager, Aktion::Loeschen);
    let anlegen = ServerAction::<ArtikelAnlegen>::new();
    let bearbeiten = ServerAction::<ArtikelBearbeiten>::new();
    let loeschen = ServerAction::<ArtikelLoeschen>::new();
    let artikel = Resource::new(
        move || {
            (
                anlegen.version().get(),
                bearbeiten.version().get(),
                loeschen.version().get(),
            )
        },
        |_| artikel_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let bezeichnung = RwSignal::new(String::new());
    // Grössen als Textfelder. Jedes Feld hat eine Kennung, damit sich genau dieses entfernen lässt.
    let naechste_zeile = RwSignal::new(0_u32);
    let groessen = RwSignal::new(Vec::<(u32, RwSignal<String>)>::new());
    let neue_zeile = move |text: String| {
        let kennung = naechste_zeile.get_untracked();
        naechste_zeile.set(kennung + 1);
        (kennung, RwSignal::new(text))
    };
    let bild = RwSignal::new(BildAenderung::Behalten);
    // Dateiname des gespeicherten Bildes; der Zeitstempel verhindert veraltete Vorschauen im Cache
    let bild_name = RwSignal::new(None::<String>);
    let bild_stand = RwSignal::new(0_i64);
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        bezeichnung.set(String::new());
        groessen.set(vec![neue_zeile(String::new())]);
        bild.set(BildAenderung::Behalten);
        bild_name.set(None);
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    Effect::new(move |_| {
        if let Some(ergebnis) = anlegen.value().get() {
            match ergebnis {
                Ok(_) => {
                    formular_meldung.set(None);
                    dialog_offen.set(false);
                }
                Err(e) => formular_meldung.set(Some(meldung_aus(e))),
            }
        }
    });
    ergebnis_beobachten(bearbeiten.value(), formular_meldung, move || {
        dialog_offen.set(false)
    });
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let eingabe = ArtikelEingabe {
            bezeichnung: bezeichnung.get_untracked(),
            groessen: groessen
                .get_untracked()
                .into_iter()
                .map(|(_, feld)| feld.get_untracked())
                .collect(),
            bild: bild.get_untracked(),
        };
        match bearbeite.get_untracked() {
            Some(id) => {
                bearbeiten.dispatch(ArtikelBearbeiten { id, eingabe });
            }
            None => {
                anlegen.dispatch(ArtikelAnlegen { eingabe });
            }
        }
    };

    let speichern_laeuft =
        Signal::derive(move || anlegen.pending().get() || bearbeiten.pending().get());
    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() {
            "Artikel bearbeiten"
        } else {
            "Neuer Artikel"
        }
        .to_string()
    });
    let hat_bild = move || match bild.get() {
        BildAenderung::Neu(_) => true,
        BildAenderung::Behalten => bild_name.get().is_some(),
        BildAenderung::Entfernen => false,
    };

    view! {
        <Seitenkopf titel="Artikel" untertitel="Bezeichnung, Grössen und Bild der Artikel">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neuer Artikel"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match artikel.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuPackage text="Noch keine Artikel erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead><tr><th>"Bezeichnung"</th><th>"Grössen"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|a| {
                                            let id = a.id;
                                            let name = a.bezeichnung.clone();
                                            let zum_bearbeiten = a.clone();
                                            let oeffnen = move || {
                                                let a = zum_bearbeiten.clone();
                                                let mut felder: Vec<(u32, RwSignal<String>)> =
                                                    a.groessen.iter().map(|g| neue_zeile(g.clone())).collect();
                                                if felder.is_empty() {
                                                    felder.push(neue_zeile(String::new()));
                                                }
                                                bearbeite.set(Some(a.id));
                                                bezeichnung.set(a.bezeichnung.clone());
                                                groessen.set(felder);
                                                bild.set(BildAenderung::Behalten);
                                                bild_name.set(a.bild.clone());
                                                bild_stand.set(a.geaendert_am.and_utc().timestamp());
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick={let oeffnen = oeffnen.clone(); move |_| if darf_bearbeiten.get_untracked() { oeffnen() }}>
                                                    <td class="font-medium">{a.bezeichnung}</td>
                                                    <td>{a.groessen.join(", ")}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=oeffnen.clone()
                                                            loeschen=move || {
                                                                if bestaetigt(&format!("Artikel „{name}“ wirklich löschen?")) {
                                                                    loeschen.dispatch(ArtikelLoeschen { id });
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
                    <input class="input w-full" required placeholder="z. B. Schraube M6" bind:value=bezeichnung/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Grössen"</legend>
                    {move || groessen.get().into_iter().map(|(kennung, feld)| view! {
                        <div class="mb-2 flex items-center gap-2">
                            <input class="input w-full" placeholder="z. B. M oder 42" bind:value=feld/>
                            <button type="button" class="btn btn-ghost btn-square" aria-label="Grösse entfernen"
                                    on:click=move |_| groessen.update(|liste| liste.retain(|(k, _)| *k != kennung))>
                                <Icon icon=icondata::LuX/>
                            </button>
                        </div>
                    }).collect_view()}
                    <button type="button" class="btn btn-ghost btn-sm justify-start"
                            on:click=move |_| groessen.update(|liste| liste.push(neue_zeile(String::new())))>
                        <Icon icon=icondata::LuPlus/>
                        "Grösse hinzufügen"
                    </button>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Bild"</legend>
                    <Show when=move || matches!(bild.get(), BildAenderung::Behalten) && bild_name.get().is_some()>
                        <img class="mb-2 max-h-40 rounded-box border border-base-300" alt="Bild des Artikels"
                             src=move || bearbeite.get()
                                 .map(|id| format!("/artikel/{id}/bild?v={}", bild_stand.get()))
                                 .unwrap_or_default()/>
                    </Show>
                    <input type="file" class="file-input w-full" accept=".jpg,.jpeg,.png,image/jpeg,image/png"
                           on:change=move |ev| bild_datei_gewaehlt(ev, bild, formular_meldung)/>
                    <p class="label">"JPG oder PNG, höchstens 2 MB"</p>
                    <Show when=hat_bild>
                        <div class="mt-2 flex items-center justify-between gap-2 text-sm">
                            <span>{move || match bild.get() {
                                BildAenderung::Neu(b) => format!("Neues Bild: {}", b.name),
                                _ => "Bild vorhanden".to_string(),
                            }}</span>
                            <button type="button" class="btn btn-ghost btn-sm text-error"
                                    on:click=move |_| bild.set(BildAenderung::Entfernen)>
                                <Icon icon=icondata::LuTrash2/>
                                "Bild entfernen"
                            </button>
                        </div>
                    </Show>
                    <Show when=move || matches!(bild.get(), BildAenderung::Entfernen) && bild_name.get().is_some()>
                        <div class="mt-2 flex items-center justify-between gap-2 text-sm">
                            <span>"Das Bild wird beim Speichern entfernt."</span>
                            <button type="button" class="btn btn-ghost btn-sm"
                                    on:click=move |_| bild.set(BildAenderung::Behalten)>
                                "Behalten"
                            </button>
                        </div>
                    </Show>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern_laeuft/>
            </form>
        </Dialog>
    }
}

/// Liest die im Dateidialog gewählte Datei im Browser. Eine zu grosse oder falsche
/// Datei meldet das Formular sofort; der Server prüft sie beim Speichern noch einmal.
#[cfg(feature = "hydrate")]
fn bild_datei_gewaehlt(
    ev: web_sys::Event,
    bild: RwSignal<BildAenderung>,
    meldung: RwSignal<Option<String>>,
) {
    let Some(eingabe) = ev
        .target()
        .and_then(|ziel| ziel.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(datei) = eingabe.files().and_then(|liste| liste.get(0)) else {
        return;
    };
    let Ok(leser) = web_sys::FileReader::new() else {
        return;
    };
    let name = datei.name();
    let beim_laden = {
        let leser = leser.clone();
        Closure::once(move || {
            let Ok(puffer) = leser.result() else {
                return;
            };
            let upload = BildUpload {
                name,
                daten: js_sys::Uint8Array::new(&puffer).to_vec(),
            };
            match upload.pruefen() {
                Ok(()) => {
                    meldung.set(None);
                    bild.set(BildAenderung::Neu(upload));
                }
                Err(text) => {
                    meldung.set(Some(text));
                    bild.set(BildAenderung::Behalten);
                }
            }
        })
    };
    leser.set_onload(Some(beim_laden.as_ref().unchecked_ref()));
    beim_laden.forget();
    let _ = leser.read_as_array_buffer(&datei);
}

#[cfg(not(feature = "hydrate"))]
fn bild_datei_gewaehlt<E>(
    _ev: E,
    _bild: RwSignal<BildAenderung>,
    _meldung: RwSignal<Option<String>>,
) {
}
