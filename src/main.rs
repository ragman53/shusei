//! Shusei - Offline reading app with OCR and STT capabilities
//!
//! Main entry point for the application.

use dioxus::prelude::*;

mod app;
mod core;
mod platform;
mod ui;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    log::info!("Starting Shusei...");
    
    // Launch the Dioxus application
    dioxus::launch(app::App);
}