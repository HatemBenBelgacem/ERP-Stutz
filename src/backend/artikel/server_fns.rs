//! Server-Funktionen für Artikel: werden im Browser aufgerufen und laufen auf dem Server.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use uuid::Uuid;

use super::model::{Artikel, ArtikelEingabe};
#[cfg(feature = "ssr")]
use {
    super::{fehler, repository},
    crate::backend::auth::server::recht_pflicht,
    crate::backend::db::pool,
    crate::models::{Aktion, Bereich},
};

#[server(input = Json)]
pub async fn artikel_laden() -> Result<Vec<Artikel>, ServerFnError> {
    recht_pflicht(&[(Bereich::Lager, Aktion::Sehen)]).await?;
    repository::alle(pool()).await.map_err(fehler)
}

#[server(input = Json)]
pub async fn artikel_anlegen(eingabe: ArtikelEingabe) -> Result<Uuid, ServerFnError> {
    recht_pflicht(&[(Bereich::Lager, Aktion::Anlegen)]).await?;
    eingabe.pruefen().map_err(ServerFnError::new)?;
    repository::einfuegen(pool(), &eingabe)
        .await
        .map_err(fehler)
}

#[server(input = Json)]
pub async fn artikel_bearbeiten(id: Uuid, eingabe: ArtikelEingabe) -> Result<(), ServerFnError> {
    recht_pflicht(&[(Bereich::Lager, Aktion::Bearbeiten)]).await?;
    eingabe.pruefen().map_err(ServerFnError::new)?;
    if repository::aktualisieren(pool(), id, &eingabe)
        .await
        .map_err(fehler)?
        == 0
    {
        return Err(ServerFnError::new("Der Artikel existiert nicht mehr."));
    }
    Ok(())
}

#[server(input = Json)]
pub async fn artikel_loeschen(id: Uuid) -> Result<(), ServerFnError> {
    recht_pflicht(&[(Bereich::Lager, Aktion::Loeschen)]).await?;
    if repository::loeschen(pool(), id).await.map_err(fehler)? == 0 {
        return Err(ServerFnError::new("Der Artikel existiert nicht mehr."));
    }
    Ok(())
}
