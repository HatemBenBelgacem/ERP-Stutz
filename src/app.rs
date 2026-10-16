use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::StaticSegment;
use leptos_router::components::{ProtectedParentRoute, ProtectedRoute, Route, Router, Routes};

use crate::backend::auth::{Abmelden, Anmelden, Registrieren, aktueller_benutzer_laden};
use crate::components::anmeldung::{AnmeldeSeite, RegistrierSeite};
use crate::components::benutzer::BenutzerSeite;
use crate::components::gemeinsam::Leer;
use crate::components::kurs::KursSeite;
use crate::components::layout::Layout;
use crate::components::mitarbeiter::MitarbeiterSeite;
use crate::components::termin::TerminSeite;
use crate::components::uebersicht::Uebersicht;
use crate::components::kein_zugriff::KeinZugriff;
use crate::models::{Benutzer, Bereich};

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

    pub fn ist_admin(&self) -> Option<bool> {
        self.benutzer.get().map(|r| matches!(r, Ok(Some(b)) if b.ist_admin()))
    }

    /// `None` = lädt noch, sonst ob der Bereich freigegeben ist.
    pub fn darf(&self, bereich: Bereich) -> Option<bool> {
        self.benutzer.get().map(|r| matches!(r, Ok(Some(b)) if b.darf(bereich)))
    }

    /// Wohin umgeleitet wird, wenn eine Seite nicht erlaubt ist.
    pub fn startseite(&self) -> String {
        self.benutzer()
            .map(|b| b.startseite().to_string())
            .unwrap_or_else(|| "/anmelden".to_string())
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
        <Stylesheet id="leptos" href="/pkg/weiterbildung.css"/>
        <Title text="Weiterbildung"/>
        <Router>
            <Routes fallback=|| view! { <Leer symbol=icondata::LuGraduationCap text="Seite nicht gefunden."/> }>
                <Route path=StaticSegment("anmelden") view=AnmeldeSeite/>
                <Route path=StaticSegment("registrieren") view=RegistrierSeite/>
                // Alles Weitere nur mit Anmeldung
                <ProtectedParentRoute
                    path=StaticSegment("")
                    view=Layout
                    condition=move || sitzung.angemeldet()
                    redirect_path=|| "/anmelden"
                >
                    // Jeder Bereich nur mit Berechtigung, sonst zur ersten erlaubten Seite
                    <ProtectedRoute
                        path=StaticSegment("")
                        view=Uebersicht
                        condition=move || sitzung.darf(Bereich::Uebersicht)
                        redirect_path=move || sitzung.startseite()
                    />
                    <ProtectedRoute
                        path=StaticSegment("kurse")
                        view=KursSeite
                        condition=move || sitzung.darf(Bereich::Kurse)
                        redirect_path=move || sitzung.startseite()
                    />
                    <ProtectedRoute
                        path=StaticSegment("mitarbeiter")
                        view=MitarbeiterSeite
                        condition=move || sitzung.darf(Bereich::Mitarbeiter)
                        redirect_path=move || sitzung.startseite()
                    />
                    <ProtectedRoute
                        path=StaticSegment("termine")
                        view=TerminSeite
                        condition=move || sitzung.darf(Bereich::Termine)
                        redirect_path=move || sitzung.startseite()
                    />
                    <Route path=StaticSegment("kein-zugriff") view=KeinZugriff/>
                    <ProtectedRoute
                        path=StaticSegment("benutzer")
                        view=BenutzerSeite
                        condition=move || sitzung.ist_admin()
                        redirect_path=move || sitzung.startseite()
                    />
                </ProtectedParentRoute>
            </Routes>
        </Router>
    }
}
