// ============================================================================
// MARCOntroller — VialRGB keyboard lighting controller over USB HID
// ============================================================================
//
// MIT License
//
// Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// ============================================================================

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");

use clap::Parser;
use tracing_subscriber::EnvFilter;


mod cli;
mod config;
mod hid;
mod vialrgb;
mod mqtt_agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Logging ─────────────────────────────────────────────────────────────
    //
    // Controlled via RUST_LOG, e.g.:
    //   PowerShell:  $env:RUST_LOG="info"
    //   CMD:         set RUST_LOG=info
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // ── 1. Parse CLI args ────────────────────────────────────────────────────
    let cli = cli::Cli::parse();

    // ── 2. Detect and set locale ─────────────────────────────────────────────
    let locale = detect_lang(cli.lang.as_deref());
    rust_i18n::set_locale(&locale);

    // ── 3. Dispatch ──────────────────────────────────────────────────────────
    // Still sync for now; later we will make cli::run async when we add the agent.
    cli::run(cli).await
}

/// Resolve the active locale code from (in priority order):
///   1. `--lang` CLI flag  
///   2. `MARCOCONTROLLER_LANG` environment variable  
///   3. `LANG` system environment variable (first two chars)  
///   4. `"en"` built-in default  
///
/// Returns a locale code string (e.g. `"en"`, `"es"`).
fn detect_lang(cli_override: Option<&str>) -> String {
    if let Some(code) = cli_override {
        return normalise_locale(code);
    }
    if let Ok(v) = std::env::var("MARCOCONTROLLER_LANG") {
        return normalise_locale(&v);
    }
    if let Ok(v) = std::env::var("LANG") {
        // System LANG is typically "es_ES.UTF-8" — take just the language part.
        return normalise_locale(&v[..v.len().min(2)]);
    }
    "en".to_owned()
}

/// Map a locale string to a supported code, defaulting to `"en"`.
///
/// Extend the match arm when new locales are added to locales/.
fn normalise_locale(code: &str) -> String {
    match code.to_lowercase().as_str() {
        s if s.starts_with("es") => "es".to_owned(),
        s if s.starts_with("en") => "en".to_owned(),
        _ => "en".to_owned(),
    }
}