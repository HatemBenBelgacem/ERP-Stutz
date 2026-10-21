use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::{Recht, use_sitzung};
use crate::backend::api::{
    TerminLoeschen, TerminSpeichern, adressen_laden, kurse_laden, mitarbeiter_laden, termine_laden,
};
use crate::components::gemeinsam::{
    Dialog, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen, bestaetigt, ergebnis_beobachten,
    meldung_aus,
};
use crate::models::{Aktion, Bereich, DATETIME_LOCAL, Mitarbeiter, Termin, TerminEingabe};

const ANZEIGE_DATUM: &str = "%d.%m.%Y";
const ANZEIGE_ZEIT: &str = "%H:%M";
const STANDARD_MAX_TEILNEHMER: &str = "20";

/// Zustand der Termin-Eingabemaske. Die Termin- und die Kursseite nutzen
/// dieselbe Maske (siehe [`TerminDialog`]).
#[derive(Clone, Copy)]
pub struct TerminMaske {
    offen: RwSignal<bool>,
    /// Gesetzt, wenn ein bestehender Termin geändert wird
    bearbeite: RwSignal<Option<Uuid>>,
    kurs_id: RwSignal<String>,
    beginn: RwSignal<String>,
    ende: RwSignal<String>,
    adresse_id: RwSignal<String>,
    max_teilnehmer: RwSignal<String>,
    ausgewaehlt: RwSignal<Vec<Uuid>>,
    suche: RwSignal<String>,
    meldung: RwSignal<Option<String>>,
    /// Erfolgsmeldung, nachdem die Maske für einen weiteren Termin neu geladen wurde
    hinweis: RwSignal<Option<String>>,
    /// Nach dem Speichern gleich den nächsten Termin zum gleichen Kurs erfassen
    weiterer: RwSignal<bool>,
}

impl Default for TerminMaske {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminMaske {
    pub fn new() -> Self {
        Self {
            offen: RwSignal::new(false),
            bearbeite: RwSignal::new(None),
            kurs_id: RwSignal::new(String::new()),
            beginn: RwSignal::new(String::new()),
            ende: RwSignal::new(String::new()),
            adresse_id: RwSignal::new(String::new()),
            max_teilnehmer: RwSignal::new(STANDARD_MAX_TEILNEHMER.to_string()),
            ausgewaehlt: RwSignal::new(Vec::new()),
            suche: RwSignal::new(String::new()),
            meldung: RwSignal::new(None),
            hinweis: RwSignal::new(None),
            weiterer: RwSignal::new(false),
        }
    }

    /// Leere Maske für einen neuen Termin, optional mit vorgewähltem Kurs.
    pub fn neu(&self, kurs: Option<Uuid>) {
        self.vorlage(kurs, None, STANDARD_MAX_TEILNEHMER.to_string());
    }

    /// Neuer Termin zum gleichen Kurs wie `t`; Adresse und Plätze werden übernommen.
    pub fn gleicher_kurs(&self, t: &Termin) {
        self.vorlage(Some(t.kurs_id), t.adresse.as_ref().map(|a| a.id), t.max_teilnehmer.to_string());
    }

    pub fn bearbeiten(&self, t: Termin) {
        self.bearbeite.set(Some(t.id));
        self.kurs_id.set(t.kurs_id.to_string());
        self.beginn.set(t.beginn.format(DATETIME_LOCAL).to_string());
        self.ende.set(t.ende.format(DATETIME_LOCAL).to_string());
        self.adresse_id.set(t.adresse.map(|a| a.id.to_string()).unwrap_or_default());
        self.max_teilnehmer.set(t.max_teilnehmer.to_string());
        self.ausgewaehlt.set(t.mitarbeiter.iter().map(|m| m.id).collect());
        self.zuruecksetzen();
    }

    fn vorlage(&self, kurs: Option<Uuid>, adresse: Option<Uuid>, max_teilnehmer: String) {
        self.bearbeite.set(None);
        self.kurs_id.set(kurs.map(|k| k.to_string()).unwrap_or_default());
        self.beginn.set(String::new());
        self.ende.set(String::new());
        self.adresse_id.set(adresse.map(|a| a.to_string()).unwrap_or_default());
        self.max_teilnehmer.set(max_teilnehmer);
        self.ausgewaehlt.set(Vec::new());
        self.zuruecksetzen();
    }

    fn zuruecksetzen(&self) {
        self.suche.set(String::new());
        self.meldung.set(None);
        self.hinweis.set(None);
        self.offen.set(true);
    }

    /// Nach dem Speichern: Maske neu laden für den nächsten Termin zum gleichen
    /// Kurs. Kurs, Adresse und Plätze bleiben, Datum und Teilnehmende werden geleert.
    fn naechster_termin(&self) {
        let kurs = self.kurs_id.get_untracked().parse().ok();
        let adresse = self.adresse_id.get_untracked().parse().ok();
        self.vorlage(kurs, adresse, self.max_teilnehmer.get_untracked());
        self.hinweis.set(Some("Termin gespeichert – jetzt den nächsten Termin zum gleichen Kurs erfassen.".into()));
    }
}

#[component]
pub fn TerminSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Termine, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Termine, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Termine, Aktion::Loeschen);
    let speichern = ServerAction::<TerminSpeichern>::new();
    let loeschen = ServerAction::<TerminLoeschen>::new();
    let termine = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| termine_laden(),
    );

    let maske = TerminMaske::new();
    let listen_meldung = RwSignal::new(None::<String>);
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let termin_bearbeiten = move |t: Termin| maske.bearbeiten(t);

    // Termin, dessen Teilnehmende rechts angezeigt werden; Daten immer aus der aktuellen Liste
    let panel_termin = RwSignal::new(None::<Uuid>);
    let panel_daten = move || {
        let id = panel_termin.get()?;
        termine.get()?.ok()?.into_iter().find(|t| t.id == id)
    };

    view! {
        <Seitenkopf titel="Termine" untertitel="Kursdurchführungen und ihre Teilnehmenden">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| maske.neu(None)>
                    <Icon icon=icondata::LuPlus/>
                    "Neuer Termin"
                </button>
            </Show>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="flex items-start gap-4">
            <div class="card min-w-0 flex-1 border border-base-300 bg-base-100">
                <div class="overflow-x-auto">
                    <Transition fallback=Laden>
                        {move || Suspend::new(async move {
                            match termine.await {
                                Ok(liste) if liste.is_empty() => view! {
                                    <Leer symbol=icondata::LuCalendar text="Noch keine Termine erfasst."/>
                                }.into_any(),
                                Ok(liste) => view! {
                                    <table class="table table-zebra max-sm:table-sm">
                                        <thead>
                                            <tr>
                                                <th>"Kurs"</th>
                                                <th class="hidden sm:table-cell">"Datum"</th>
                                                <th class="hidden lg:table-cell">"Veranstaltungsort"</th>
                                                <th>
                                                    <span class="sm:hidden">"Plätze"</span>
                                                    <span class="hidden sm:inline">"Teilnehmende"</span>
                                                </th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {liste.into_iter().map(|t| {
                                                let id = t.id;
                                                let fuer_doppelklick = t.clone();
                                                let fuer_button = t.clone();
                                                let fuer_gleichen_kurs = t.clone();
                                                let datum = if t.beginn.date() == t.ende.date() {
                                                    t.beginn.format(ANZEIGE_DATUM).to_string()
                                                } else {
                                                    format!("{} – {}", t.beginn.format(ANZEIGE_DATUM), t.ende.format(ANZEIGE_DATUM))
                                                };
                                                let zeit = format!("{} – {}", t.beginn.format(ANZEIGE_ZEIT), t.ende.format(ANZEIGE_ZEIT));
                                                let datum_kurz = datum.clone();
                                                let zeit_kurz = zeit.clone();
                                                let adresse_kurz = t.adresse.as_ref().map(|a| a.anzeige()).unwrap_or_default();
                                                let adresse_zeilen = t.adresse.as_ref().map(|a| a.zeilen()).unwrap_or_default();
                                                let belegt = t.mitarbeiter.len();
                                                let voll = belegt >= t.max_teilnehmer as usize;
                                                let plaetze = format!("{belegt} / {}", t.max_teilnehmer);
                                                let ist_offen = move || panel_termin.get() == Some(id);
                                                view! {
                                                    <tr class="select-none hover:bg-base-300/60"
                                                        class=("bg-primary/5", ist_offen)
                                                        class:cursor-pointer=move || darf_bearbeiten.get()
                                                        title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                        on:dblclick=move |_| if darf_bearbeiten.get_untracked() {
                                                            termin_bearbeiten(fuer_doppelklick.clone())
                                                        }>
                                                        <td>
                                                            <div class="font-medium">{t.kurs_titel}</div>
                                                            // Auf kleinen Bildschirmen steht der Veranstaltungsort unter dem Kurs
                                                            <div class="text-xs text-base-content/60 lg:hidden">{adresse_kurz}</div>
                                                            // Auf dem Smartphone auch Datum und Zeit
                                                            <div class="text-xs text-base-content/60 sm:hidden">{format!("{datum_kurz} · {zeit_kurz}")}</div>
                                                        </td>
                                                        <td class="hidden whitespace-nowrap sm:table-cell">
                                                            <div>{datum}</div>
                                                            <div class="text-xs text-base-content/60">{zeit}</div>
                                                        </td>
                                                        // Bezeichnung, Strasse und PLZ/Ort je auf einer Zeile
                                                        <td class="hidden lg:table-cell">
                                                            {adresse_zeilen.into_iter().enumerate().map(|(i, zeile)| view! {
                                                                <div class:font-medium=i == 0 class=("text-xs", i > 0) class=("text-base-content/60", i > 0)>{zeile}</div>
                                                            }).collect_view()}
                                                        </td>
                                                        <td>
                                                            <button type="button"
                                                                class="btn btn-sm gap-1.5 whitespace-nowrap max-sm:px-2"
                                                                class=(["btn-primary", "btn-soft"], ist_offen)
                                                                class=("btn-ghost", move || !ist_offen())
                                                                class:text-warning=move || voll && !ist_offen()
                                                                title=if voll { "Ausgebucht – Teilnehmende anzeigen" } else { "Teilnehmende anzeigen" }
                                                                aria-label="Teilnehmende anzeigen"
                                                                aria-pressed=move || ist_offen().to_string()
                                                                on:dblclick=|ev| ev.stop_propagation()
                                                                on:click=move |_| panel_termin.update(|p| {
                                                                    *p = if *p == Some(id) { None } else { Some(id) };
                                                                })>
                                                                <Icon icon=icondata::LuUsers/>
                                                                {plaetze}
                                                            </button>
                                                        </td>
                                                        <td>
                                                            <div class="flex justify-end gap-1">
                                                                <Show when=move || darf_anlegen.get()>
                                                                    <button class="btn btn-ghost btn-sm btn-square"
                                                                        title="Neuer Termin zum gleichen Kurs"
                                                                        aria-label="Neuer Termin zum gleichen Kurs"
                                                                        on:dblclick=|ev| ev.stop_propagation()
                                                                        on:click={
                                                                            let t = fuer_gleichen_kurs.clone();
                                                                            move |_| maske.gleicher_kurs(&t)
                                                                        }>
                                                                        <Icon icon=icondata::LuCopyPlus/>
                                                                    </button>
                                                                </Show>
                                                                <PdfKnopf termin_id=id/>
                                                                <ZeilenAktionen
                                                                    bearbeiten=move || termin_bearbeiten(fuer_button.clone())
                                                                    loeschen=move || {
                                                                        if bestaetigt("Termin wirklich löschen?") {
                                                                            loeschen.dispatch(TerminLoeschen { id });
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

            // Teilnehmende des gewählten Termins: ab xl rechts neben der Liste,
            // darunter als Panel, das von rechts über den Inhalt fährt.
            <Transition>
                {move || panel_daten().map(|t| view! {
                    <div class="fixed inset-0 z-40 bg-base-content/20 xl:hidden" on:click=move |_| panel_termin.set(None)></div>
                    <aside class="fixed inset-y-0 right-0 z-50 w-full sm:w-96 xl:sticky xl:top-0 xl:z-auto xl:shrink-0"
                        aria-label="Teilnehmende">
                        <TeilnehmerPanel
                            termin=t
                            schliessen=move || panel_termin.set(None)
                            bearbeiten=move |t| termin_bearbeiten(t)
                            darf_bearbeiten
                        />
                    </aside>
                })}
            </Transition>
        </div>

        <TerminDialog maske speichern/>
    }
}

/// Eingabemaske für einen Termin. Mit „Speichern & nächster Termin“ wird die
/// Maske nach dem Speichern neu geladen, um gleich den nächsten Termin zum
/// gleichen Kurs zu erfassen.
#[component]
pub fn TerminDialog(maske: TerminMaske, speichern: ServerAction<TerminSpeichern>) -> impl IntoView {
    let darf_anlegen = use_sitzung().recht(Bereich::Termine, Aktion::Anlegen);
    // Auswahllisten für Kurs (N:1), Adresse (N:1) und Teilnehmende (M:N)
    let kurse = Resource::new(|| (), |_| kurse_laden());
    let adressen = Resource::new(|| (), |_| adressen_laden());
    let mitarbeiter = Resource::new(|| (), |_| mitarbeiter_laden());

    let TerminMaske { offen, bearbeite, kurs_id, beginn, ende, adresse_id, max_teilnehmer, ausgewaehlt, suche, meldung, hinweis, weiterer } = maske;

    ergebnis_beobachten(speichern.value(), meldung, move || {
        if weiterer.get_untracked() {
            maske.naechster_termin();
        } else {
            offen.set(false);
        }
    });

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        hinweis.set(None);
        let Ok(kurs) = kurs_id.get_untracked().parse::<Uuid>() else {
            meldung.set(Some("Bitte einen Kurs auswählen.".into()));
            return;
        };
        let Some(max) = max_teilnehmer.get_untracked().trim().parse::<i32>().ok().filter(|m| *m >= 1) else {
            meldung.set(Some("Die maximale Teilnehmerzahl muss eine ganze Zahl ab 1 sein.".into()));
            return;
        };
        speichern.dispatch(TerminSpeichern {
            id: bearbeite.get_untracked(),
            eingabe: TerminEingabe {
                kurs_id: kurs,
                beginn: beginn.get_untracked(),
                ende: ende.get_untracked(),
                adresse_id: adresse_id.get_untracked().parse().ok(),
                max_teilnehmer: max,
                mitarbeiter_ids: ausgewaehlt.get_untracked(),
            },
        });
    };

    // Gültige Obergrenze aus dem Formular, `None` solange das Feld ungültig ist
    let max_plaetze = Signal::derive(move || {
        max_teilnehmer.with(|m| m.trim().parse::<usize>().ok().filter(|m| *m >= 1))
    });

    let ueberbucht = move || max_plaetze.get().is_some_and(|max| ausgewaehlt.with(Vec::len) > max);

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Termin bearbeiten" } else { "Neuer Termin" }.to_string()
    });
    let beschaeftigt = speichern.pending();

    view! {
        <Dialog offen titel=dialog_titel>
            <form on:submit=absenden>
                <Show when=move || hinweis.with(Option::is_some)>
                    <div role="status" class="alert alert-success alert-soft mb-4">
                        <Icon icon=icondata::LuCircleCheck/>
                        <span>{move || hinweis.get().unwrap_or_default()}</span>
                    </div>
                </Show>
                <Fehlermeldung meldung/>
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
                <div class="grid gap-x-4 sm:grid-cols-[1fr_10rem]">
                    <fieldset class="fieldset min-w-0">
                        <legend class="fieldset-legend">"Veranstaltungsort"</legend>
                        <select class="select w-full"
                            on:change:target=move |ev| adresse_id.set(ev.target().value())>
                            <option value="" selected=move || adresse_id.get().is_empty()>"Kein Veranstaltungsort"</option>
                            <Suspense>
                                {move || Suspend::new(async move {
                                    adressen.await.unwrap_or_default().into_iter().map(|a| {
                                        let id = a.id.to_string();
                                        let wert = id.clone();
                                        view! {
                                            <option value=id selected=move || adresse_id.get() == wert>{a.anzeige()}</option>
                                        }
                                    }).collect_view()
                                })}
                            </Suspense>
                        </select>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Max. Teilnehmende"</legend>
                        <input class="input w-full" type="number" min="1" required bind:value=max_teilnehmer/>
                    </fieldset>
                </div>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">
                        "Teilnehmende"
                        <span class="badge badge-sm"
                            class:badge-ghost=move || !ueberbucht()
                            class:badge-error=ueberbucht>
                            {move || match max_plaetze.get() {
                                Some(max) => format!("{} / {max}", ausgewaehlt.with(Vec::len)),
                                None => ausgewaehlt.with(Vec::len).to_string(),
                            }}
                        </span>
                    </legend>
                    <Suspense fallback=Laden>
                        {move || Suspend::new(async move {
                            let liste = mitarbeiter.await.unwrap_or_default();
                            view! { <MitarbeiterAuswahl liste ausgewaehlt suche max_plaetze/> }
                        })}
                    </Suspense>
                </fieldset>

                // Beide Knöpfe senden das Formular ab (inkl. Pflichtfeld-Prüfung);
                // der Klick merkt sich vorher, was danach passieren soll.
                <div class="modal-action flex-wrap">
                    <button type="button" class="btn btn-ghost" on:click=move |_| offen.set(false)>
                        <Icon icon=icondata::LuX/>
                        "Abbrechen"
                    </button>
                    <Show when=move || darf_anlegen.get()>
                        <button type="submit" class="btn btn-primary btn-soft"
                            title="Speichern und gleich einen weiteren Termin zum gleichen Kurs erfassen"
                            disabled=move || beschaeftigt.get()
                            on:click=move |_| weiterer.set(true)>
                            <Icon icon=icondata::LuCalendarPlus/>
                            "Speichern & nächster Termin"
                        </button>
                    </Show>
                    <button type="submit" class="btn btn-primary" disabled=move || beschaeftigt.get()
                        on:click=move |_| weiterer.set(false)>
                        <Show when=move || beschaeftigt.get() fallback=|| view! { <Icon icon=icondata::LuSave/> }>
                            <span class="loading loading-spinner loading-xs"></span>
                        </Show>
                        "Speichern"
                    </button>
                </div>
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
    max_plaetze: Signal<Option<usize>>,
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

    // Bei erreichter Obergrenze lassen sich nur noch Haken entfernen.
    let voll = move || max_plaetze.get().is_some_and(|max| ausgewaehlt.with(Vec::len) >= max);

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

        <Show when=voll>
            <p class="text-xs text-warning">
                {move || format!("Maximal {} Teilnehmende – der Termin ist voll.", max_plaetze.get().unwrap_or_default())}
            </p>
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
                        disabled=move || voll() && !ausgewaehlt.with(|v| v.contains(&m.id))
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

/// Tabelle der Teilnehmenden eines Termins.
#[component]
fn TeilnehmerPanel(
    termin: Termin,
    schliessen: impl Fn() + Copy + 'static,
    bearbeiten: impl Fn(Termin) + Send + Sync + 'static,
    darf_bearbeiten: Recht,
) -> impl IntoView {
    let belegt = termin.mitarbeiter.len();
    let datum = termin.beginn.format(ANZEIGE_DATUM).to_string();
    let plaetze = format!("{belegt} / {} Plätze", termin.max_teilnehmer);
    let voll = belegt >= termin.max_teilnehmer as usize;
    let zum_bearbeiten = termin.clone();
    let termin_id = termin.id;
    let bearbeiten = std::sync::Arc::new(bearbeiten);

    view! {
        <div class="flex h-full flex-col overflow-hidden border-base-300 bg-base-100 shadow-xl sm:border-l xl:h-auto xl:max-h-[calc(100vh-8rem)] xl:rounded-box xl:border xl:shadow-none">
            <div class="flex items-start justify-between gap-2 border-b border-base-300 p-4">
                <div class="min-w-0">
                    <h2 class="truncate font-semibold">{termin.kurs_titel}</h2>
                    <p class="text-sm text-base-content/60">{datum}</p>
                    <span class="badge badge-sm mt-1"
                        class=(["badge-warning", "badge-soft"], voll)
                        class:badge-ghost=!voll>
                        {plaetze}
                    </span>
                </div>
                <button class="btn btn-ghost btn-sm btn-square" aria-label="Teilnehmende schliessen"
                    on:click=move |_| schliessen()>
                    <Icon icon=icondata::LuX/>
                </button>
            </div>
            <div class="flex-1 overflow-y-auto">
                {if termin.mitarbeiter.is_empty() {
                    view! { <Leer symbol=icondata::LuUsers text="Noch keine Teilnehmenden."/> }.into_any()
                } else {
                    view! {
                        <table class="table table-zebra table-sm">
                            <thead><tr><th>"Name"</th><th>"Abteilung"</th></tr></thead>
                            <tbody>
                                {termin.mitarbeiter.into_iter().map(|m| view! {
                                    <tr>
                                        <td>
                                            <div class="font-medium">{m.name()}</div>
                                            <div class="text-xs text-base-content/60 break-all">{m.email}</div>
                                        </td>
                                        <td>{m.abteilung}</td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }}
            </div>
            <div class="flex gap-2 border-t border-base-300 p-3">
                <a class="btn btn-sm btn-ghost flex-1" href=pdf_pfad(termin_id) target="_blank" rel="external">
                    <Icon icon=icondata::LuFileText/>
                    "Als PDF"
                </a>
                <Show when=move || darf_bearbeiten.get()>
                    <button class="btn btn-sm btn-ghost flex-1" on:click={
                        let bearbeiten = bearbeiten.clone();
                        let termin = zum_bearbeiten.clone();
                        move |_| bearbeiten(termin.clone())
                    }>
                        <Icon icon=icondata::LuPencil/>
                        "Bearbeiten"
                    </button>
                </Show>
            </div>
        </div>
    }
}

/// Adresse der Teilnehmerliste als PDF (erzeugt vom Server).
fn pdf_pfad(termin_id: Uuid) -> String {
    format!("/termine/{termin_id}/teilnehmerliste.pdf")
}

/// Öffnet die Teilnehmerliste als PDF in einem neuen Tab.
#[component]
fn PdfKnopf(termin_id: Uuid) -> impl IntoView {
    view! {
        // rel="external": normaler Seitenaufruf statt Router-Navigation
        <a class="btn btn-ghost btn-sm btn-square" href=pdf_pfad(termin_id) target="_blank" rel="external"
            title="Teilnehmerliste als PDF" aria-label="Teilnehmerliste als PDF"
            on:dblclick=|ev| ev.stop_propagation()>
            <Icon icon=icondata::LuFileText/>
        </a>
    }
}
