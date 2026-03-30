// ============================================================================
// src/ui.rs — Desktop UI (egui/eframe)
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// Notes:
//   • HIDAPI is not guaranteed to be Send/Sync. We keep it inside the UI thread.
//   • To avoid spamming the device while dragging sliders, we use debounce
//     + auto-apply scheduling.
// ============================================================================

use anyhow::{anyhow, Context, Result};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use image::ImageReader;

use crate::mqtt_agent;
use crate::{config, hid, vialrgb};

// ── Public API ───────────────────────────────────────────────────────────────

pub fn run(cfg_path: PathBuf, initial_locale: String) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MARCOntroller")
            .with_icon(load_window_icon()?),
        ..Default::default()
    };

    // eframe::run_native blocks until the window is closed.
    // In eframe 0.33 the app creator returns Result<Box<dyn App>, DynError>.
    eframe::run_native(
        "MARCOntroller",
        native_options,
        Box::new(move |_cc| Ok(Box::new(MarcontrollerUi::new(cfg_path, initial_locale.clone())))),
    )
        .map_err(|e| anyhow!("eframe: {e}"))
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn load_window_icon() -> Result<egui::IconData> {
    let icon_path = PathBuf::from("assets/icon.png");

    let image = ImageReader::open(&icon_path)
        .with_context(|| format!("open window icon: {}", icon_path.display()))?
        .decode()
        .with_context(|| format!("decode window icon: {}", icon_path.display()))?
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Ok(egui::IconData {
        rgba,
        width,
        height,
    })
}

// ── UI model ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Control,
    Direct,
    Config,
}

#[derive(Debug, Clone)]
enum AgentEvent {
    Started,
    Error(String),
}

#[derive(Debug, Clone)]
enum AgentStatus {
    Started,
    Error(String),
}

struct MarcontrollerUi {
    // ── Sync state ─────────────────────────────────────────────────────────
    did_initial_sync: bool,
    last_online: bool,

    // ── Locale (UI runtime selector) ───────────────────────────────────────
    ui_locale: String,

    // ── Persistent config (TOML) ───────────────────────────────────────────
    cfg_path: PathBuf,
    cfg: config::AppConfig,
    dirty: bool,

    // Tracks whether config.toml was successfully loaded.
    // Used to decide if it is safe to auto-start the MQTT agent.
    cfg_loaded_ok: bool,

    tab: Tab,

    // ── HID state ─────────────────────────────────────────────────────────
    hid_api: Option<hidapi::HidApi>,
    kb_online: bool,
    last_probe: Instant,

    // ── Auto-apply scheduling ─────────────────────────────────────────────
    auto_apply: bool,
    debounce: Duration,

    solid_dirty: bool,
    solid_changed_at: Instant,

    direct_dirty: bool,
    direct_changed_at: Instant,
    effect_dirty: bool,
    effect_changed_at: Instant,

    // ── Control tab state (solid) ─────────────────────────────────────────
    solid_rgb: [u8; 3],
    solid_brightness: u8,

    // ── Control tab state (effect) ────────────────────────────────────────
    effect_id: u16,
    effect_speed: u8,

    // ── Direct tab state (per-LED) ─────────────────────────────────────────
    led_count: Option<u16>,
    led_index: u16,
    direct_rgb: [u8; 3],
    direct_brightness: u8,

    // ── Observability ──────────────────────────────────────────────────────
    last_info: Option<vialrgb::Info>,
    last_mode: Option<vialrgb::Mode>,
    last_error: Option<String>,

    // ── Agent (MQTT) ─────────────────────────────────────────────────────
    agent_started: bool,
    agent_tx: mpsc::Sender<AgentEvent>,
    agent_rx: mpsc::Receiver<AgentEvent>,
    agent_status: Option<AgentStatus>,
}

impl MarcontrollerUi {
    // ── Construction ──────────────────────────────────────────────────────

    fn new(cfg_path: PathBuf, initial_locale: String) -> Self {
        let (cfg, cfg_loaded_ok, last_error) = match config::load(&cfg_path) {
            Ok(c) => (c, true, None),
            Err(e) => (
                config::AppConfig::default(),
                false,
                Some(t!("ui.err_config_load_failed", error = format!("{e:#}")).to_string()),
            ),
        };

        let (agent_tx, agent_rx) = mpsc::channel::<AgentEvent>();

        Self {
            did_initial_sync: false,
            last_online: false,

            // Default UI locale: keep whatever main.rs set, but we don't have a direct getter.
            // Users can switch it from the dropdown.
            ui_locale: initial_locale,

            cfg_path,
            cfg,
            dirty: false,
            cfg_loaded_ok,
            tab: Tab::Control,

            hid_api: None,
            kb_online: false,
            last_probe: Instant::now() - Duration::from_secs(10),

            auto_apply: true,
            debounce: Duration::from_millis(180),

            solid_dirty: false,
            solid_changed_at: Instant::now(),

            direct_dirty: false,
            direct_changed_at: Instant::now(),
            effect_dirty: false,
            effect_changed_at: Instant::now(),

            solid_rgb: [255, 0, 0],
            solid_brightness: 128,

            effect_id: vialrgb::EFFECT_SOLID_COLOR,
            effect_speed: 0,

            led_count: None,
            led_index: 0,
            direct_rgb: [255, 255, 255],
            direct_brightness: 64,

            last_info: None,
            last_mode: None,
            last_error,

            agent_started: false,
            agent_tx,
            agent_rx,
            agent_status: None,        }
    }

    // ── Error handling helpers ─────────────────────────────────────────────

    fn mark_error(&mut self, e: anyhow::Error) {
        self.last_error = Some(format!("{e:#}"));
    }

    fn clear_error(&mut self) {
        self.last_error = None;
    }

    // ── HID helpers ─────────────────────────────────────────────────────────

    fn ensure_hid_api(&mut self) -> Result<&mut hidapi::HidApi> {
        if self.hid_api.is_none() {
            self.hid_api = Some(hidapi::HidApi::new().context("hidapi init")?);
        }
        Ok(self.hid_api.as_mut().unwrap())
    }

    fn parse_vid_pid(&self) -> Result<(u16, u16)> {
        let vid = parse_hex_u16(&self.cfg.hid.vid).context("parse vid")?;
        let pid = parse_hex_u16(&self.cfg.hid.pid).context("parse pid")?;
        Ok((vid, pid))
    }

    fn open_device(&mut self) -> Result<hidapi::HidDevice> {
        // Pre-compute everything that borrows `self` immutably before taking &mut HIDAPI.
        let (vid, pid) = self.parse_vid_pid()?;
        let serial = self.cfg.hid.serial.clone();

        let api = self.ensure_hid_api()?;
        let _ = api.refresh_devices();

        hid::open_device(api, vid, pid, serial.as_deref()).context("open_device")
    }

    /// Probe keyboard presence (and cache basic info) without changing lighting.
    ///
    /// Returns true if the device is reachable, false otherwise.
    fn probe_keyboard(&mut self, noisy: bool) -> bool {
        let res: Result<()> = (|| {
            let dev = self.open_device()?;
            let info = vialrgb::get_info(&dev).context("get_info")?;
            self.last_info = Some(info);
            Ok(())
        })();

        match res {
            Ok(()) => {
                self.kb_online = true;
                if noisy {
                    self.clear_error();
                }
                true
            }
            Err(e) => {
                let msg = format!("{e:#}");

                // "no_device" is a normal offline condition (hot-unplug / wrong VID/PID / etc.)
                if msg.contains("no_device") {
                    self.kb_online = false;
                    if noisy {
                        self.last_error = Some(msg);
                    }
                    return false;
                }

                self.kb_online = false;
                if noisy {
                    self.mark_error(e);
                }
                false
            }
        }
    }

    /// Read current keyboard mode/HSV and map into UI controls (best-effort).
    fn read_mode(&mut self) -> Result<()> {
        let dev = self.open_device()?;

        let m = vialrgb::get_mode(&dev).context("get_mode")?;
        self.last_mode = Some(m);

        // Keep UI effect controls aligned with the real keyboard state, but do
        // not overwrite a local pending edit while debounce is waiting to fire.
        if !self.effect_dirty {
            self.effect_id = m.mode;
            self.effect_speed = m.speed;
        }

        // Best-effort mapping:
        // - If OFF or v=0 => reflect brightness 0.
        // - Otherwise compute an approximate RGB from HSV for the color picker.
        if m.mode == vialrgb::EFFECT_OFF || m.v == 0 {
            self.solid_brightness = 0;
        } else {
            self.solid_brightness = m.v;
            self.solid_rgb = hsv_to_rgb(m.h, m.s, m.v);
        }

        Ok(())
    }

    /// Turn lighting OFF.
    fn set_off(&mut self) -> Result<()> {
        let dev = self.open_device()?;
        vialrgb::set_mode(&dev, vialrgb::EFFECT_OFF, 0, 0, 0, 0).context("set_mode off")?;
        Ok(())
    }

    // ── Apply helpers (solid / direct) ──────────────────────────────────────

    fn schedule_solid_apply(&mut self) {
        self.solid_dirty = true;
        self.solid_changed_at = Instant::now();
    }

    fn schedule_direct_apply(&mut self) {
        self.direct_dirty = true;
        self.direct_changed_at = Instant::now();
    }

    fn should_fire_solid(&self) -> bool {
        self.auto_apply && self.solid_dirty && self.solid_changed_at.elapsed() >= self.debounce
    }

    fn should_fire_direct(&self) -> bool {
        self.auto_apply && self.direct_dirty && self.direct_changed_at.elapsed() >= self.debounce
    }

    fn schedule_effect_apply(&mut self) {
        self.effect_dirty = true;
        self.effect_changed_at = Instant::now();
    }

    fn should_fire_effect(&self) -> bool {
        self.auto_apply && self.effect_dirty && self.effect_changed_at.elapsed() >= self.debounce
    }

    fn apply_solid_now(&mut self) -> Result<()> {
        let dev = self.open_device()?;

        let (h, s, _v_from_rgb) =
            vialrgb::rgb_to_hsv(self.solid_rgb[0], self.solid_rgb[1], self.solid_rgb[2]);
        let v = self.solid_brightness;

        // Preserve the current animation speed when switching to SOLID_COLOR from
        // the UI so colour/brightness changes do not silently reset speed to 0.
        let cur = vialrgb::get_mode(&dev).context("get_mode")?;

        vialrgb::set_mode(&dev, vialrgb::EFFECT_SOLID_COLOR, cur.speed, h, s, v)
            .context("set_mode solid_color")?;

        Ok(())
    }

    fn apply_effect_now(&mut self) -> Result<()> {
        let dev = self.open_device()?;

        // Avoid an extra HID round-trip on every effect change. Use the last mode
        // cached by the UI as the source of HSV values, and fall back to the solid
        // controls only if we do not have a previous keyboard snapshot yet.
        let (h, s, v) = if let Some(m) = self.last_mode {
            (m.h, m.s, m.v)
        } else {
            let (hh, ss, _vv) =
                vialrgb::rgb_to_hsv(self.solid_rgb[0], self.solid_rgb[1], self.solid_rgb[2]);
            (hh, ss, self.solid_brightness)
        };

        vialrgb::set_mode(
            &dev,
            self.effect_id,
            self.effect_speed,
            h,
            s,
            v,
        )
            .context("set_mode effect")?;

        Ok(())
    }

    fn refresh_led_count(&mut self) -> Result<u16> {
        let dev = self.open_device()?;
        let n = vialrgb::get_led_count(&dev).context("get_led_count")?;
        self.led_count = Some(n);

        // Clamp index into range when count changes.
        if n == 0 {
            self.led_index = 0;
        } else if self.led_index >= n {
            self.led_index = n - 1;
        }

        Ok(n)
    }

    fn apply_direct_led_now(&mut self) -> Result<()> {
        let dev = self.open_device()?;

        // Ensure we have a count to clamp index safely.
        let n = match self.led_count {
            Some(v) => v,
            None => self.refresh_led_count()?,
        };

        if n == 0 {
            anyhow::bail!("no_leds");
        }
        if self.led_index >= n {
            anyhow::bail!("led_index_out_of_range");
        }

        let (h, s, _vv) =
            vialrgb::rgb_to_hsv(self.direct_rgb[0], self.direct_rgb[1], self.direct_rgb[2]);
        let v = self.direct_brightness;

        // direct_fastset expects per-LED HSV tuples.
        vialrgb::direct_fastset(&dev, self.led_index, &[(h, s, v)]).context("direct_fastset")?;

        Ok(())
    }

    fn apply_direct_all_now(&mut self) -> Result<()> {
        let dev = self.open_device()?;

        let (h, s, _vv) =
            vialrgb::rgb_to_hsv(self.direct_rgb[0], self.direct_rgb[1], self.direct_rgb[2]);
        let v = self.direct_brightness;

        // This both enables DIRECT and paints all LEDs in one call.
        let _painted = vialrgb::direct_set_all(&dev, 0, h, s, v).context("direct_set_all")?;
        Ok(())
    }

    // ── UI rendering ────────────────────────────────────────────────────────

    fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Tabs
            ui.selectable_value(
                &mut self.tab,
                Tab::Control,
                t!("ui.tab_control").to_string(),
            );
            ui.selectable_value(
                &mut self.tab,
                Tab::Direct,
                t!("ui.tab_direct").to_string(),
            );
            ui.selectable_value(&mut self.tab, Tab::Config, t!("ui.tab_config").to_string());

            ui.separator();

            // Language selector (runtime)
            egui::ComboBox::from_id_salt("ui_lang")
                .selected_text(match self.ui_locale.as_str() {
                    "es" => t!("ui.lang_es").to_string(),
                    _ => t!("ui.lang_en").to_string(),
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.ui_locale == "en", t!("ui.lang_en").to_string())
                        .clicked()
                    {
                        self.ui_locale = "en".to_owned();
                        rust_i18n::set_locale("en");
                    }
                    if ui
                        .selectable_label(self.ui_locale == "es", t!("ui.lang_es").to_string())
                        .clicked()
                    {
                        self.ui_locale = "es".to_owned();
                        rust_i18n::set_locale("es");
                    }
                });

            ui.separator();

            // Actions
            if ui.button(t!("ui.btn_probe").to_string()).clicked() {
                self.probe_keyboard(true);
            }
            if ui.button(t!("ui.btn_read").to_string()).clicked() {
                if let Err(e) = self.read_mode() {
                    self.mark_error(e);
                }
            }

            ui.separator();

            if ui.button(t!("ui.btn_off").to_string()).clicked() {
                if let Err(e) = self.set_off() {
                    self.mark_error(e);
                } else if let Err(e) = self.read_mode() {
                    // Keep OFF applied even if read fails, but show the error.
                    self.mark_error(e);
                }
            }

            ui.separator();

            // Auto apply toggle (global)
            let changed = ui
                .checkbox(&mut self.auto_apply, t!("ui.label_auto_apply").to_string())
                .changed();

            // If auto-apply is toggled ON, fire pending changes sooner.
            if changed && self.auto_apply {
                self.solid_changed_at = Instant::now() - self.debounce;
                self.direct_changed_at = Instant::now() - self.debounce;
                self.effect_changed_at = Instant::now() - self.debounce;
            }
        });
    }

    fn ui_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let status = if self.kb_online {
                t!("ui.status_online").to_string()
            } else {
                t!("ui.status_offline").to_string()
            };

            ui.label(format!("{}: {status}", t!("ui.status_keyboard").to_string()));
            ui.separator();
            ui.label(format!("{}: {}", t!("ui.status_config").to_string(), self.cfg_path.display()));
        });

        if let Some(info) = self.last_info {
            ui.label(t!("ui.status_vialrgb_info",
                protocol = info.protocol_version,
                max_brightness = info.max_brightness
            ).to_string());
        }

        if let Some(m) = self.last_mode {
            ui.label(t!("ui.status_mode_info",
                id = m.mode,
                speed = m.speed,
                h = m.h,
                s = m.s,
                v = m.v
            ).to_string());
        }

        // Show current agent status using the active UI locale.
        if let Some(status) = &self.agent_status {
            let msg = match status {
                AgentStatus::Started => t!("ui.agent_started").to_string(),
                AgentStatus::Error(err) => t!("ui.agent_error", error = err).to_string(),
            };

            ui.label(msg);
        }

        if let Some(e) = &self.last_error {
            ui.separator();
            ui.label(format!("{}: {e}", t!("ui.status_error").to_string()));
        }
    }

    fn ui_control_tab(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(t!("ui.group_solid").to_string());

            ui.horizontal(|ui| {
                ui.label("Efecto:");

                let mut selected = self.effect_id;

                egui::ComboBox::from_id_salt("effect_selector")
                    .selected_text(ui_effect_name_for_id(selected))
                    .show_ui(ui, |ui| {
                        for &id in ui_effect_ids() {
                            ui.selectable_value(&mut selected, id, ui_effect_name_for_id(id));
                        }
                    });

                if selected != self.effect_id {
                    self.effect_id = selected;
                    self.schedule_effect_apply();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Velocidad:");

                let resp = ui.add(egui::Slider::new(&mut self.effect_speed, 0..=255));

                if resp.changed() {
                    self.schedule_effect_apply();
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}:", t!("ui.label_color").to_string()));
                let resp = ui.color_edit_button_srgb(&mut self.solid_rgb);
                if resp.changed() {
                    self.schedule_solid_apply();
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}:", t!("ui.label_brightness").to_string()));
                let resp = ui.add(egui::Slider::new(&mut self.solid_brightness, 0..=255));
                if resp.changed() {
                    self.schedule_solid_apply();
                }
            });

            // If auto-apply is disabled, still allow manual apply in this tab.
            if !self.auto_apply {
                if ui.button(t!("ui.btn_apply").to_string()).clicked() {
                    if let Err(e) = self.apply_solid_now() {
                        self.mark_error(e);
                    } else if let Err(e) = self.read_mode() {
                        self.mark_error(e);
                    }
                }
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(t!("ui.group_notes").to_string());
            ui.label(t!("ui.note_effect").to_string());
            ui.label(t!("ui.note_limits").to_string());
        });
    }

    fn ui_direct_tab(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui.button(t!("ui.btn_refresh_leds").to_string()).clicked() {
                    if let Err(e) = self.refresh_led_count() {
                        self.mark_error(e);
                    } else {
                        self.clear_error();
                    }
                }

                if let Some(n) = self.led_count {
                    ui.label(format!("{}: {n}", t!("ui.label_leds").to_string()));
                } else {
                    ui.label(format!("{}: {}", t!("ui.label_leds").to_string(), t!("ui.label_leds_unknown").to_string()));
                }
            });

            ui.add_space(6.0);

            // Index selector
            let max_idx = self.led_count.map(|n| n.saturating_sub(1)).unwrap_or(255);
            ui.horizontal(|ui| {
                ui.label(format!("{}:", t!("ui.label_index").to_string()));
                let resp = ui.add(egui::Slider::new(&mut self.led_index, 0..=max_idx));
                if resp.changed() {
                    // Changing index does not apply by itself.
                }
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(format!("{}:", t!("ui.label_color").to_string()));
                let resp = ui.color_edit_button_srgb(&mut self.direct_rgb);
                if resp.changed() {
                    self.schedule_direct_apply();
                }
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}:", t!("ui.label_brightness").to_string()));
                let resp = ui.add(egui::Slider::new(&mut self.direct_brightness, 0..=255));
                if resp.changed() {
                    self.schedule_direct_apply();
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                // When auto-apply is enabled, we apply "selected LED" on debounce.
                // Keep explicit buttons for "All LEDs" and for manual mode.
                if ui.button(t!("ui.btn_set_selected_led").to_string()).clicked() {
                    if let Err(e) = self.apply_direct_led_now() {
                        self.mark_error(e);
                    } else {
                        self.clear_error();
                    }
                }

                if ui.button(t!("ui.btn_set_all_leds").to_string()).clicked() {
                    if let Err(e) = self.apply_direct_all_now() {
                        self.mark_error(e);
                    } else {
                        self.clear_error();
                    }
                }
            });

            if !self.auto_apply {
                ui.add_space(6.0);
                ui.label(t!("ui.msg_auto_apply_disabled").to_string());
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label(t!("ui.direct_notes_title").to_string());
            ui.label(t!("ui.direct_note_1").to_string());
            ui.label(t!("ui.direct_note_2").to_string());
            ui.label(t!("ui.direct_note_3").to_string());
        });
    }

    fn ui_config_tab(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui.button(t!("ui.btn_reload").to_string()).clicked() {
                    match config::load(&self.cfg_path) {
                        Ok(c) => {
                            self.cfg = c;
                            self.dirty = false;
                            self.cfg_loaded_ok = true;
                            self.clear_error();
                        }
                        Err(e) => {
                            self.cfg_loaded_ok = false;
                            self.mark_error(e);
                        }
                    }
                }

                let save_btn = ui.add_enabled(self.dirty, egui::Button::new(t!("ui.btn_save").to_string()));
                if save_btn.clicked() {
                    if let Err(e) = config::save(&self.cfg_path, &self.cfg) {
                        self.mark_error(e);
                    } else {
                        self.dirty = false;
                        self.cfg_loaded_ok = true;
                        self.clear_error();
                    }
                }
            });

            ui.add_space(8.0);

            egui::Grid::new("cfg_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    // HID
                    ui.label(t!("ui.cfg_hid_vid").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.hid.vid).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_hid_pid").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.hid.pid).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_hid_serial").to_string());
                    let mut serial_txt = self.cfg.hid.serial.clone().unwrap_or_default();
                    let resp = ui.text_edit_singleline(&mut serial_txt);
                    if resp.changed() {
                        self.cfg.hid.serial = if serial_txt.trim().is_empty() {
                            None
                        } else {
                            Some(serial_txt)
                        };
                        self.dirty = true;
                    }
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    // MQTT
                    ui.label(t!("ui.cfg_mqtt_host").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.mqtt.host).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_mqtt_port").to_string());
                    self.dirty |= ui
                        .add(egui::DragValue::new(&mut self.cfg.mqtt.port).range(1..=65535))
                        .changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_mqtt_username").to_string());
                    let mut user_txt = self.cfg.mqtt.username.clone().unwrap_or_default();
                    let resp = ui.text_edit_singleline(&mut user_txt);
                    if resp.changed() {
                        self.cfg.mqtt.username = Some(user_txt);
                        self.dirty = true;
                    }
                    ui.end_row();

                    ui.label(t!("ui.cfg_mqtt_password").to_string());
                    let mut pass_txt = self.cfg.mqtt.password.clone().unwrap_or_default();
                    let resp = ui.add(egui::TextEdit::singleline(&mut pass_txt).password(true));
                    if resp.changed() {
                        self.cfg.mqtt.password = Some(pass_txt);
                        self.dirty = true;
                    }
                    ui.end_row();

                    ui.label(t!("ui.cfg_mqtt_client_id").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.mqtt.client_id).changed();
                    ui.end_row();

                    ui.separator();
                    ui.separator();
                    ui.end_row();

                    // Home Assistant
                    ui.label(t!("ui.cfg_ha_prefix").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.ha.discovery_prefix).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_ha_object_id").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.ha.object_id).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_ha_unique_id").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.ha.unique_id).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_ha_name").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.ha.name).changed();
                    ui.end_row();

                    ui.label(t!("ui.cfg_ha_base_topic").to_string());
                    self.dirty |= ui.text_edit_singleline(&mut self.cfg.ha.base_topic).changed();
                    ui.end_row();
                });
        });
    }

    // ── Tick (background orchestration) ─────────────────────────────────────

    fn tick_sync(&mut self) {
        // Start the MQTT agent automatically when the UI is running.
        self.start_agent_if_needed();

        // Drain agent messages without blocking the UI thread.
        while let Ok(event) = self.agent_rx.try_recv() {
            match event {
                AgentEvent::Started => {
                    self.agent_status = Some(AgentStatus::Started);
                }
                AgentEvent::Error(err) => {
                    self.agent_status = Some(AgentStatus::Error(err.clone()));
                    self.last_error = Some(t!("ui.agent_error", error = err).to_string());
                }
            }
        }

        // ── Initial sync ───────────────────────────────────────────────────
        //
        // Avoid showing fake/default values: do a best-effort probe + read once.
        if !self.did_initial_sync {
            let online = self.probe_keyboard(false);
            self.last_online = online;

            if online {
                if let Err(e) = self.read_mode() {
                    self.mark_error(e);
                }
            }

            self.did_initial_sync = true;
        }

        // ── Periodic probe (hot-plug + state refresh) ──────────────────────
        //
        // Detect online/offline transitions and also refresh the real keyboard
        // mode periodically so the status bar reflects changes made from HA,
        // Vial or any other external HID client.
        if self.last_probe.elapsed() >= Duration::from_secs(2) {
            self.last_probe = Instant::now();

            let online_now = self.probe_keyboard(false);

            if online_now {
                if let Err(e) = self.read_mode() {
                    self.mark_error(e);
                }

                // Populate LED count lazily when we come online.
                if !self.last_online && self.led_count.is_none() {
                    let _ = self.refresh_led_count();
                }
            }

            self.last_online = online_now;
        }

        // ── Auto-apply (debounced) ─────────────────────────────────────────
        //
        // We only write when the keyboard is reachable.
        if self.kb_online {
            if self.should_fire_solid() {
                if let Err(e) = self.apply_solid_now() {
                    self.mark_error(e);
                } else {
                    self.solid_dirty = false;
                    // Sync from device for consistent UI.
                    let _ = self.read_mode();
                }
            }

            if self.should_fire_effect() {
                if let Err(e) = self.apply_effect_now() {
                    self.mark_error(e);
                } else {
                    self.effect_dirty = false;

                    // Update the local status optimistically and let the periodic
                    // probe confirm the real keyboard state shortly afterwards.
                    let (h, s, v) = if let Some(m) = self.last_mode {
                        (m.h, m.s, m.v)
                    } else {
                        let (hh, ss, _vv) =
                            vialrgb::rgb_to_hsv(self.solid_rgb[0], self.solid_rgb[1], self.solid_rgb[2]);
                        (hh, ss, self.solid_brightness)
                    };

                    self.last_mode = Some(vialrgb::Mode {
                        mode: self.effect_id,
                        speed: self.effect_speed,
                        h,
                        s,
                        v,
                    });
                }
            }

            if self.should_fire_direct() {
                if let Err(e) = self.apply_direct_led_now() {
                    self.mark_error(e);
                } else {
                    self.direct_dirty = false;
                }
            }
        }
    }

    // ── MQTT agent bootstrap ──────────────────────────────────────────────
    //
    // The UI runs on a synchronous event loop (eframe), while the MQTT agent
    // is async and long-lived. We run the agent on a dedicated thread with its
    // own Tokio runtime.
    fn start_agent_if_needed(&mut self) {
        if self.agent_started {
            return;
        }

        // Avoid starting the agent if config.toml failed to load.
        if !self.cfg_loaded_ok {
            return;
        }

        // Clone config for the agent thread.
        let cfg = self.cfg.clone();
        let tx = self.agent_tx.clone();

        std::thread::spawn(move || {
            // Build a Tokio runtime for the agent.
            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error(format!("tokio runtime: {e}")));
                    return;
                }
            };

            let _ = tx.send(AgentEvent::Started);

            // Run forever (reconnect loop lives inside mqtt_agent::run).
            let res = rt.block_on(async { mqtt_agent::run(cfg).await });

            if let Err(e) = res {
                let _ = tx.send(AgentEvent::Error(format!("{e:#}")));
            }
        });

        self.agent_started = true;
    }
}

// ── eframe integration ───────────────────────────────────────────────────────

impl eframe::App for MarcontrollerUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick_sync();

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            self.ui_top_bar(ui);
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            self.ui_status_bar(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Control => self.ui_control_tab(ui),
            Tab::Direct => self.ui_direct_tab(ui),
            Tab::Config => self.ui_config_tab(ui),
        });

        // Keep the UI responsive during idle (probe + debounce).
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Parse a hex string (with or without "0x" prefix) into a `u16`.
fn parse_hex_u16(s: &str) -> Result<u16> {
    let clean = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(clean, 16).context("parse_hex_u16")
}

/// Convert HSV (0–255) to RGB (0–255) using integer math.
///
/// This is a common 8-bit HSV->RGB conversion (region/remainder).
fn hsv_to_rgb(h: u8, s: u8, v: u8) -> [u8; 3] {
    if s == 0 {
        return [v, v, v];
    }

    let region: u8 = h / 43; // 0..5
    let remainder: u16 = ((h - (region * 43)) as u16) * 6; // 0..255

    let v16 = v as u16;
    let s16 = s as u16;

    let p: u16 = (v16 * (255 - s16)) / 255;
    let q: u16 = (v16 * (255 - ((s16 * remainder) / 255))) / 255;
    let t: u16 = (v16 * (255 - ((s16 * (255 - remainder)) / 255))) / 255;

    let (r, g, b) = match region {
        0 => (v16, t, p),
        1 => (q, v16, p),
        2 => (p, v16, t),
        3 => (p, q, v16),
        4 => (t, p, v16),
        _ => (v16, p, q),
    };

    [r as u8, g as u8, b as u8]
}

fn ui_effect_name_for_id(id: u16) -> String {
    match id {
        0  => "OFF".to_owned(),
        1  => "DIRECT".to_owned(),
        2  => "Solid Color".to_owned(),
        3  => "Alpha Mods".to_owned(),
        4  => "Gradient Up Down".to_owned(),
        5  => "Gradient Left Right".to_owned(),
        6  => "Breathing".to_owned(),
        7  => "Band Sat".to_owned(),
        8  => "Band Val".to_owned(),
        9  => "Band Pinwheel Sat".to_owned(),
        10 => "Band Pinwheel Val".to_owned(),
        11 => "Band Spiral Sat".to_owned(),
        12 => "Band Spiral Val".to_owned(),
        13 => "Cycle All".to_owned(),
        14 => "Cycle Left Right".to_owned(),
        15 => "Cycle Up Down".to_owned(),
        16 => "Rainbow Moving Chevron".to_owned(),
        17 => "Cycle Out In".to_owned(),
        18 => "Cycle Out In Dual".to_owned(),
        19 => "Cycle Pinwheel".to_owned(),
        20 => "Cycle Spiral".to_owned(),
        21 => "Dual Beacon".to_owned(),
        22 => "Rainbow Beacon".to_owned(),
        23 => "Rainbow Pinwheels".to_owned(),
        24 => "Flower Blooming".to_owned(),
        25 => "Raindrops".to_owned(),
        26 => "Jellybean Raindrops".to_owned(),
        27 => "Hue Breathing".to_owned(),
        28 => "Hue Pendulum".to_owned(),
        29 => "Hue Wave".to_owned(),
        30 => "Pixel Rain".to_owned(),
        31 => "Pixel Flow".to_owned(),
        32 => "Pixel Fractal".to_owned(),
        33 => "Typing Heatmap".to_owned(),
        34 => "Digital Rain".to_owned(),
        35 => "Solid Reactive Simple".to_owned(),
        36 => "Solid Reactive".to_owned(),
        37 => "Solid Reactive Wide".to_owned(),
        38 => "Solid Reactive Cross".to_owned(),
        39 => "Solid Reactive Nexus".to_owned(),
        40 => "Splash".to_owned(),
        41 => "Solid Splash".to_owned(),
        42 => "Starlight Smooth".to_owned(),
        43 => "Starlight".to_owned(),
        44 => "Starlight Dual Sat".to_owned(),
        45 => "Starlight Dual Hue".to_owned(),
        46 => "Riverflow".to_owned(),
        other => format!("Effect {other}"),
    }
}

fn ui_effect_ids() -> &'static [u16] {
    &[
        2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
        38, 39, 40, 41, 42, 43, 44, 45, 46,
    ]
}