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
use sys_locale::get_locale;

mod cli;
mod config;
mod hid;
mod vialrgb;
mod mqtt_agent;
mod ui;

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
    cli::run(cli, locale).await}

/// Resolve the active locale code from (in priority order):
///   1. `--lang` CLI flag
///   2. `MARCOCONTROLLER_LANG` environment variable
///   3. `LC_ALL` / `LC_MESSAGES` / `LANG` / `LANGUAGE`
///   4. System locale via platform API
///   5. `"en"` built-in default
///
/// Returns a supported locale code string (e.g. `"en"`, `"es"`).
fn detect_lang(cli_override: Option<&str>) -> String {
    if let Some(code) = cli_override {
        return normalise_locale(code);
    }

    if let Ok(v) = std::env::var("MARCOCONTROLLER_LANG") {
        if !v.trim().is_empty() {
            return normalise_locale(&v);
        }
    }

    for key in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(v) = std::env::var(key) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return normalise_locale(trimmed);
            }
        }
    }

    if let Some(v) = get_locale() {
        return normalise_locale(&v);
    }

    "en".to_owned()
}

/// Map a locale string to a supported code, defaulting to `"en"`.
///
/// Accepts common forms such as:
/// - "es"
/// - "es-ES"
/// - "es_ES.UTF-8"
/// - "en-US"
/// - "English_United States"
fn normalise_locale(code: &str) -> String {
    let lower = code.trim().to_lowercase();

    let primary = lower
        .split(['-', '_', '.', '@'])
        .next()
        .unwrap_or("");

    match primary {
        "es" => "es".to_owned(),
        "en" => "en".to_owned(),
        _ => {
            if lower.starts_with("spanish") {
                "es".to_owned()
            } else if lower.starts_with("english") {
                "en".to_owned()
            } else {
                "en".to_owned()
            }
        }
    }
}