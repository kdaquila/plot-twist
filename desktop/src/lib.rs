//! plot-twist desktop shell: wires the core `Session` to the WebView (Tauri commands and
//! events) and to the local HTTP API.

mod commands;
mod events;
mod startup;

pub use startup::run;
