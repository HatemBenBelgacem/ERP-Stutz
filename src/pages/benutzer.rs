use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::auth::{BenutzerLoeschen, BenutzerSpeichern, benutzer_laden};
use crate::components::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen, bestaetigt,
    ergebnis_beobachten, meldung_aus,
};
use crate::models::{PASSWORT_MIN_LAENGE, ROLLE_ADMIN, ROLLE_BENUTZER};

#[component]
pub fn BenutzerSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let speichern = ServerAction::<BenutzerSpeichern>::new();
    let loeschen = ServerAction::<BenutzerLoeschen>::new();
    let benutzer = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| benutzer_laden(),
    );

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let rolle = RwSignal::new(ROLLE_BENUTZER.to_string());
    let aktiv = RwSignal::new(true);
    let passwort = RwSignal::new(String::new());
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        name.set(String::new());
        email.set(String::new());
        rolle.set(ROLLE_BENUTZER.to_string());
        aktiv.set(true);
        passwort.set(String::new());
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    ergebnis_beobachten(speichern.value(), formular_meldung, move || dialog_offen.set(false));
    ergebnis_beobachten(loeschen.value(), listen_meldung, || {});

    let absenden = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        speichern.dispatch(BenutzerSpeichern {
            id: bearbeite.get_untracked(),
            name: name.get_untracked(),
            email: email.get_untracked(),
            rolle: rolle.get_untracked(),
            aktiv: aktiv.get_untracked(),
            passwort: passwort.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Benutzer bearbeiten" } else { "Neuer Benutzer" }.to_string()
    });
    let eigene_id = move || sitzung.benutzer().map(|b| b.id);

    view! {
        <Seitenkopf titel="Benutzer" untertitel="Konten, Rollen und Zugänge verwalten">
            <button class="btn btn-primary" on:click=move |_| neu()>
                <Icon icon=icondata::LuUserPlus/>
                "Neuer Benutzer"
            </button>
        </Seitenkopf>

        <Fehlermeldung meldung=listen_meldung/>

        <div class="card border border-base-300 bg-base-100">
            <div class="overflow-x-auto">
                <Transition fallback=Laden>
                    {move || Suspend::new(async move {
                        match benutzer.await {
                            Ok(liste) if liste.is_empty() => view! {
                                <Leer symbol=icondata::LuUserCog text="Noch keine Benutzer."/>
                            }.into_any(),
                            Ok(liste) => view! {
                                <table class="table">
                                    <thead><tr><th>"Name"</th><th>"E-Mail"</th><th>"Rolle"</th><th>"Status"</th><th>"Erstellt"</th><th></th></tr></thead>
                                    <tbody>
                                        {liste.into_iter().map(|b| {
                                            let id = b.id;
                                            let zum_bearbeiten = b.clone();
                                            let ist_admin = b.ist_admin();
                                            view! {
                                                <tr class="hover:bg-base-200/50">
                                                    <td class="font-medium">
                                                        {b.name}
                                                        <Show when=move || eigene_id() == Some(id)>
                                                            <span class="badge badge-sm badge-ghost ml-2">"Du"</span>
                                                        </Show>
                                                    </td>
                                                    <td class="text-base-content/70">{b.email}</td>
                                                    <td>
                                                        {if ist_admin {
                                                            view! {
                                                                <span class="badge badge-primary badge-soft gap-1">
                                                                    <Icon icon=icondata::LuShieldCheck width="0.85rem" height="0.85rem"/>
                                                                    "Admin"
                                                                </span>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-ghost">"Benutzer"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td>
                                                        {if b.aktiv {
                                                            view! { <span class="badge badge-success badge-soft">"Aktiv"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-error badge-soft">"Deaktiviert"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td class="whitespace-nowrap">{b.erstellt_am.format("%d.%m.%Y").to_string()}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=move || {
                                                                let b = zum_bearbeiten.clone();
                                                                bearbeite.set(Some(b.id));
                                                                name.set(b.name);
                                                                email.set(b.email);
                                                                rolle.set(b.rolle);
                                                                aktiv.set(b.aktiv);
                                                                passwort.set(String::new());
                                                                formular_meldung.set(None);
                                                                dialog_offen.set(true);
                                                            }
                                                            loeschen=move || {
                                                                if bestaetigt("Benutzer wirklich löschen?") {
                                                                    loeschen.dispatch(BenutzerLoeschen { id });
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
                    <legend class="fieldset-legend">"Name"</legend>
                    <input class="input w-full" required bind:value=name/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"E-Mail"</legend>
                    <input class="input w-full" type="email" autocomplete="off" required bind:value=email/>
                </fieldset>
                <div class="grid gap-x-4 sm:grid-cols-2">
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Rolle"</legend>
                        <select class="select w-full" on:change:target=move |ev| rolle.set(ev.target().value())>
                            <option value=ROLLE_BENUTZER selected=move || rolle.get() == ROLLE_BENUTZER>"Benutzer"</option>
                            <option value=ROLLE_ADMIN selected=move || rolle.get() == ROLLE_ADMIN>"Admin"</option>
                        </select>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Status"</legend>
                        <label class="label h-10 cursor-pointer gap-3">
                            <input type="checkbox" class="toggle toggle-success" bind:checked=aktiv/>
                            {move || if aktiv.get() { "Aktiv" } else { "Deaktiviert" }}
                        </label>
                    </fieldset>
                </div>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">
                        {move || if bearbeite.get().is_some() { "Neues Passwort" } else { "Passwort" }}
                    </legend>
                    <input class="input w-full" type="password" autocomplete="new-password"
                        required=move || bearbeite.get().is_none()
                        placeholder=move || if bearbeite.get().is_some() { "Leer lassen = unverändert" } else { "" }
                        bind:value=passwort/>
                    <p class="label">{format!("Mindestens {PASSWORT_MIN_LAENGE} Zeichen.")}</p>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}
