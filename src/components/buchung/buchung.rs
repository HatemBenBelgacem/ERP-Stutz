//! Auf Projekte gebuchte Stunden (Buchung N:1 Projekt, N:1 Mitarbeiter): Liste aller
//! Buchungen und eine schnelle Erfassungsmaske „Stunden erfassen“.

use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::mitarbeiter_laden;
use crate::backend::projekte::{BuchungLoeschen, BuchungSpeichern, buchungen_laden, projekte_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Bereich, Buchung, Mitarbeiter, Projekt, stunden_anzeige};

const ANZEIGE_ERFASST: &str = "%d.%m.%Y %H:%M";
/// So viele Buchungen zeigt „Stunden erfassen“ unter der Maske
const ZULETZT_ANZAHL: usize = 10;

/// Liest die Zeit aus dem Formular; Komma und Punkt sind erlaubt.
fn zeit_aus(text: &str) -> Result<f64, String> {
    match text.trim().replace(',', ".").parse::<f64>() {
        Ok(zeit) if zeit.is_finite() && zeit > 0.0 => Ok(zeit),
        _ => Err("Die Zeit muss eine Zahl grösser als 0 sein, z. B. 1.5.".into()),
    }
}

fn projekt_text(p: &Projekt) -> String {
    format!("{} – {}", p.projektnummer, p.bezeichnung)
}

/// Auswahlliste der Projekte.
#[component]
fn ProjektAuswahl(projekte: Resource<Result<Vec<Projekt>, ServerFnError>>, projekt_id: RwSignal<String>) -> impl IntoView {
    view! {
        <select class="select w-full" required
            on:change:target=move |ev| projekt_id.set(ev.target().value())>
            <option value="" disabled selected=move || projekt_id.get().is_empty()>"Projekt wählen …"</option>
            <Suspense>
                {move || Suspend::new(async move {
                    projekte.await.unwrap_or_default().into_iter().map(|p| {
                        let id = p.id.to_string();
                        let wert = id.clone();
                        view! {
                            <option value=id selected=move || projekt_id.get() == wert>{projekt_text(&p)}</option>
                        }
                    }).collect_view()
                })}
            </Suspense>
        </select>
    }
}

/// Auswahlliste der Mitarbeitenden.
#[component]
fn MitarbeiterAuswahl(
    mitarbeiter: Resource<Result<Vec<Mitarbeiter>, ServerFnError>>,
    mitarbeiter_id: RwSignal<String>,
) -> impl IntoView {
    view! {
        <select class="select w-full" required
            on:change:target=move |ev| mitarbeiter_id.set(ev.target().value())>
            <option value="" disabled selected=move || mitarbeiter_id.get().is_empty()>"Mitarbeiter wählen …"</option>
            <Suspense>
                {move || Suspend::new(async move {
                    mitarbeiter.await.unwrap_or_default().into_iter().map(|m| {
                        let id = m.id.to_string();
                        let wert = id.clone();
                        view! {
                            <option value=id selected=move || mitarbeiter_id.get() == wert>{m.name()}</option>
                        }
                    }).collect_view()
                })}
            </Suspense>
        </select>
    }
}

/// Liest Projekt, Mitarbeiter und Zeit aus dem Formular.
fn eingabe_aus(projekt_id: &str, mitarbeiter_id: &str, zeit: &str) -> Result<(Uuid, Uuid, f64), String> {
    let projekt = projekt_id.parse().map_err(|_| "Bitte ein Projekt auswählen.".to_string())?;
    let mitarbeiter = mitarbeiter_id.parse().map_err(|_| "Bitte einen Mitarbeiter auswählen.".to_string())?;
    Ok((projekt, mitarbeiter, zeit_aus(zeit)?))
}

/// Tabelle der Buchungen; `aktionen` liefert pro Zeile die Knöpfe rechts.
#[component]
fn BuchungTabelle<A, V>(liste: Vec<Buchung>, aktionen: A) -> impl IntoView
where
    A: Fn(Buchung) -> V + 'static,
    V: IntoView + 'static,
{
    let summe: f64 = liste.iter().map(|b| b.zeit).sum();
    view! {
        <table class="table table-zebra max-sm:table-sm">
            <thead><tr><th>"Projekt"</th><th class="hidden md:table-cell">"Mitarbeiter"</th><th class="hidden sm:table-cell">"Erfasst am"</th><th class="text-right">"Zeit"</th><th></th></tr></thead>
            <tbody>
                {liste.into_iter().map(|b| {
                    let erfasst = b.erfasst_am.format(ANZEIGE_ERFASST).to_string();
                    let erfasst_kurz = erfasst.clone();
                    let mitarbeiter = b.mitarbeiter_name.clone().unwrap_or_else(|| "–".into());
                    let mitarbeiter_kurz = mitarbeiter.clone();
                    view! {
                        <tr class="hover:bg-base-300/60">
                            <td>
                                <div class="font-medium">{b.projektnummer.clone()}</div>
                                <div class="text-xs text-base-content/60">{b.projekt_bezeichnung.clone()}</div>
                                <div class="text-xs text-base-content/60 md:hidden">{mitarbeiter_kurz}</div>
                                // Auf dem Smartphone auch das Erfassungsdatum
                                <div class="text-xs text-base-content/60 sm:hidden">{erfasst_kurz}</div>
                            </td>
                            <td class="hidden md:table-cell">{mitarbeiter}</td>
                            <td class="hidden whitespace-nowrap sm:table-cell">{erfasst}</td>
                            <td class="whitespace-nowrap text-right tabular-nums">{stunden_anzeige(b.zeit)}</td>
                            <td>{aktionen(b)}</td>
                        </tr>
                    }
                }).collect_view()}
            </tbody>
            <tfoot>
                <tr><th>"Total"</th><th class="hidden md:table-cell"></th><th class="hidden sm:table-cell"></th><th class="text-right tabular-nums">{stunden_anzeige(summe)}</th><th></th></tr>
            </tfoot>
        </table>
    }
}

#[component]
pub fn BuchungSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Buchungen, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Buchungen, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Buchungen, Aktion::Loeschen);
    let speichern = ServerAction::<BuchungSpeichern>::new();
    let loeschen = ServerAction::<BuchungLoeschen>::new();
    let buchungen = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| buchungen_laden(),
    );
    let projekte = Resource::new(|| (), |_| projekte_laden());
    let mitarbeiter = Resource::new(|| (), |_| mitarbeiter_laden());

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let projekt_id = RwSignal::new(String::new());
    let mitarbeiter_id = RwSignal::new(String::new());
    let zeit = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        projekt_id.set(String::new());
        mitarbeiter_id.set(String::new());
        zeit.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    let bearbeiten = move |b: Buchung| {
        bearbeite.set(Some(b.id));
        projekt_id.set(b.projekt_id.to_string());
        mitarbeiter_id.set(b.mitarbeiter_id.map(|m| m.to_string()).unwrap_or_default());
        zeit.set(b.zeit.to_string());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match eingabe_aus(&projekt_id.get_untracked(), &mitarbeiter_id.get_untracked(), &zeit.get_untracked()) {
            Ok((projekt_id, mitarbeiter_id, zeit)) => {
                speichern.dispatch(BuchungSpeichern { id: bearbeite.get_untracked(), projekt_id, mitarbeiter_id, zeit });
            }
            Err(m) => formular_meldung.set(Some(m)),
        }
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Buchung bearbeiten" } else { "Neue Buchung" }.to_string()
    });

    view! {
        <Seitenkopf titel="Buchungen" untertitel="Auf Projekte gebuchte Stunden">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuPlus/>
                    "Neue Buchung"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match buchungen.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuReceipt text="Noch keine Buchungen erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <BuchungTabelle liste aktionen=move |b: Buchung| {
                                    let id = b.id;
                                    view! {
                                        <ZeilenAktionen
                                            bearbeiten=move || bearbeiten(b.clone())
                                            loeschen=move || {
                                                if bestaetigt("Buchung wirklich löschen?") {
                                                    loeschen.dispatch(BuchungLoeschen { id });
                                                }
                                            }
                                            darf_bearbeiten=move || darf_bearbeiten.get()
                                            darf_loeschen=move || darf_loeschen.get()
                                        />
                                    }
                                }/>
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
                    <legend class="fieldset-legend">"Projekt"</legend>
                    <ProjektAuswahl projekte projekt_id/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Mitarbeiter"</legend>
                    <MitarbeiterAuswahl mitarbeiter mitarbeiter_id/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Zeit (Stunden)"</legend>
                    <input class="input w-full" type="number" min="0.01" step="0.01" required bind:value=zeit/>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}

/// Schnelle Erfassung: Projekt wählen, Zeit eingeben, speichern. Danach wird
/// die Maske neu geladen (Mitarbeiter und Projekt bleiben, Zeit ist leer) für die nächste Buchung.
#[component]
pub fn StundenSeite() -> impl IntoView {
    let speichern = ServerAction::<BuchungSpeichern>::new();
    let buchungen = Resource::new(move || speichern.version().get(), |_| buchungen_laden());
    let projekte = Resource::new(|| (), |_| projekte_laden());
    let mitarbeiter = Resource::new(|| (), |_| mitarbeiter_laden());

    let projekt_id = RwSignal::new(String::new());
    let mitarbeiter_id = RwSignal::new(String::new());
    let zeit = RwSignal::new(String::new());
    let meldung = RwSignal::new(None::<String>);
    let hinweis = RwSignal::new(None::<String>);

    ergebnis_beobachten(speichern.value(), meldung, move || {
        zeit.set(String::new());
        hinweis.set(Some("Stunden gebucht.".into()));
    });

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        hinweis.set(None);
        match eingabe_aus(&projekt_id.get_untracked(), &mitarbeiter_id.get_untracked(), &zeit.get_untracked()) {
            Ok((projekt_id, mitarbeiter_id, zeit)) => {
                speichern.dispatch(BuchungSpeichern { id: None, projekt_id, mitarbeiter_id, zeit });
            }
            Err(m) => meldung.set(Some(m)),
        }
    };
    let beschaeftigt = speichern.pending();

    view! {
        <Seitenkopf titel="Stunden erfassen" untertitel="Arbeitszeit auf ein Projekt buchen">
            {()}
        </Seitenkopf>

        <div class="card mb-6 max-w-2xl border border-base-300 bg-base-100">
            <form class="card-body" on:submit=absenden>
                <Show when=move || hinweis.with(Option::is_some)>
                    <div role="status" class="alert alert-success alert-soft">
                        <Icon icon=icondata::LuCircleCheck/>
                        <span>{move || hinweis.get().unwrap_or_default()}</span>
                    </div>
                </Show>
                <Fehlermeldung meldung/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Mitarbeiter"</legend>
                    <MitarbeiterAuswahl mitarbeiter mitarbeiter_id/>
                </fieldset>
                <div class="grid gap-x-4 sm:grid-cols-[1fr_10rem]">
                    <fieldset class="fieldset min-w-0">
                        <legend class="fieldset-legend">"Projekt"</legend>
                        <ProjektAuswahl projekte projekt_id/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Zeit (Stunden)"</legend>
                        <input class="input w-full" type="number" min="0.01" step="0.01" required
                            placeholder="z. B. 1.5" bind:value=zeit/>
                    </fieldset>
                </div>
                <div class="card-actions justify-end">
                    <button type="submit" class="btn btn-primary" disabled=move || beschaeftigt.get()>
                        <Show when=move || beschaeftigt.get() fallback=|| view! { <Icon icon=icondata::LuClockPlus/> }>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        "Stunden buchen"
                    </button>
                </div>
            </form>
        </div>

        <h2 class="mb-3 font-semibold">"Zuletzt erfasst"</h2>
        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match buchungen.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuClock text="Noch keine Stunden erfasst."/>
                            }.into_any(),
                            Ok(mut liste) => {
                                liste.truncate(ZULETZT_ANZAHL);
                                view! { <BuchungTabelle liste aktionen=|_| ()/> }.into_any()
                            }
                            Err(e) => view! { <div class="alert alert-error m-4">{meldung_aus(e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </div>
    }
}
