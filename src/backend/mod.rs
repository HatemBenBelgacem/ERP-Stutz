//! Backend: Datenbank und Server-Funktionen.

pub mod api;
pub mod auth;
pub mod rollen;

#[cfg(feature = "ssr")]
pub mod microsoft;

#[cfg(feature = "ssr")]
pub mod db;
