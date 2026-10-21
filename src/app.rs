use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::StaticSegment;
use leptos_router::components::{Outlet, ParentRoute, ProtectedParentRoute, Redirect, Route, Router, Routes};

use crate::backend::auth::{Abmelden, Anmelden, Registrieren, aktueller_benutzer_laden};
use crate::components::adresse::AdresseSeite;
use crate::components::anmeldung::{AnmeldeSeite, RegistrierSeite};
use crate::components::benutzer::BenutzerSeite;
use crate::components::einstellungen::{EinstellungenSeite, EinstellungenStart};
use crate::components::gemeinsam::Leer;
use crate::components::geschaeftsbereich::GeschaeftsbereichSeite;
use crate::components::kein_zugriff::KeinZugriff;
use crate::components::kurs::KursSeite;
use crate::components::layout::Layout;
use crate::components::mitarbeiter::MitarbeiterSeite;
use crate::components::modul::ModulSeite;
use crate::components::rolle::RollenSeite;
use crate::components::termin::TerminSeite;
use crate::components::weiterbildung::{WeiterbildungSeite, WeiterbildungStart};
use crate::models::{Aktion, Benutzer, Bereich};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-base-200">
                <App/>
            </body>
        </html>
    }
}

/// Anmeldezustand, für alle Komponenten per Kontext verfügbar.
#[derive(Clone, Copy)]
pub struct Sitzung {
    pub benutzer: Resource<Result<Option<Benutzer>, ServerFnError>>,
    pub anmelden: ServerAction<Anmelden>,
    pub registrieren: ServerAction<Registrieren>,
    pub abmelden: ServerAction<Abmelden>,
}

impl Sitzung {
    /// Angemeldeter Benutzer; `None`, solange geladen wird oder niemand angemeldet ist.
    pub fn benutzer(&self) -> Option<Benutzer> {
        self.benutzer.get().and_then(Result::ok).flatten()
    }

    /// `None` = lädt noch, sonst ob jemand angemeldet ist.
    pub fn angemeldet(&self) -> Option<bool> {
        self.benutzer.get().map(|r| matches!(r, Ok(Some(_))))
    }

    /// `None` = lädt noch, sonst ob der Benutzer die Funktion nutzen darf.
    pub fn darf(&self, bereich: Bereich, aktion: Aktion) -> Option<bool> {
        self.benutzer.get().map(|r| matches!(r, Ok(Some(b)) if b.darf(bereich, aktion)))
    }

    pub fn sieht(&self, bereich: Bereich) -> Option<bool> {
        self.darf(bereich, Aktion::Sehen)
    }

    /// Für Buttons: ein Recht, das reaktiv abgefragt werden kann.
    pub fn recht(self, bereich: Bereich, aktion: Aktion) -> Recht {
        Recht { sitzung: self, bereich, aktion }
    }

    /// Wohin umgeleitet wird, wenn eine Seite nicht erlaubt ist.
    pub fn startseite(&self) -> String {
        self.benutzer()
            .map(|b| b.startseite().to_string())
            .unwrap_or_else(|| "/anmelden".to_string())
    }
}

/// Ein Recht des angemeldeten Benutzers. Liest direkt den Anmeldezustand der
/// App, damit es auch beim Abbau einer Seite (z. B. nach dem Abmelden) gültig bleibt.
#[derive(Clone, Copy)]
pub struct Recht {
    sitzung: Sitzung,
    bereich: Bereich,
    aktion: Aktion,
}

impl Recht {
    /// `true` nur, wenn das Recht sicher vorhanden ist (reaktiv).
    pub fn get(&self) -> bool {
        self.sitzung.darf(self.bereich, self.aktion) == Some(true)
    }

    pub fn get_untracked(&self) -> bool {
        self.sitzung
            .benutzer
            .get_untracked()
            .is_some_and(|r| matches!(r, Ok(Some(b)) if b.darf(self.bereich, self.aktion)))
    }
}

pub fn use_sitzung() -> Sitzung {
    expect_context()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let anmelden = ServerAction::<Anmelden>::new();
    let registrieren = ServerAction::<Registrieren>::new();
    let abmelden = ServerAction::<Abmelden>::new();
    let benutzer = Resource::new(
        move || (anmelden.version().get(), registrieren.version().get(), abmelden.version().get()),
        |_| aktueller_benutzer_laden(),
    );
    let sitzung = Sitzung { benutzer, anmelden, registrieren, abmelden };
    provide_context(sitzung);

    view! {
        <Stylesheet id="leptos" href="/pkg/erp-stutz.css"/>
        <Title text="ERP-Stutz AG"/>
        <Router>
            <Routes fallback=|| view! { <Leer symbol=icondata::LuHardHat text="Seite nicht gefunden."/> }>
                <Route path=StaticSegment("anmelden") view=AnmeldeSeite/>
                <Route path=StaticSegment("registrieren") view=RegistrierSeite/>
                // Alles Weitere nur mit Anmeldung
                <ProtectedParentRoute
                    path=StaticSegment("")
                    view=Layout
                    condition=move || sitzung.angemeldet()
                    redirect_path=|| "/anmelden"
                >
                    // Jeder Bereich nur mit dem Recht „Sehen“ (siehe Bereichsschutz)
                    // Startseite: erster Bereich, den der Benutzer sehen darf
                    <Route path=StaticSegment("") view=Startseite/>
                    <ParentRoute path=StaticSegment("personal") view=Outlet>
                        <Route path=StaticSegment("") view=Startseite/>
                        <ParentRoute path=StaticSegment("weiterbildung") view=WeiterbildungSeite>
                            <Route path=StaticSegment("") view=WeiterbildungStart/>
                            <Route
                                path=StaticSegment("termine")
                                view=|| view! { <Bereichsschutz bereich=Bereich::Termine><TerminSeite/></Bereichsschutz> }
                            />
                            <Route
                                path=StaticSegment("kurse")
                                view=|| view! { <Bereichsschutz bereich=Bereich::Kurse><KursSeite/></Bereichsschutz> }
                            />
                        </ParentRoute>
                        <Route
                            path=StaticSegment("mitarbeiter")
                            view=|| view! { <Bereichsschutz bereich=Bereich::Mitarbeiter><MitarbeiterSeite/></Bereichsschutz> }
                        />
                    </ParentRoute>
                    <Route
                        path=StaticSegment("adressen")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Adressen><AdresseSeite/></Bereichsschutz> }
                    />
                    <Route
                        path=StaticSegment("einkauf")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Einkauf><ModulSeite bereich=Bereich::Einkauf/></Bereichsschutz> }
                    />
                    <Route
                        path=StaticSegment("lager")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Lager><ModulSeite bereich=Bereich::Lager/></Bereichsschutz> }
                    />
                    <Route
                        path=StaticSegment("projekte")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Projekte><ModulSeite bereich=Bereich::Projekte/></Bereichsschutz> }
                    />
                    <Route
                        path=StaticSegment("werkhof")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Werkhof><ModulSeite bereich=Bereich::Werkhof/></Bereichsschutz> }
                    />
                    <Route
                        path=StaticSegment("finanzen")
                        view=|| view! { <Bereichsschutz bereich=Bereich::Finanzen><ModulSeite bereich=Bereich::Finanzen/></Bereichsschutz> }
                    />
                    <ParentRoute path=StaticSegment("einstellungen") view=EinstellungenSeite>
                        <Route path=StaticSegment("") view=EinstellungenStart/>
                        <Route
                            path=StaticSegment("benutzer")
                            view=|| view! { <Bereichsschutz bereich=Bereich::Benutzer><BenutzerSeite/></Bereichsschutz> }
                        />
                        <Route
                            path=StaticSegment("rollen")
                            view=|| view! { <Bereichsschutz bereich=Bereich::Rollen><RollenSeite/></Bereichsschutz> }
                        />
                        <Route
                            path=StaticSegment("geschaeftsbereiche")
                            view=|| view! { <Bereichsschutz bereich=Bereich::Geschaeftsbereiche><GeschaeftsbereichSeite/></Bereichsschutz> }
                        />
                    </ParentRoute>
                    <Route path=StaticSegment("kein-zugriff") view=KeinZugriff/>
                </ProtectedParentRoute>
            </Routes>
        </Router>
    }
}

/// Leitet zum ersten Bereich weiter, den der Benutzer sehen darf.
#[component]
fn Startseite() -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <Transition>
            {move || sitzung.benutzer().map(|b| view! { <Redirect path=b.startseite()/> })}
        </Transition>
    }
}

/// Zeigt die Seite nur mit dem Recht „Sehen“ für den Bereich, sonst geht es
/// zur ersten erlaubten Seite. Ist niemand angemeldet, übernimmt die äussere
/// Route die Weiterleitung zur Anmeldung – so leitet nach dem Abmelden nur
/// eine Stelle weiter.
#[component]
fn Bereichsschutz(bereich: Bereich, children: ChildrenFn) -> impl IntoView {
    let sitzung = use_sitzung();
    let zugang = Memo::new(move |_| sitzung.benutzer().map(|b| (b.sieht(bereich), b.startseite())));
    view! {
        <Transition>
            {move || match zugang.get() {
                Some((true, _)) => children().into_any(),
                Some((false, startseite)) => view! { <Redirect path=startseite/> }.into_any(),
                None => ().into_any(),
            }}
        </Transition>
    }
}
