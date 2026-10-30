//! Backend: Datenbank und Server-Funktionen.

pub mod api;
pub mod artikel;
pub mod auth;
pub mod projekte;
pub mod rollen;

#[cfg(feature = "ssr")]
pub mod microsoft;

#[cfg(feature = "ssr")]
pub mod pdf;

#[cfg(feature = "ssr")]
pub mod db;
