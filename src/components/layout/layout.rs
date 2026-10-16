//! Grundgerüst der App: Header oben, einklappbare Sidebar links, Inhalt rechts.

use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::{A, Outlet};

use crate::app::use_sitzung;

const NAVIGATION: [(&str, &str, icondata::Icon); 4] = [
    ("/", "Übersicht", icondata::LuLayoutDashboard),
    ("/termine", "Termine", icondata::LuCalendar),
    ("/kurse", "Kurse", icondata::LuBookOpen),
    ("/mitarbeiter", "Mitarbeiter", icondata::LuUsers),
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
        <header class="navbar z-50 h-16 min-h-16 gap-2 border-b border-base-300 bg-base-100 px-3">
            <button
                class="btn btn-ghost btn-square md:hidden"
                aria-label="Navigation öffnen"
                on:click=move |_| mobil_offen.update(|o| *o = !*o)
            >
                <Icon icon=icondata::LuMenu width="1.25rem" height="1.25rem"/>
            </button>
            <button
                class="btn btn-ghost btn-square hidden lg:inline-flex"
                title=move || if eingeklappt.get() { "Navigation ausklappen" } else { "Navigation einklappen" }
                aria-label="Navigation ein- oder ausklappen"
                on:click=move |_| eingeklappt.update(|e| *e = !*e)
            >
                {move || {
                    let symbol = if eingeklappt.get() { icondata::LuPanelLeftOpen } else { icondata::LuPanelLeftClose };
                    view! { <Icon icon=symbol width="1.25rem" height="1.25rem"/> }
                }}
            </button>
            <A href="/" attr:class="flex items-center gap-2 text-lg font-semibold">
                <span class="rounded-field bg-primary p-1.5 text-primary-content">
                    <Icon icon=icondata::LuGraduationCap width="1.25rem" height="1.25rem"/>
                </span>
                "Weiterbildung"
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
    let beschriftung = move || beschriftung_klassen(eingeklappt.get());

    view! {
        <Show when=move || mobil_offen.get()>
            <div class="fixed inset-0 top-16 z-30 bg-base-content/20 md:hidden" on:click=move |_| mobil_offen.set(false)></div>
        </Show>
        <aside class=klassen aria-label="Hauptnavigation">
            <ul class="menu w-full gap-1 p-2">
                {NAVIGATION
                    .into_iter()
                    .map(|(href, text, icon)| view! { <NavEintrag href text icon eingeklappt mobil_offen/> })
                    .collect_view()}
                <Transition>
                    <Show when=move || sitzung.ist_admin() == Some(true)>
                        <li class="menu-title mt-2">
                            <span class=beschriftung>"Verwaltung"</span>
                        </li>
                        <NavEintrag href="/benutzer" text="Benutzer" icon=icondata::LuUserCog eingeklappt mobil_offen/>
                    </Show>
                </Transition>
            </ul>
        </aside>
    }
}

/// Menütexte: auf dem Tablet immer ausgeblendet, auf dem Desktop nur eingeklappt.
fn beschriftung_klassen(eingeklappt: bool) -> &'static str {
    if eingeklappt { "md:hidden" } else { "md:max-lg:hidden" }
}

#[component]
fn NavEintrag(
    href: &'static str,
    text: &'static str,
    icon: icondata::Icon,
    eingeklappt: RwSignal<bool>,
    mobil_offen: RwSignal<bool>,
) -> impl IntoView {
    let beschriftung = move || beschriftung_klassen(eingeklappt.get());
    view! {
        <li on:click=move |_| mobil_offen.set(false)>
            <A
                href=href
                exact=true
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
                let rolle = if b.ist_admin() { "Administrator" } else { "Benutzer" };
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
