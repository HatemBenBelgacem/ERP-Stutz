use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::api::{TeilnehmendeAnmelden, aktive_termine_laden, mitarbeiter_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ergebnis_beobachten, meldung_aus,
};
use crate::components::termin::{ANZEIGE_DATUM, ANZEIGE_ZEIT, AdresseZeilen, MitarbeiterAuswahl};
use crate::models::{Aktion, Bereich, Termin};

/// Aktive Termine als Kacheln. Hier lassen sich nur Mitarbeitende anmelden;
/// alles andere am Termin bleibt unverändert.
#[component]
pub fn TerminAnmeldungSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anmelden = sitzung.recht(Bereich::Anmeldung, Aktion::TeilnehmendeHinzufuegen);
    let darf_termine_anmelden = sitzung.recht(Bereich::Termine, Aktion::TeilnehmendeHinzufuegen);
    let darf_termine_bearbeiten = sitzung.recht(Bereich::Termine, Aktion::Bearbeiten);
    let kann_anmelden =
        move || darf_anmelden.get() || darf_termine_anmelden.get() || darf_termine_bearbeiten.get();

    let anmelden = ServerAction::<TeilnehmendeAnmelden>::new();
    let termine = Resource::new(move || anmelden.version().get(), |_| aktive_termine_laden());

    // Termin, für den der Anmelde-Dialog offen ist
    let dialog_offen = RwSignal::new(false);
    let termin = RwSignal::new(None::<Termin>);
    let oeffnen = move |t: Termin| {
        termin.set(Some(t));
        dialog_offen.set(true);
    };

    view! {
        <Seitenkopf titel="Anmeldung" untertitel="Aktive Termine – Mitarbeitende anmelden">
            {()}
        </Seitenkopf>

        <Transition fallback=Laden>
            {move || Suspend::new(async move {
                match termine.await {
                    Ok(liste) if liste.is_empty() => view! {
                        <div class="card border border-base-300 bg-base-100">
                            <Leer symbol=icondata::LuCalendarCheck text="Zurzeit gibt es keine aktiven Termine."/>
                        </div>
                    }.into_any(),
                    Ok(liste) => view! {
                        <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
                            {liste.into_iter().map(|t| view! {
                                <TerminKachel termin=t kann_anmelden=Signal::derive(kann_anmelden) oeffnen/>
                            }).collect_view()}
                        </div>
                    }.into_any(),
                    Err(e) => view! { <div class="alert alert-error">{meldung_aus(e)}</div> }.into_any(),
                }
            })}
        </Transition>

        <AnmeldeDialog offen=dialog_offen termin anmelden/>
    }
}

/// Eine Kachel pro Termin: Kurs, Datum, Ort, Status und Belegung.
#[component]
fn TerminKachel(
    termin: Termin,
    kann_anmelden: Signal<bool>,
    oeffnen: impl Fn(Termin) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let belegt = termin.mitarbeiter.len();
    let max = termin.max_teilnehmer.max(0) as usize;
    let frei = max.saturating_sub(belegt);
    let voll = frei == 0;
    // Farbe der Belegung: grün, ab 80 % gelb, voll rot
    let farbe = if voll {
        "progress-error"
    } else if belegt * 5 >= max * 4 {
        "progress-warning"
    } else {
        "progress-success"
    };
    let datum = if termin.beginn.date() == termin.ende.date() {
        termin.beginn.format(ANZEIGE_DATUM).to_string()
    } else {
        format!("{} – {}", termin.beginn.format(ANZEIGE_DATUM), termin.ende.format(ANZEIGE_DATUM))
    };
    let zeit = format!("{} – {} Uhr", termin.beginn.format(ANZEIGE_ZEIT), termin.ende.format(ANZEIGE_ZEIT));
    let status = termin.status;
    let adresse = termin.adresse.clone();
    let titel = termin.kurs_titel.clone();

    view! {
        <div class="card border border-base-300 bg-base-100">
            <div class="card-body gap-3 p-5">
                <div class="flex items-start justify-between gap-2">
                    <h2 class="card-title text-base leading-snug">{titel}</h2>
                    <span class=format!("badge badge-sm shrink-0 whitespace-nowrap {}", status.badge_klasse())>
                        {status.titel()}
                    </span>
                </div>

                <div class="flex gap-2 text-sm">
                    <Icon icon=icondata::LuCalendar attr:class="mt-0.5 shrink-0 opacity-60"/>
                    <div>
                        <div>{datum}</div>
                        <div class="text-base-content/60">{zeit}</div>
                    </div>
                </div>
                <div class="flex gap-2 text-sm">
                    <Icon icon=icondata::LuMapPin attr:class="mt-0.5 shrink-0 opacity-60"/>
                    {if adresse.is_some() {
                        view! { <div><AdresseZeilen adresse/></div> }.into_any()
                    } else {
                        view! { <div class="text-base-content/50">"Kein Veranstaltungsort"</div> }.into_any()
                    }}
                </div>

                <div class="mt-auto">
                    <div class="mb-1 flex items-baseline justify-between text-sm">
                        <span class="font-medium">{format!("{belegt} von {max} angemeldet")}</span>
                        <span class="text-base-content/60">
                            {if voll { "Ausgebucht".to_string() } else { format!("{frei} frei") }}
                        </span>
                    </div>
                    <progress class=format!("progress w-full {farbe}") value=belegt max=max></progress>
                </div>

                <Show when=move || kann_anmelden.get()>
                    <button class="btn btn-primary btn-sm"
                        disabled=voll
                        title=if voll { "Dieser Termin ist ausgebucht" } else { "Mitarbeitende anmelden" }
                        on:click={
                            let termin = termin.clone();
                            move |_| oeffnen(termin.clone())
                        }>
                        <Icon icon=icondata::LuUserPlus/>
                        "Mitarbeitende anmelden"
                    </button>
                </Show>
            </div>
        </div>
    }
}

/// Auswahl der Mitarbeitenden für einen Termin. Bisherige Teilnehmende
/// sind gesetzt und lassen sich hier nicht entfernen.
#[component]
fn AnmeldeDialog(
    offen: RwSignal<bool>,
    termin: RwSignal<Option<Termin>>,
    anmelden: ServerAction<TeilnehmendeAnmelden>,
) -> impl IntoView {
    let mitarbeiter = Resource::new(|| (), |_| mitarbeiter_laden());
    let ausgewaehlt = RwSignal::new(Vec::<Uuid>::new());
    let bestehend = RwSignal::new(Vec::<Uuid>::new());
    let suche = RwSignal::new(String::new());
    let meldung = RwSignal::new(None::<String>);

    // Beim Öffnen die bisherigen Teilnehmenden übernehmen
    Effect::new(move |_| {
        if let Some(t) = termin.get() {
            let ids: Vec<Uuid> = t.mitarbeiter.iter().map(|m| m.id).collect();
            bestehend.set(ids.clone());
            ausgewaehlt.set(ids);
            suche.set(String::new());
            meldung.set(None);
        }
    });
    ergebnis_beobachten(anmelden.value(), meldung, move || offen.set(false));

    let max_plaetze = Signal::derive(move || termin.with(|t| t.as_ref().map(|t| t.max_teilnehmer.max(0) as usize)));
    let gesperrt = Signal::derive(move || bestehend.get());
    let titel = Signal::derive(move || {
        termin.with(|t| {
            t.as_ref()
                .map(|t| format!("{} – {}", t.kurs_titel, t.beginn.format(ANZEIGE_DATUM)))
                .unwrap_or_default()
        })
    });

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(t) = termin.get_untracked() else { return };
        let bisher = bestehend.get_untracked();
        let neu: Vec<Uuid> = ausgewaehlt.get_untracked().into_iter().filter(|id| !bisher.contains(id)).collect();
        if neu.is_empty() {
            meldung.set(Some("Bitte mindestens eine Person auswählen.".into()));
            return;
        }
        anmelden.dispatch(TeilnehmendeAnmelden { termin_id: t.id, mitarbeiter_ids: neu });
    };

    view! {
        <Dialog offen titel>
            <form on:submit=absenden>
                <Fehlermeldung meldung/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">
                        "Teilnehmende"
                        <span class="badge badge-sm badge-ghost">
                            {move || format!("{} / {}", ausgewaehlt.with(Vec::len), max_plaetze.get().unwrap_or_default())}
                        </span>
                    </legend>
                    <Suspense fallback=Laden>
                        {move || Suspend::new(async move {
                            let liste = mitarbeiter.await.unwrap_or_default();
                            view! { <MitarbeiterAuswahl liste ausgewaehlt suche max_plaetze gesperrt/> }
                        })}
                    </Suspense>
                </fieldset>
                <DialogAktionen offen beschaeftigt=anmelden.pending()/>
            </form>
        </Dialog>
    }
}
