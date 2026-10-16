use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::api::{TerminLoeschen, TerminSpeichern, kurse_laden, mitarbeiter_laden, termine_laden};
use crate::components::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};
use crate::models::{DATETIME_LOCAL, Mitarbeiter, TerminEingabe};

const ANZEIGE_DATUM: &str = "%d.%m.%Y";
const ANZEIGE_ZEIT: &str = "%H:%M";

#[component]
pub fn TerminSeite() -> impl IntoView {
    let speichern = ServerAction::<TerminSpeichern>::new();
    let loeschen = ServerAction::<TerminLoeschen>::new();
    let termine = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| termine_laden(),
    );
    // Auswahllisten für Kurs (N:1) und Teilnehmende (M:N)
    let kurse = Resource::new(|| (), |_| kurse_laden());
    let mitarbeiter = Resource::new(|| (), |_| mitarbeiter_laden());

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let kurs_id = RwSignal::new(String::new());
    let beginn = RwSignal::new(String::new());
    let ende = RwSignal::new(String::new());
    let ort = RwSignal::new(String::new());
    let ausgewaehlt = RwSignal::new(Vec::<Uuid>::new());
    let suche = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        kurs_id.set(String::new());
        beginn.set(String::new());
        ende.set(String::new());
        ort.set(String::new());
        ausgewaehlt.set(Vec::new());
        suche.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Ok(kurs) = kurs_id.get_untracked().parse::<Uuid>() else {
            formular_meldung.set(Some("Bitte einen Kurs auswählen.".into()));
            return;
        };
        speichern.dispatch(TerminSpeichern {
            id: bearbeite.get_untracked(),
            eingabe: TerminEingabe {
                kurs_id: kurs,
                beginn: beginn.get_untracked(),
                ende: ende.get_untracked(),
                ort: ort.get_untracked(),
                mitarbeiter_ids: ausgewaehlt.get_untracked(),
            },
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Termin bearbeiten" } else { "Neuer Termin" }.to_string()
    });

    view! {
        <Seitenkopf titel="Termine" untertitel="Kursdurchführungen und ihre Teilnehmenden">
            <button class="btn btn-primary" on:click=move |_| neu()>
                <Icon icon=icondata::LuPlus/>
                "Neuer Termin"
            </button>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match termine.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuCalendar text="Noch keine Termine erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table">
                                    <thead><tr><th>"Kurs"</th><th>"Datum"</th><th>"Ort"</th><th>"Teilnehmende"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|t| {
                                            let id = t.id;
                                            let zum_bearbeiten = t.clone();
                                            let datum = if t.beginn.date() == t.ende.date() {
                                                t.beginn.format(ANZEIGE_DATUM).to_string()
                                            } else {
                                                format!("{} – {}", t.beginn.format(ANZEIGE_DATUM), t.ende.format(ANZEIGE_DATUM))
                                            };
                                            let zeit = format!("{} – {}", t.beginn.format(ANZEIGE_ZEIT), t.ende.format(ANZEIGE_ZEIT));
                                            view! {
                                                <tr class="hover:bg-base-200/50">
                                                    <td class="font-medium">{t.kurs_titel}</td>
                                                    <td class="whitespace-nowrap">
                                                        <div>{datum}</div>
                                                        <div class="text-xs text-base-content/60">{zeit}</div>
                                                    </td>
                                                    <td>{t.ort}</td>
                                                    <td>
                                                        <div class="flex flex-wrap gap-1">
                                                            {t.mitarbeiter.iter().map(|m| view! {
                                                                <span class="badge badge-sm badge-primary badge-soft">{m.name()}</span>
                                                            }).collect_view()}
                                                        </div>
                                                    </td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=move || {
                                                                let t = zum_bearbeiten.clone();
                                                                bearbeite.set(Some(t.id));
                                                                kurs_id.set(t.kurs_id.to_string());
                                                                beginn.set(t.beginn.format(DATETIME_LOCAL).to_string());
                                                                ende.set(t.ende.format(DATETIME_LOCAL).to_string());
                                                                ort.set(t.ort);
                                                                ausgewaehlt.set(t.mitarbeiter.iter().map(|m| m.id).collect());
                                                                suche.set(String::new());
                                                                formular_meldung.set(None);
                                                                dialog_offen.set(true);
                                                            }
                                                            loeschen=move || {
                                                                if bestaetigt("Termin wirklich löschen?") {
                                                                    loeschen.dispatch(TerminLoeschen { id });
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
                    <legend class="fieldset-legend">"Kurs"</legend>
                    <select class="select w-full" required
                        on:change:target=move |ev| kurs_id.set(ev.target().value())>
                        <option value="" disabled selected=move || kurs_id.get().is_empty()>"Kurs wählen …"</option>
                        <Suspense>
                            {move || Suspend::new(async move {
                                kurse.await.unwrap_or_default().into_iter().map(|k| {
                                    let id = k.id.to_string();
                                    let wert = id.clone();
                                    view! {
                                        <option value=id selected=move || kurs_id.get() == wert>{k.titel}</option>
                                    }
                                }).collect_view()
                            })}
                        </Suspense>
                    </select>
                </fieldset>
                <div class="grid gap-x-4 sm:grid-cols-2">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Beginn"</legend>
                        <input class="input w-full" type="datetime-local" required bind:value=beginn/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Ende"</legend>
                        <input class="input w-full" type="datetime-local" required bind:value=ende/>
                    </fieldset>
                </div>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Ort"</legend>
                    <input class="input w-full" bind:value=ort/>
                </fieldset>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">
                        "Teilnehmende"
                        <span class="badge badge-sm badge-ghost">{move || ausgewaehlt.with(Vec::len)}</span>
                    </legend>
                    <Suspense fallback=Laden>
                        {move || Suspend::new(async move {
                            let liste = mitarbeiter.await.unwrap_or_default();
                            view! { <MitarbeiterAuswahl liste ausgewaehlt suche/> }
                        })}
                    </Suspense>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}

/// Suchbare Mehrfachauswahl der Teilnehmenden. Ausgewählte stehen als Badges
/// oberhalb der Suche und bleiben sichtbar, auch wenn der Filter sie ausblendet.
#[component]
fn MitarbeiterAuswahl(
    liste: Vec<Mitarbeiter>,
    ausgewaehlt: RwSignal<Vec<Uuid>>,
    suche: RwSignal<String>,
) -> impl IntoView {
    if liste.is_empty() {
        return view! { <p class="p-2 text-base-content/60">"Noch keine Mitarbeiter erfasst."</p> }.into_any();
    }
    let gesamt = liste.len();
    let liste = StoredValue::new(liste);

    let umschalten = move |id: Uuid, an: bool| {
        ausgewaehlt.update(|v| {
            v.retain(|x| *x != id);
            if an {
                v.push(id);
            }
        });
    };

    // Jedes Suchwort muss in Name, E-Mail oder Abteilung vorkommen.
    let treffer = move || {
        let woerter: Vec<String> = suche.with(|s| s.to_lowercase().split_whitespace().map(String::from).collect());
        liste.with_value(|l| {
            l.iter()
                .filter(|m| {
                    let text = format!("{} {} {} {}", m.vorname, m.nachname, m.email, m.abteilung).to_lowercase();
                    woerter.iter().all(|w| text.contains(w.as_str()))
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    let gewaehlte = move || {
        liste.with_value(|l| {
            l.iter()
                .filter(|m| ausgewaehlt.with(|a| a.contains(&m.id)))
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    view! {
        <Show when=move || ausgewaehlt.with(|a| !a.is_empty())>
            <div class="flex flex-wrap gap-1 pb-1">
                <For each=gewaehlte key=|m| m.id let:m>
                    <span class="badge badge-primary badge-soft gap-1 pr-1">
                        {m.name()}
                        <button type="button" class="btn btn-ghost btn-xs btn-circle h-4 min-h-4 w-4"
                            aria-label=format!("{} entfernen", m.name())
                            on:click=move |_| umschalten(m.id, false)>
                            <Icon icon=icondata::LuX width="0.75rem" height="0.75rem"/>
                        </button>
                    </span>
                </For>
            </div>
        </Show>

        <label class="input w-full">
            <Icon icon=icondata::LuSearch attr:class="opacity-50"/>
            <input
                type="search"
                class="grow"
                placeholder="Name, E-Mail oder Abteilung suchen …"
                bind:value=suche
                // Enter soll nicht das ganze Formular absenden
                on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); }
            />
        </label>

        <div class="max-h-56 overflow-y-auto rounded-box border border-base-300 p-1">
            <For each=treffer key=|m| m.id let:m>
                <label class="label w-full cursor-pointer rounded-field px-2 py-1.5 hover:bg-base-200">
                    <input type="checkbox" class="checkbox checkbox-sm checkbox-primary"
                        prop:checked=move || ausgewaehlt.with(|v| v.contains(&m.id))
                        on:change:target=move |ev| umschalten(m.id, ev.target().checked())/>
                    <span class="flex flex-col">
                        <span class="text-base-content">{m.name()}</span>
                        <span class="text-xs">
                            {if m.abteilung.is_empty() { m.email.clone() } else { format!("{} · {}", m.abteilung, m.email) }}
                        </span>
                    </span>
                </label>
            </For>
            <Show when=move || suche.with(|s| !s.trim().is_empty()) && treffer().is_empty()>
                <p class="p-2 text-base-content/60">"Keine Treffer."</p>
            </Show>
        </div>
        <p class="label">{move || format!("{} von {} Mitarbeitenden angezeigt", treffer().len(), gesamt)}</p>
    }
    .into_any()
}
