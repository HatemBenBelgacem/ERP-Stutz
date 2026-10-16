//! Backend: Datenbank und Server-Funktionen.

pub mod api;
pub mod auth;

#[cfg(feature = "ssr")]
pub mod db;
