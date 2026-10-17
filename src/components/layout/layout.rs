//! Grundgerüst der App: Header oben, einklappbare Sidebar links, Inhalt rechts.

use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_location;

use crate::app::use_sitzung;
use crate::models::Bereich;

/// Ein Menüpunkt; sichtbar, sobald einer seiner Bereiche sichtbar ist.
#[derive(Clone, Copy)]
struct Ziel {
    text: &'static str,
    href: &'static str,
    icon: icondata::Icon,
    bereiche: &'static [Bereich],
}

/// Navigation in Gruppen; Gruppen mit Titel lassen sich einklappen,
/// Gruppen ohne Titel stehen für sich.
const NAVIGATION: [(Option<(&str, icondata::Icon)>, &[Ziel]); 2] = [
    (
        Some(("Personal", icondata::LuBriefcase)),
        &[
            // Termine und Kurse als Reiter
            Ziel { text: "Weiterbildung", href: "/personal/weiterbildung", icon: icondata::LuGraduationCap, bereiche: &Bereich::WEITERBILDUNG },
            Ziel { text: "Mitarbeiter", href: "/personal/mitarbeiter", icon: icondata::LuUsers, bereiche: &[Bereich::Mitarbeiter] },
        ],
    ),
    (
        None,
        &[
            Ziel { text: "Einkauf", href: "/einkauf", icon: icondata::LuShoppingCart, bereiche: &[Bereich::Einkauf] },
            Ziel { text: "Projekte", href: "/projekte", icon: icondata::LuConstruction, bereiche: &[Bereich::Projekte] },
            Ziel { text: "Finanzen", href: "/finanzen", icon: icondata::LuWallet, bereiche: &[Bereich::Finanzen] },
        ],
    ),
];

#[component]
pub fn Layout() -> impl IntoView {
    // Smartphone (< md): Sidebar als ausklappbares Menü.
    // Tablet (md bis lg): schmale Leiste nur mit Icons.
    // Desktop (ab lg): breit oder per Knopf auf Icons eingeklappt.
    let eingeklappt = RwSignal::new(false);
    let mobil_offen = RwSignal::new(false);

    view! {
        <div class="flex h-screen flex-col">
            <Header eingeklappt mobil_offen/>
            <div class="flex flex-1 overflow-hidden">
                <Sidebar eingeklappt mobil_offen/>
                <main class="min-w-0 flex-1 overflow-y-auto p-3 sm:p-4 lg:p-8">
                    <div class="mx-auto max-w-6xl"><Outlet/></div>
                </main>
            </div>
        </div>
    }
}

#[component]
fn Header(eingeklappt: RwSignal<bool>, mobil_offen: RwSignal<bool>) -> impl IntoView {
    view! {
        // Farben wie stutzag.ch: gelber Balken, schwarze Schrift
        <header class="navbar z-50 h-16 min-h-16 gap-2 border-b border-stutz-dunkel bg-stutz px-3 text-black">
            <button
                class="btn btn-ghost btn-square text-black hover:border-transparent hover:bg-black/10 md:hidden"
                aria-label="Navigation öffnen"
                on:click=move |_| mobil_offen.update(|o| *o = !*o)
            >
                <Icon icon=icondata::LuMenu width="1.25rem" height="1.25rem"/>
            </button>
            <button
                class="btn btn-ghost btn-square hidden text-black hover:border-transparent hover:bg-black/10 lg:inline-flex"
                title=move || if eingeklappt.get() { "Navigation ausklappen" } else { "Navigation einklappen" }
                aria-label="Navigation ein- oder ausklappen"
                on:click=move |_| eingeklappt.update(|e| *e = !*e)
            >
                {move || {
                    let symbol = if eingeklappt.get() { icondata::LuPanelLeftOpen } else { icondata::LuPanelLeftClose };
                    view! { <Icon icon=symbol width="1.25rem" height="1.25rem"/> }
                }}
            </button>
            <A href="/" attr:class="flex items-center gap-2 text-lg font-black tracking-tight">
                <span class="rounded-field bg-black p-1.5 text-stutz">
                    <Icon icon=icondata::LuHardHat width="1.25rem" height="1.25rem"/>
                </span>
                "ERP-Stutz AG"
            </A>
            <div class="flex-1"></div>
            <Benutzermenue/>
        </header>
    }
}

#[component]
fn Sidebar(eingeklappt: RwSignal<bool>, mobil_offen: RwSignal<bool>) -> impl IntoView {
    let klassen = move || {
        let mobil = if mobil_offen.get() { "translate-x-0 shadow-xl" } else { "-translate-x-full" };
        let breite = if eingeklappt.get() { "lg:w-16" } else { "lg:w-60" };
        format!(
            "fixed top-16 bottom-0 left-0 z-40 w-60 {mobil} md:static md:w-16 md:translate-x-0 md:shadow-none {breite} \
             flex shrink-0 flex-col overflow-hidden border-r border-base-300 bg-base-100 transition-all duration-200"
        )
    };
    let sitzung = use_sitzung();

    view! {
        <Show when=move || mobil_offen.get()>
            <div class="fixed inset-0 top-16 z-30 bg-base-content/20 md:hidden" on:click=move |_| mobil_offen.set(false)></div>
        </Show>
        <aside class=klassen aria-label="Hauptnavigation">
            <ul class="menu w-full gap-1 p-2">
                // Nur Bereiche anzeigen, die der Benutzer sehen darf
                <Transition>
                    {NAVIGATION
                        .into_iter()
                        .map(|(gruppe, eintraege)| {
                            let sieht = move |ziel: &Ziel| ziel.bereiche.iter().any(|b| sitzung.sieht(*b) == Some(true));
                            let sichtbar = move || eintraege.iter().any(sieht);
                            let liste = move || {
                                eintraege
                                    .iter()
                                    .map(|&ziel| view! {
                                        <Show when=move || sieht(&ziel)>
                                            <NavEintrag href=ziel.href text=ziel.text icon=ziel.icon exact=false eingeklappt mobil_offen/>
                                        </Show>
                                    })
                                    .collect_view()
                            };
                            view! {
                                <Show when=sichtbar>
                                    <li class="pointer-events-none mx-2 my-2 h-px bg-base-300 p-0 first:hidden" aria-hidden="true"></li>
                                    {match gruppe {
                                        Some((titel, icon)) => view! {
                                            <NavGruppe titel icon eintraege eingeklappt>{liste()}</NavGruppe>
                                        }.into_any(),
                                        None => liste().into_any(),
                                    }}
                                </Show>
                            }
                        })
                        .collect_view()}
                </Transition>
            </ul>
            // Fusszeile: Einstellungen (Benutzer- und Rollenverwaltung)
            <Transition>
                <Show when=move || sitzung.benutzer().is_some_and(|b| b.sieht_einstellungen())>
                    <ul class="menu mt-auto w-full border-t border-base-300 p-2">
                        <NavEintrag href="/einstellungen" text="Einstellungen" icon=icondata::LuSettings
                            exact=false eingeklappt mobil_offen/>
                    </ul>
                </Show>
            </Transition>
        </aside>
    }
}

/// Menütexte: auf dem Tablet immer ausgeblendet, auf dem Desktop nur eingeklappt.
fn beschriftung_klassen(eingeklappt: bool) -> &'static str {
    if eingeklappt { "md:hidden" } else { "md:max-lg:hidden" }
}

/// Einklappbare Gruppe. Eingeklappt ist der Titel markiert, wenn eine
/// ihrer Seiten offen ist.
#[component]
fn NavGruppe(
    titel: &'static str,
    icon: icondata::Icon,
    eintraege: &'static [Ziel],
    eingeklappt: RwSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let offen = RwSignal::new(true);
    let pfad = use_location().pathname;
    let aktiv = move || pfad.with(|p| eintraege.iter().any(|z| p.starts_with(z.href)));
    let beschriftung = move || beschriftung_klassen(eingeklappt.get());
    // Als Icon-Leiste ohne Einzug und Linie
    let einzug = move || {
        if eingeklappt.get() { "md:ms-0 md:ps-0 md:before:hidden" } else { "md:max-lg:ms-0 md:max-lg:ps-0 md:max-lg:before:hidden" }
    };
    view! {
        <li>
            <button
                type="button"
                class="flex items-center gap-3 whitespace-nowrap"
                class=("text-primary", move || !offen.get() && aktiv())
                class=("font-medium", move || !offen.get() && aktiv())
                title=move || format!("{titel} {}", if offen.get() { "einklappen" } else { "ausklappen" })
                aria-expanded=move || offen.get().to_string()
                on:click=move |_| offen.update(|o| *o = !*o)
            >
                <Icon icon width="1.25rem" height="1.25rem" attr:class="shrink-0"/>
                <span class=move || format!("flex-1 text-left {}", beschriftung())>{titel}</span>
                <span class=move || format!("transition-transform {} {}", beschriftung(), if offen.get() { "" } else { "-rotate-90" })>
                    <Icon icon=icondata::LuChevronDown attr:class="opacity-60"/>
                </span>
            </button>
            <Show when=move || offen.get()>
                <ul class=move || format!("mt-1 flex flex-col gap-1 {}", einzug())>{children()}</ul>
            </Show>
        </li>
    }
}

#[component]
fn NavEintrag(
    href: &'static str,
    text: &'static str,
    icon: icondata::Icon,
    eingeklappt: RwSignal<bool>,
    mobil_offen: RwSignal<bool>,
    /// `false`: auch auf Unterseiten als aktiv markieren
    #[prop(default = true)]
    exact: bool,
) -> impl IntoView {
    let beschriftung = move || beschriftung_klassen(eingeklappt.get());
    view! {
        <li on:click=move |_| mobil_offen.set(false)>
            <A
                href=href
                exact=exact
                attr:class="flex items-center gap-3 whitespace-nowrap aria-[current=page]:bg-primary/10 aria-[current=page]:font-medium aria-[current=page]:text-primary"
                attr:title=text
            >
                <Icon icon width="1.25rem" height="1.25rem" attr:class="shrink-0"/>
                <span class=beschriftung>{text}</span>
            </A>
        </li>
    }
}

/// Name des angemeldeten Benutzers mit Dropdown zum Abmelden.
#[component]
fn Benutzermenue() -> impl IntoView {
    let sitzung = use_sitzung();
    view! {
        <Transition>
            {move || sitzung.benutzer().map(|b| {
                let rolle = b.rolle_name.clone().unwrap_or_else(|| "Keine Rolle".to_string());
                view! {
                    <div class="dropdown dropdown-end">
                        <div tabindex="0" role="button" class="btn btn-ghost gap-2">
                            <Icon icon=icondata::LuCircleUser width="1.25rem" height="1.25rem"/>
                            <span class="hidden sm:inline">{b.name.clone()}</span>
                            <Icon icon=icondata::LuChevronDown attr:class="opacity-60"/>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu z-50 mt-2 w-60 rounded-box border border-base-300 bg-base-100 p-2 shadow-lg">
                            <li class="pointer-events-none px-3 py-2">
                                <div class="flex flex-col items-start gap-0.5 p-0">
                                    <span class="font-medium">{b.name}</span>
                                    <span class="text-xs text-base-content/60">{b.email}</span>
                                    <span class="badge badge-sm badge-primary badge-soft mt-1">{rolle}</span>
                                </div>
                            </li>
                            <li>
                                <button on:click=move |_| { sitzung.abmelden.dispatch(crate::backend::auth::Abmelden {}); }>
                                    <Icon icon=icondata::LuLogOut/>
                                    "Abmelden"
                                </button>
                            </li>
                        </ul>
                    </div>
                }
            })}
        </Transition>
    }
}
