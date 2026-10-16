#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use weiterbildung::app::*;

    dotenvy::dotenv().ok();
    if let Err(e) = weiterbildung::backend::db::init().await {
        eprintln!("Datenbank konnte nicht initialisiert werden: {e}");
        std::process::exit(1);
    }

    let conf = get_configuration(None).unwrap();
    let mut leptos_options = conf.leptos_options;
    // Hoster wie Railway geben den Port über PORT vor
    if let Some(port) = std::env::var("PORT").ok().and_then(|p| p.parse::<u16>().ok()) {
        leptos_options.site_addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    }
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        // Anmeldung mit Microsoft (OAuth-Weiterleitung und Rückkehr)
        .route("/auth/microsoft/start", axum::routing::get(weiterbildung::backend::microsoft::start))
        .route("/auth/microsoft/callback", axum::routing::get(weiterbildung::backend::microsoft::rueckruf))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("Server läuft auf http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Im Browser startet `hydrate()` aus lib.rs die App.
}
