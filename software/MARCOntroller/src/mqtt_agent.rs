// ============================================================================
// src/mqtt_agent.rs — MQTT agent + Home Assistant Discovery (MQTT Light JSON)
// ============================================================================
//
// MIT License — Copyright (c) 2026 Jesús Guillén (jguillen-lab)
//
// ============================================================================

use anyhow::{anyhow, Context, Result};
use rumqttc::{AsyncClient, Event, Incoming, LastWill, MqttOptions, Outgoing, Publish, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, MissedTickBehavior};
use tracing::{info, warn};

use crate::{config::AppConfig, hid, vialrgb};

// ── HA JSON payload types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HaRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HaLightCommand {
    pub state: Option<String>,
    pub brightness: Option<u8>,
    pub color: Option<HaRgb>,
    pub effect: Option<String>, // optional for now
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HaLightState {
    pub state: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<HaRgb>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
}

impl HaLightState {
    pub fn off() -> Self {
        Self {
            state: "OFF".to_owned(),
            brightness: None,
            color_mode: None,
            color: None,
            effect: None,
        }
    }
}

// ── Effect catalogue ─────────────────────────────────────────────────────────
//
// Home Assistant expects effect names (strings), while VialRGB exposes effect
// IDs (u16). For now we keep the mapping simple and deterministic:
//   • OFF and DIRECT are not exposed as HA effects
//   • Known RGB Matrix effects get readable HA names
//   • Unknown IDs fall back to a generic "Effect <id>" name
//

#[derive(Debug, Clone)]
struct EffectCatalog {
    ha_names: Vec<String>,
}

fn ha_effect_name_for_id(id: u16) -> String {
    match id {
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

fn effect_id_for_ha_name(name: &str) -> Option<u16> {
    // Accept:
    //   • pretty HA names ("Solid Color")
    //   • machine-like aliases ("solid_color")
    //   • legacy compatibility names ("vialrgb_6")
    //   • generic fallback names ("effect_46", "Effect 46")
    let token = name
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '/'], "_");

    match token.as_str() {
        "solid_color" => Some(2),
        "alpha_mods" => Some(3),
        "gradient_up_down" => Some(4),
        "gradient_left_right" => Some(5),
        "breathing" => Some(6),
        "band_sat" => Some(7),
        "band_val" => Some(8),
        "band_pinwheel_sat" => Some(9),
        "band_pinwheel_val" => Some(10),
        "band_spiral_sat" => Some(11),
        "band_spiral_val" => Some(12),
        "cycle_all" => Some(13),
        "cycle_left_right" => Some(14),
        "cycle_up_down" => Some(15),
        "rainbow_moving_chevron" => Some(16),
        "cycle_out_in" => Some(17),
        "cycle_out_in_dual" => Some(18),
        "cycle_pinwheel" => Some(19),
        "cycle_spiral" => Some(20),
        "dual_beacon" => Some(21),
        "rainbow_beacon" => Some(22),
        "rainbow_pinwheels" => Some(23),
        "flower_blooming" => Some(24),
        "raindrops" => Some(25),
        "jellybean_raindrops" => Some(26),
        "hue_breathing" => Some(27),
        "hue_pendulum" => Some(28),
        "hue_wave" => Some(29),
        "pixel_rain" => Some(30),
        "pixel_flow" => Some(31),
        "pixel_fractal" => Some(32),
        "typing_heatmap" => Some(33),
        "digital_rain" => Some(34),
        "solid_reactive_simple" => Some(35),
        "solid_reactive" => Some(36),
        "solid_reactive_wide" => Some(37),
        "solid_reactive_cross" => Some(38),
        "solid_reactive_nexus" => Some(39),
        "splash" => Some(40),
        "solid_splash" => Some(41),
        "starlight_smooth" => Some(42),
        "starlight" => Some(43),
        "starlight_dual_sat" => Some(44),
        "starlight_dual_hue" => Some(45),
        "riverflow" => Some(46),
        _ => {
            if let Some(suffix) = token.strip_prefix("vialrgb_") {
                return suffix.parse::<u16>().ok();
            }
            if let Some(suffix) = token.strip_prefix("effect_") {
                return suffix.parse::<u16>().ok();
            }
            None
        }
    }
}

fn build_effect_catalog(ids: Vec<u16>) -> EffectCatalog {
    let ha_names = ids
        .into_iter()
        .filter(|id| *id != vialrgb::EFFECT_OFF && *id != vialrgb::EFFECT_DIRECT)
        .map(ha_effect_name_for_id)
        .collect();

    EffectCatalog { ha_names }
}

// ── Topics ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Topics {
    discovery_topic: String,
    command_topic: String,
    state_topic: String,
    availability_topic: String,
    ha_status_topic: String,
}

impl Topics {
    fn from_cfg(cfg: &AppConfig) -> Self {
        // Home Assistant discovery topic format:
        //   <discovery_prefix>/light/<object_id>/config
        let discovery_topic = format!(
            "{}/light/{}/config",
            cfg.ha.discovery_prefix.trim_end_matches('/'),
            cfg.ha.object_id
        );

        let base = cfg.ha.base_topic.trim_end_matches('/');
        let command_topic = format!("{base}/set");
        let state_topic = format!("{base}/state");
        let availability_topic = format!("{base}/availability");

        Self {
            discovery_topic,
            command_topic,
            state_topic,
            availability_topic,
            ha_status_topic: "homeassistant/status".to_owned(),
        }
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

pub async fn run(cfg: AppConfig) -> Result<()> {
    // Simple reconnect loop.
    loop {
        if let Err(e) = run_once(cfg.clone()).await {
            warn!("mqtt_agent run_once error: {e:#}");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

// ── Core loop ────────────────────────────────────────────────────────────────

async fn run_once(cfg: AppConfig) -> Result<()> {
    let topics = Topics::from_cfg(&cfg);

    // Start HID worker.
    let (hid_tx, hid_rx) = mpsc::channel::<HidJob>(32);
    std::thread::spawn({
        let cfg = cfg.clone();
        move || hid_worker(cfg, hid_rx)
    });

    // MQTT connection options.
    let mut opts = MqttOptions::new(
        cfg.mqtt.client_id.clone(),
        cfg.mqtt.host.clone(),
        cfg.mqtt.port,
    );
    opts.set_keep_alive(Duration::from_secs(cfg.mqtt.keep_alive_secs as u64));

    // Only set credentials when username is non-empty.
    if let Some(u) = cfg.mqtt.username.as_deref().filter(|s| !s.is_empty()) {
        let p = cfg.mqtt.password.as_deref().unwrap_or("");
        opts.set_credentials(u, p);
    }

    // LWT: if the agent dies unexpectedly, availability will be "offline".
    opts.set_last_will(LastWill::new(
        topics.availability_topic.clone(),
        "offline",
        QoS::AtLeastOnce,
        true,
    ));

    let (client, mut eventloop) = AsyncClient::new(opts, 10);

    // Subscribe to command topic.
    client
        .subscribe(topics.command_topic.clone(), QoS::AtLeastOnce)
        .await
        .context("mqtt subscribe command_topic")?;

    // Subscribe to HA birth message if configured.
    if cfg.ha.republish_on_ha_birth {
        client
            .subscribe(topics.ha_status_topic.clone(), QoS::AtLeastOnce)
            .await
            .context("mqtt subscribe ha_status_topic")?;
    }

    // Cache the last state we published to MQTT so we can detect out-of-band
    // keyboard changes (UI, Vial, external HID tools) and resync Home Assistant.
    let mut last_published_state: Option<HaLightState>;

    // Remember last "ON" parameters (used when HA sends ON without color/brightness).
    let mut saved_on = SavedOn {
        brightness: 128,
        color: HaRgb { r: 255, g: 0, b: 0 },
        effect: None,
    };

    // Determine initial keyboard availability via a probe.
    let mut kb_online = match hid_probe(&hid_tx).await {
        Ok(v) => v,
        Err(e) => {
            warn!("hid probe failed: {e:#}");
            false
        }
    };

    // Read supported keyboard effects (best-effort) so HA only advertises what
    // this firmware actually supports.
    let mut effect_catalog: Option<EffectCatalog> = None;

    if kb_online {
        if let Ok(Some(ids)) = hid_get_supported_effects(&hid_tx).await {
            effect_catalog = Some(build_effect_catalog(ids));
        }
    }

    // Publish discovery at startup (optional).
    if cfg.ha.publish_discovery_on_start {
        publish_discovery(&client, &cfg, &topics, effect_catalog.as_ref()).await?;
    }

    publish_availability(&client, &topics, kb_online).await?;

    // Publish INITIAL state based on real keyboard state when available.
    if kb_online {
        if let Ok(Some(st)) = hid_get_state(&hid_tx).await {
            update_saved_on_from_state(&mut saved_on, &st);
            publish_state(&client, &cfg, &topics, &st).await?;
            last_published_state = Some(st);
        } else {
            // Fallback: avoid "unknown" in HA.
            let st = HaLightState::off();
            publish_state(&client, &cfg, &topics, &st).await?;
            last_published_state = Some(st);
        }
    } else {
        // Keyboard offline: keep a deterministic retained state.
        let st = HaLightState::off();
        publish_state(&client, &cfg, &topics, &st).await?;
        last_published_state = Some(st);
    }

    // Periodic keyboard probe (hot-plug detection).
    let mut kb_probe = interval(Duration::from_secs(2));
    kb_probe.set_missed_tick_behavior(MissedTickBehavior::Delay);

    // Event loop (MQTT + periodic HID probe).
    loop {
        tokio::select! {
                        _ = kb_probe.tick() => {
                let online_now = hid_probe(&hid_tx).await.unwrap_or(false);

                if online_now != kb_online {
                    kb_online = online_now;
                    let _ = publish_availability(&client, &topics, kb_online).await;

                    info!("keyboard availability changed: {}", if kb_online { "online" } else { "offline" });

                    // When coming online, immediately sync the real state.
                    if kb_online {
                        if let Ok(Some(st)) = hid_get_state(&hid_tx).await {
                            update_saved_on_from_state(&mut saved_on, &st);

                            if last_published_state.as_ref() != Some(&st) {
                                let _ = publish_state(&client, &cfg, &topics, &st).await;
                                last_published_state = Some(st);
                            }
                        }
                    } else {
                        let st = HaLightState::off();
                        if last_published_state.as_ref() != Some(&st) {
                            let _ = publish_state(&client, &cfg, &topics, &st).await;
                            last_published_state = Some(st);
                        }
                    }
                } else if kb_online {
                    // Poll the real keyboard state even when availability has not
                    // changed so Home Assistant stays in sync with UI/Vial/manual HID changes.
                    if let Ok(Some(st)) = hid_get_state(&hid_tx).await {
                        update_saved_on_from_state(&mut saved_on, &st);

                        if last_published_state.as_ref() != Some(&st) {
                            let _ = publish_state(&client, &cfg, &topics, &st).await;
                            last_published_state = Some(st);
                        }
                    }
                }
            }

            ev = eventloop.poll() => {
                match ev {
                    Ok(Event::Incoming(Incoming::Publish(p))) => {
                        if p.topic == topics.command_topic {
                            handle_command(
                                &client,
                                &cfg,
                                &topics,
                                &hid_tx,
                                &mut saved_on,
                                &mut kb_online,
                                p,
                            ).await;
                        } else if p.topic == topics.ha_status_topic {
                            handle_ha_status(
                                &client,
                                &cfg,
                                &topics,
                                &mut kb_online,
                                &hid_tx,
                                &mut saved_on,
                                p
                            ).await;
                        }
                    }
                    Ok(Event::Outgoing(Outgoing::Disconnect)) => {
                        anyhow::bail!("mqtt disconnected");
                    }
                    Ok(_) => {}
                    Err(e) => anyhow::bail!("mqtt poll error: {e}"),
                }
            }
        }
    }
}

// ── HA birth message handler ─────────────────────────────────────────────────

async fn handle_ha_status(
    client: &AsyncClient,
    cfg: &AppConfig,
    topics: &Topics,
    kb_online: &mut bool,
    hid_tx: &mpsc::Sender<HidJob>,
    saved_on: &mut SavedOn,
    p: Publish,
) {
    let payload = match std::str::from_utf8(&p.payload) {
        Ok(s) => s.trim(),
        Err(_) => return,
    };

    // HA typically publishes "online"/"offline" on homeassistant/status.
    if payload == "online" && cfg.ha.republish_on_ha_birth {
        tokio::time::sleep(Duration::from_millis(250)).await;

        // Re-read supported keyboard effects on HA restart so the discovery
        // payload stays aligned with the currently connected keyboard/firmware.
        let effect_catalog = if let Ok(Some(ids)) = hid_get_supported_effects(hid_tx).await {
            Some(build_effect_catalog(ids))
        } else {
            None
        };

        if let Err(e) = publish_discovery(client, cfg, topics, effect_catalog.as_ref()).await {
            warn!("republish discovery error: {e:#}");
        }

        // Re-probe keyboard and republish current availability.
        let online_now = hid_probe(hid_tx).await.unwrap_or(false);
        if online_now != *kb_online {
            *kb_online = online_now;
        }
        let _ = publish_availability(client, topics, *kb_online).await;

        // Also republish current real state when online (helps after HA restart).
        if *kb_online {
            if let Ok(Some(st)) = hid_get_state(hid_tx).await {
                update_saved_on_from_state(saved_on, &st);
                let _ = publish_state(client, cfg, topics, &st).await;
            }
        }
    }
}

// ── Command handler ──────────────────────────────────────────────────────────

struct SavedOn {
    brightness: u8,
    color: HaRgb,
    effect: Option<String>,
}

fn update_saved_on_from_state(saved_on: &mut SavedOn, st: &HaLightState) {
    if st.state == "ON" {
        if let Some(b) = st.brightness {
            saved_on.brightness = b;
        }
        if let Some(c) = st.color.clone() {
            saved_on.color = c;
        }
        if let Some(eff) = st.effect.clone() {
            saved_on.effect = Some(eff);
        }
    }
}

async fn handle_command(
    client: &AsyncClient,
    cfg: &AppConfig,
    topics: &Topics,
    hid_tx: &mpsc::Sender<HidJob>,
    saved_on: &mut SavedOn,
    kb_online: &mut bool,
    p: Publish,
) {
    let payload = match std::str::from_utf8(&p.payload) {
        Ok(s) => s,
        Err(e) => {
            warn!("invalid UTF-8 payload: {e}");
            return;
        }
    };

    let mut cmd: HaLightCommand = match serde_json::from_str(payload) {
        Ok(c) => c,
        Err(e) => {
            warn!("invalid JSON payload: {e}; payload={payload}");
            return;
        }
    };

    // OFF is authoritative; ignore other fields.
    if matches!(cmd.state.as_deref(), Some("OFF" | "off")) {
        match hid_apply(hid_tx, cmd).await {
            Ok(true) => {
                if !*kb_online {
                    *kb_online = true;
                    let _ = publish_availability(client, topics, true).await;
                }
                let _ = publish_state(client, cfg, topics, &HaLightState::off()).await;
            }
            Ok(false) => {
                if *kb_online {
                    *kb_online = false;
                    let _ = publish_availability(client, topics, false).await;
                }
                info!("keyboard not available (no_device) — ignoring OFF command");
            }
            Err(e) => warn!("hid apply OFF failed: {e:#}"),
        }
        return;
    }

    // If HA sends "ON" without brightness/color, restore the last known "ON" settings.
    if matches!(cmd.state.as_deref(), Some("ON" | "on")) || cmd.state.is_none() {
        if cmd.brightness.is_none() {
            cmd.brightness = Some(saved_on.brightness);
        }
        if cmd.color.is_none() {
            cmd.color = Some(saved_on.color.clone());
        }
        if cmd.effect.is_none() {
            cmd.effect = saved_on.effect.clone();
        }
    }

    // Apply to keyboard.
    match hid_apply(hid_tx, cmd.clone()).await {
        Ok(true) => {
            if !*kb_online {
                *kb_online = true;
                let _ = publish_availability(client, topics, true).await;
            }
        }
        Ok(false) => {
            if *kb_online {
                *kb_online = false;
                let _ = publish_availability(client, topics, false).await;
            }
            info!("keyboard not available (no_device) — ignoring ON command");
            return;
        }
        Err(e) => {
            warn!("hid apply failed: {e:#}");
            return;
        }
    }

    // Update saved "ON" settings.
    if let Some(b) = cmd.brightness {
        saved_on.brightness = b;
    }
    if let Some(c) = cmd.color.clone() {
        saved_on.color = c;
    }
    if let Some(eff) = cmd.effect.clone() {
        saved_on.effect = Some(eff);
    }

    // Publish optimistic state (echo what we applied).
    let st = HaLightState {
        state: "ON".to_owned(),
        brightness: cmd.brightness,
        color_mode: Some("rgb".to_owned()),
        color: cmd.color,
        effect: cmd.effect,
    };

    let _ = publish_state(client, cfg, topics, &st).await;
}

// ── Publish helpers ──────────────────────────────────────────────────────────

async fn publish_discovery(client: &AsyncClient, cfg: &AppConfig, topics: &Topics,
                           effects: Option<&EffectCatalog>,
) -> Result<()> {
    let payload = build_discovery_payload(cfg, topics, effects)?;
    client
        .publish(
            topics.discovery_topic.clone(),
            QoS::AtLeastOnce,
            cfg.mqtt.retain_discovery,
            payload,
        )
        .await
        .context("mqtt publish discovery")?;

    info!("published discovery: {}", topics.discovery_topic);
    Ok(())
}

async fn publish_availability(client: &AsyncClient, topics: &Topics, online: bool) -> Result<()> {
    let payload = if online { "online" } else { "offline" };
    client
        .publish(
            topics.availability_topic.clone(),
            QoS::AtLeastOnce,
            true,
            payload,
        )
        .await
        .context("mqtt publish availability")?;
    Ok(())
}

async fn publish_state(
    client: &AsyncClient,
    cfg: &AppConfig,
    topics: &Topics,
    st: &HaLightState,
) -> Result<()> {
    let payload = serde_json::to_string(st).context("state_to_json")?;
    client
        .publish(
            topics.state_topic.clone(),
            QoS::AtLeastOnce,
            cfg.mqtt.retain_state,
            payload,
        )
        .await
        .context("mqtt publish state")?;
    Ok(())
}

fn build_discovery_payload(cfg: &AppConfig, topics: &Topics, effects: Option<&EffectCatalog>) -> Result<String> {
    #[derive(Serialize)]
    struct Device<'a> {
        identifiers: [&'a str; 1],
        name: &'a str,
        manufacturer: &'a str,
        model: &'a str,
        sw_version: &'a str,
    }

    #[derive(Serialize)]
    struct Origin<'a> {
        name: &'a str,
        sw_version: &'a str,
    }

    #[derive(Serialize)]
    struct Discovery<'a> {
        name: &'a str,
        unique_id: &'a str,

        // Use MQTT Light JSON schema.
        schema: &'a str,

        command_topic: &'a str,
        state_topic: &'a str,

        availability_topic: &'a str,
        payload_available: &'a str,
        payload_not_available: &'a str,

        brightness: bool,
        supported_color_modes: [&'a str; 1],

        #[serde(skip_serializing_if = "Option::is_none")]
        effect: Option<bool>,

        #[serde(skip_serializing_if = "Option::is_none")]
        effect_list: Option<Vec<String>>,

        device: Device<'a>,
        origin: Origin<'a>,
    }

    // Publish effect support only when the connected keyboard/firmware reports
    // at least one HA-exposed effect.
    let effect_enabled = effects
        .map(|e| !e.ha_names.is_empty())
        .unwrap_or(false);

    let effect_list = effects.and_then(|e| {
        if e.ha_names.is_empty() {
            None
        } else {
            Some(e.ha_names.clone())
        }
    });

    let d = Discovery {
        name: &cfg.ha.name,
        unique_id: &cfg.ha.unique_id,
        schema: "json",
        command_topic: &topics.command_topic,
        state_topic: &topics.state_topic,
        availability_topic: &topics.availability_topic,
        payload_available: "online",
        payload_not_available: "offline",
        brightness: true,
        supported_color_modes: ["rgb"],
        device: Device {
            identifiers: [&cfg.ha.unique_id],
            name: &cfg.ha.name,
            manufacturer: "QMK/Vial",
            model: "VialRGB keyboard",
            sw_version: env!("CARGO_PKG_VERSION"),
        },
        origin: Origin {
            name: "MARCOntroller",
            sw_version: env!("CARGO_PKG_VERSION"),
        },
        effect: effect_enabled.then_some(true),
        effect_list,
    };

    serde_json::to_string(&d).context("discovery_to_json")
}

// ── HID worker thread ────────────────────────────────────────────────────────
//
// Replies are typed via HidReply to support Probe/Apply/GetState on one channel.
// ---------------------------------------------------------------------------

enum HidRequest {
    Probe,
    GetState,
    GetSupportedEffects,
    Apply(HaLightCommand),
}

enum HidReply {
    Availability(bool),
    State(Option<HaLightState>),
    SupportedEffects(Option<Vec<u16>>),
}

struct HidJob {
    req: HidRequest,
    reply: oneshot::Sender<Result<HidReply>>,
}

async fn hid_probe(tx: &mpsc::Sender<HidJob>) -> Result<bool> {
    let (reply_tx, reply_rx) = oneshot::channel();
    tx.send(HidJob {
        req: HidRequest::Probe,
        reply: reply_tx,
    })
        .await?;

    match reply_rx.await?? {
        HidReply::Availability(v) => Ok(v),
        _ => Err(anyhow!("unexpected_hid_reply")),
    }
}

async fn hid_get_state(tx: &mpsc::Sender<HidJob>) -> Result<Option<HaLightState>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    tx.send(HidJob {
        req: HidRequest::GetState,
        reply: reply_tx,
    })
        .await?;

    match reply_rx.await?? {
        HidReply::State(st) => Ok(st),
        _ => Err(anyhow!("unexpected_hid_reply")),
    }
}

async fn hid_get_supported_effects(tx: &mpsc::Sender<HidJob>) -> Result<Option<Vec<u16>>> {
    let (reply_tx, reply_rx) = oneshot::channel();
    tx.send(HidJob {
        req: HidRequest::GetSupportedEffects,
        reply: reply_tx,
    })
        .await?;

    match reply_rx.await?? {
        HidReply::SupportedEffects(v) => Ok(v),
        _ => Err(anyhow!("unexpected_hid_reply")),
    }
}

async fn hid_apply(tx: &mpsc::Sender<HidJob>, cmd: HaLightCommand) -> Result<bool> {
    let (reply_tx, reply_rx) = oneshot::channel();
    tx.send(HidJob {
        req: HidRequest::Apply(cmd),
        reply: reply_tx,
    })
        .await?;

    match reply_rx.await?? {
        HidReply::Availability(v) => Ok(v),
        _ => Err(anyhow!("unexpected_hid_reply")),
    }
}

fn hid_worker(cfg: AppConfig, mut rx: mpsc::Receiver<HidJob>) {
    let mut api = match hidapi::HidApi::new() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("HIDAPI init failed: {e}");
            return;
        }
    };

    let vid = match parse_hex_u16(&cfg.hid.vid) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid VID in config: {e:#}");
            return;
        }
    };
    let pid = match parse_hex_u16(&cfg.hid.pid) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid PID in config: {e:#}");
            return;
        }
    };

    while let Some(job) = rx.blocking_recv() {
        // Refresh device list to handle hot-plug.
        let _ = api.refresh_devices();

        let res: Result<HidReply> = match &job.req {
            HidRequest::Probe => match probe_once(&api, vid, pid, cfg.hid.serial.as_deref()) {
                Ok(()) => Ok(HidReply::Availability(true)),
                Err(e) => {
                    if is_no_device_anyhow(&e) {
                        Ok(HidReply::Availability(false))
                    } else {
                        Err(e)
                    }
                }
            },

            HidRequest::GetState => match get_state_once(&api, vid, pid, cfg.hid.serial.as_deref()) {
                Ok(st) => Ok(HidReply::State(Some(st))),
                Err(e) => {
                    if is_no_device_anyhow(&e) {
                        Ok(HidReply::State(None))
                    } else {
                        Err(e)
                    }
                }
            },

            HidRequest::GetSupportedEffects => {
                match get_supported_effects_once(&api, vid, pid, cfg.hid.serial.as_deref()) {
                    Ok(ids) => Ok(HidReply::SupportedEffects(Some(ids))),
                    Err(e) => {
                        if is_no_device_anyhow(&e) {
                            Ok(HidReply::SupportedEffects(None))
                        } else {
                            Err(e)
                        }
                    }
                }
            },

            HidRequest::Apply(cmd) => match apply_once(&api, vid, pid, cfg.hid.serial.as_deref(), cmd) {
                Ok(()) => Ok(HidReply::Availability(true)),
                Err(e) => {
                    if is_no_device_anyhow(&e) {
                        Ok(HidReply::Availability(false))
                    } else {
                        Err(e)
                    }
                }
            },
        };

        let _ = job.reply.send(res);
    }
}

fn probe_once(api: &hidapi::HidApi, vid: u16, pid: u16, serial: Option<&str>) -> Result<()> {
    // Keyboard is "online" if we can open it and read something simple.
    let dev = hid::open_device(api, vid, pid, serial).context("open_device")?;
    let _ = vialrgb::get_info(&dev).context("get_info")?;
    Ok(())
}

fn get_state_once(api: &hidapi::HidApi, vid: u16, pid: u16, serial: Option<&str>) -> Result<HaLightState> {
    let dev = hid::open_device(api, vid, pid, serial).context("open_device")?;
    let m = vialrgb::get_mode(&dev).context("get_mode")?;

    // Consider OFF if effect is OFF or brightness is 0.
    if m.mode == vialrgb::EFFECT_OFF || m.v == 0 {
        return Ok(HaLightState::off());
    }

    let rgb = hsv_to_rgb(m.h, m.s, m.v);

    // Reconstruct the current HA effect name from the real keyboard mode so
    // Home Assistant stays in sync with out-of-band changes (UI, Vial, HID tools).
    let effect = match m.mode {
        vialrgb::EFFECT_OFF => None,
        vialrgb::EFFECT_DIRECT => None,
        other => Some(ha_effect_name_for_id(other)),
    };

    Ok(HaLightState {
        state: "ON".to_owned(),
        brightness: Some(m.v),
        color_mode: Some("rgb".to_owned()),
        color: Some(rgb),
        effect,
    })
}

fn get_supported_effects_once(
    api: &hidapi::HidApi,
    vid: u16,
    pid: u16,
    serial: Option<&str>,
) -> Result<Vec<u16>> {
    let dev = hid::open_device(api, vid, pid, serial).context("open_device")?;
    vialrgb::get_supported_effects(&dev).context("get_supported_effects")
}

fn apply_once(
    api: &hidapi::HidApi,
    vid: u16,
    pid: u16,
    serial: Option<&str>,
    cmd: &HaLightCommand,
) -> Result<()> {
    let dev = hid::open_device(api, vid, pid, serial).context("open_device")?;

    // OFF
    if matches!(cmd.state.as_deref(), Some("OFF" | "off")) {
        vialrgb::set_mode(&dev, vialrgb::EFFECT_OFF, 0, 0, 0, 0)?;
        return Ok(());
    }

    // ON / update
    // ON / update
    //
    // If HA provides an effect name, translate it to a VialRGB mode ID.
    // Otherwise keep the current behaviour: default to SOLID_COLOR unless we
    // explicitly decide otherwise in a later step.
    let h: u8;
    let s: u8;
    let mut v: u8;

    if let Some(rgb) = &cmd.color {
        let (hh, ss, vv) = vialrgb::rgb_to_hsv(rgb.r, rgb.g, rgb.b);
        h = hh;
        s = ss;
        v = vv;
    } else {
        let cur = vialrgb::get_mode(&dev)?;
        h = cur.h;
        s = cur.s;
        v = cur.v;
    }

    if let Some(b) = cmd.brightness {
        v = b;
    }

    let mode = if let Some(effect_name) = cmd.effect.as_deref() {
        effect_id_for_ha_name(effect_name).unwrap_or(vialrgb::EFFECT_SOLID_COLOR)
    } else {
        vialrgb::EFFECT_SOLID_COLOR
    };

    vialrgb::set_mode(&dev, mode, 0, h, s, v)?;
    Ok(())
}

/// Convert HSV (0–255) to RGB (0–255) using integer math.
/// QMK uses 0–255 ranges for HSV.
fn hsv_to_rgb(h: u8, s: u8, v: u8) -> HaRgb {
    if s == 0 {
        return HaRgb { r: v, g: v, b: v };
    }

    // This is a common 8-bit HSV->RGB conversion (region/remainder).
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

    HaRgb {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    }
}

/// Detect the common "device not present" error.
/// We treat this as a normal offline condition (not a hard error).
fn is_no_device_anyhow(e: &anyhow::Error) -> bool {
    let msg = format!("{e:#}");
    msg.contains("no_device")
}

/// Parse a hex string (with or without "0x" prefix) into a `u16`.
fn parse_hex_u16(s: &str) -> Result<u16> {
    let clean = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(clean, 16).context("parse_hex_u16")
}