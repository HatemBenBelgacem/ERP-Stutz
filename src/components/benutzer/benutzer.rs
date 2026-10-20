use leptos::prelude::*;
use leptos_icons::Icon;
use uuid::Uuid;

use crate::app::use_sitzung;
use crate::backend::auth::{BenutzerLoeschen, BenutzerSpeichern, benutzer_laden};
use crate::backend::rollen::rollen_laden;
use crate::components::gemeinsam::{
    Dialog, DialogAktionen, Fehlermeldung, Laden, Leer, Seitenkopf, ZeilenAktionen, bestaetigt,
    ergebnis_beobachten, meldung_aus,
};
use crate::models::{Aktion, Benutzer, Bereich, PASSWORT_MIN_LAENGE};

#[component]
pub fn BenutzerSeite() -> impl IntoView {
    let sitzung = use_sitzung();
    let darf_anlegen = sitzung.recht(Bereich::Benutzer, Aktion::Anlegen);
    let darf_bearbeiten = sitzung.recht(Bereich::Benutzer, Aktion::Bearbeiten);
    let darf_loeschen = sitzung.recht(Bereich::Benutzer, Aktion::Loeschen);

    let speichern = ServerAction::<BenutzerSpeichern>::new();
    let loeschen = ServerAction::<BenutzerLoeschen>::new();
    let benutzer = Resource::new(
        move || (speichern.version().get(), loeschen.version().get()),
        |_| benutzer_laden(),
    );
    let rollen = Resource::new(|| (), |_| rollen_laden());

    let dialog_offen = RwSignal::new(false);
    let bearbeite = RwSignal::new(None::<Uuid>);
    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    // Leerer Text = keine Rolle
    let rolle_id = RwSignal::new(String::new());
    let aktiv = RwSignal::new(true);
    let passwort = RwSignal::new(String::new());
    let microsoft = RwSignal::new(false);
    let formular_meldung = RwSignal::new(None::<String>);
    let listen_meldung = RwSignal::new(None::<String>);

    let neu = move || {
        bearbeite.set(None);
        name.set(String::new());
        email.set(String::new());
        rolle_id.set(String::new());
        aktiv.set(true);
        passwort.set(String::new());
        microsoft.set(false);
        formular_meldung.set(None);
        dialog_offen.set(true);
    };
    let benutzer_bearbeiten = move |b: Benutzer| {
        bearbeite.set(Some(b.id));
        name.set(b.name);
        email.set(b.email);
        rolle_id.set(b.rolle_id.map(|r| r.to_string()).unwrap_or_default());
        aktiv.set(b.aktiv);
        passwort.set(String::new());
        microsoft.set(b.microsoft);
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
            rolle_id: rolle_id.get_untracked().parse().ok(),
            aktiv: aktiv.get_untracked(),
            passwort: passwort.get_untracked(),
        });
    };

    let dialog_titel = Signal::derive(move || {
        if bearbeite.get().is_some() { "Benutzer bearbeiten" } else { "Neuer Benutzer" }.to_string()
    });
    let eigene_id = move || sitzung.benutzer().map(|b| b.id);

    view! {
        <Seitenkopf titel="Benutzer" untertitel="Konten, ihre Rolle und ihr Status">
            <Show when=move || darf_anlegen.get()>
                <button class="btn btn-primary" on:click=move |_| neu()>
                    <Icon icon=icondata::LuUserPlus/>
                    "Neuer Benutzer"
                </button>
            </Show>
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
                                <table class="table table-zebra max-sm:table-sm">
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th class="hidden md:table-cell">"E-Mail"</th>
                                            <th>"Rolle"</th>
                                            <th class="hidden sm:table-cell">"Status"</th>
                                            <th class="hidden lg:table-cell">"Erstellt"</th>
                                            <th></th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {liste.into_iter().map(|b| {
                                            let id = b.id;
                                            let fuer_doppelklick = b.clone();
                                            let fuer_button = b.clone();
                                            view! {
                                                <tr class="select-none hover:bg-base-300/60"
                                                    class:cursor-pointer=move || darf_bearbeiten.get()
                                                    title=move || darf_bearbeiten.get().then_some("Doppelklick zum Bearbeiten")
                                                    on:dblclick=move |_| if darf_bearbeiten.get_untracked() {
                                                        benutzer_bearbeiten(fuer_doppelklick.clone())
                                                    }>
                                                    <td>
                                                        <div class="flex flex-wrap items-center gap-x-2 gap-y-1 font-medium">
                                                            {b.name}
                                                            <Show when=move || eigene_id() == Some(id)>
                                                                <span class="badge badge-sm badge-ghost">"Du"</span>
                                                            </Show>
                                                            {b.microsoft.then(|| view! {
                                                                <span class="badge badge-sm badge-info badge-soft" title="Meldet sich mit Microsoft an">
                                                                    "Microsoft"
                                                                </span>
                                                            })}
                                                        </div>
                                                        <div class="text-xs text-base-content/60 break-all md:hidden">{b.email.clone()}</div>
                                                        {(!b.aktiv).then(|| view! {
                                                            <span class="badge badge-xs badge-error badge-soft sm:hidden">"Deaktiviert"</span>
                                                        })}
                                                    </td>
                                                    <td class="hidden text-base-content/70 md:table-cell">{b.email}</td>
                                                    <td><RollenBadge ist_admin=b.ist_admin name=b.rolle_name/></td>
                                                    <td class="hidden sm:table-cell">
                                                        {if b.aktiv {
                                                            view! { <span class="badge badge-success badge-soft">"Aktiv"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="badge badge-error badge-soft">"Deaktiviert"</span> }.into_any()
                                                        }}
                                                    </td>
                                                    <td class="hidden whitespace-nowrap lg:table-cell">{b.erstellt_am.format("%d.%m.%Y").to_string()}</td>
                                                    <td>
                                                        <ZeilenAktionen
                                                            bearbeiten=move || benutzer_bearbeiten(fuer_button.clone())
                                                            loeschen=move || {
                                                                if bestaetigt("Benutzer wirklich löschen?") {
                                                                    loeschen.dispatch(BenutzerLoeschen { id });
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
                        <select class="select w-full" on:change:target=move |ev| rolle_id.set(ev.target().value())>
                            <option value="" selected=move || rolle_id.get().is_empty()>"Keine Rolle – kein Zugriff"</option>
                            <Suspense>
                                {move || Suspend::new(async move {
                                    rollen.await.unwrap_or_default().into_iter().map(|r| {
                                        let wert = r.id.to_string();
                                        let vergleich = wert.clone();
                                        view! {
                                            <option value=wert selected=move || rolle_id.get() == vergleich>{r.name}</option>
                                        }
                                    }).collect_view()
                                })}
                            </Suspense>
                        </select>
                        <p class="label">"Was die Rolle erlaubt, legst du unter „Rollen“ fest."</p>
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
                        placeholder=move || if bearbeite.get().is_some() { "Leer lassen = unverändert" } else { "Optional" }
                        bind:value=passwort/>
                    <p class="label">
                        {move || if microsoft.get() {
                            "Dieses Konto meldet sich mit Microsoft an; ein Passwort ist zusätzlich möglich.".to_string()
                        } else {
                            format!("Mindestens {PASSWORT_MIN_LAENGE} Zeichen. Ohne Passwort ist nur die Anmeldung mit Microsoft möglich.")
                        }}
                    </p>
                </fieldset>
                <DialogAktionen offen=dialog_offen beschaeftigt=speichern.pending()/>
            </form>
        </Dialog>
    }
}

/// Rollenname als Badge; Admin-Rollen mit Schild-Icon.
#[component]
pub fn RollenBadge(ist_admin: bool, name: Option<String>) -> impl IntoView {
    match name {
        Some(name) if ist_admin => view! {
            <span class="badge badge-primary badge-soft gap-1 whitespace-nowrap">
                <Icon icon=icondata::LuShieldCheck width="0.85rem" height="0.85rem"/>
                {name}
            </span>
        }
        .into_any(),
        Some(name) => view! { <span class="badge badge-ghost whitespace-nowrap">{name}</span> }.into_any(),
        None => view! { <span class="badge badge-warning badge-soft whitespace-nowrap">"Keine Rolle"</span> }.into_any(),
    }
}
