use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::StaticSegment;
use leptos_router::components::{ProtectedParentRoute, ProtectedRoute, Route, Router, Routes};

use crate::auth::{Abmelden, Anmelden, Registrieren, aktueller_benutzer_laden};
use crate::components::Leer;
use crate::layout::Layout;
use crate::models::Benutzer;
use crate::pages::anmelden::{AnmeldeSeite, RegistrierSeite};
use crate::pages::benutzer::BenutzerSeite;
use crate::pages::kurse::KursSeite;
use crate::pages::mitarbeiter::MitarbeiterSeite;
use crate::pages::termine::TerminSeite;
use crate::pages::uebersicht::Uebersicht;

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
                    <Route path=StaticSegment("") view=Uebersicht/>
                    <Route path=StaticSegment("kurse") view=KursSeite/>
                    <Route path=StaticSegment("mitarbeiter") view=MitarbeiterSeite/>
                    <Route path=StaticSegment("termine") view=TerminSeite/>
                    <ProtectedRoute
                        path=StaticSegment("benutzer")
                        view=BenutzerSeite
                        condition=move || sitzung.ist_admin()
                        redirect_path=|| "/"
                    />
                </ProtectedParentRoute>
            </Routes>
        </Router>
    }
}
