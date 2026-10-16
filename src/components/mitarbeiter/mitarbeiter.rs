use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::backend::api::{MitarbeiterLoeschen, MitarbeiterSpeichern, mitarbeiter_laden};
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen,
    bestaetigt, ergebnis_beobachten, meldung_aus,
};

#[component]
pub fn MitarbeiterSeite() -> impl IntoView {
    let speichern = ServerAction::<MitarbeiterSpeichern>::new();
    let loeschen = ServerAction::<MitarbeiterLoeschen>::new();
    let mitarbeiter = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| mitarbeiter_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let vorname = RwSignal::new(String::new());
    let nachname = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let abteilung = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        vorname.set(String::new());
        nachname.set(String::new());
        email.set(String::new());
        abteilung.set(String::new());
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
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Mitarbeiter bearbeiten" } else { "Neuer Mitarbeiter" }.to_string()
    });

    view! {
        <Seitenkopf titel="Mitarbeiter" untertitel="Personen, die an Weiterbildungen teilnehmen">
            <button class="btn btn-primary" on:click=move |_| neu()>
                <Icon icon=icondata::LuPlus/>
                "Neuer Mitarbeiter"
            </button>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match mitarbeiter.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuUsers text="Noch keine Mitarbeiter erfasst."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table max-sm:table-sm">
                                    <thead><tr><th>"Name"</th><th class="hidden md:table-cell">"E-Mail"</th><th class="hidden sm:table-cell">"Abteilung"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|m| {
                                            let id = m.id;
                                            let name = m.name();
                                            let zum_bearbeiten = m.clone();
                                            let bearbeiten = move || {
                                                let m = zum_bearbeiten.clone();
                                                bearbeite.set(Some(m.id));
                                                vorname.set(m.vorname);
                                                nachname.set(m.nachname);
                                                email.set(m.email);
                                                abteilung.set(m.abteilung);
                                                formular_meldung.set(None);
                                                dialog_offen.set(true);
                                            };
                                            view! {
                                                <tr class="cursor-pointer select-none hover:bg-base-200/50" title="Doppelklick zum Bearbeiten"
                                                    on:dblclick={let bearbeiten = bearbeiten.clone(); move |_| bearbeiten()}>
                                                    <td>
                                                        <div class="font-medium">{name}</div>
                                                        <div class="text-xs text-base-content/60 break-all md:hidden">{m.email.clone()}</div>
                                                    </td>
                                                    <td class="hidden text-base-content/70 md:table-cell">{m.email}</td>
                                                    <td class="hidden sm:table-cell">{m.abteilung}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=bearbeiten.clone()
                                                            loeschen=move || {
                                                                if bestaetigt("Mitarbeiter wirklich löschen? Die Teilnahmen an Terminen werden ebenfalls entfernt.") {
                                                                    loeschen.dispatch(MitarbeiterLoeschen { id });
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
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
