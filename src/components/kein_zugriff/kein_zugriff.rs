use leptos::prelude::*;
use leptos_icons::Icon;

/// Für angemeldete Benutzer, denen noch kein Bereich freigegeben ist.
#[component]
pub fn KeinZugriff() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-3 py-16 text-center">
            <span class="rounded-box bg-warning/15 p-4 text-warning">
                <Icon icon=icondata::LuLock width="2rem" height="2rem"/>
            </span>
            <h1 class="text-xl font-bold">"Noch kein Zugriff"</h1>
            <p class="max-w-md text-base-content/70">
                "Dein Konto ist angelegt, aber dir sind noch keine Bereiche freigegeben. "
                "Bitte wende dich an einen Administrator."
            </p>
        </div>
    }
}
