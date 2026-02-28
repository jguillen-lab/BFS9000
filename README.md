
# BFS9000 — Umbrella repository (hardware + firmware + software)

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

Main repository that groups several subprojects around the **BFS9000** keyboard (hardware) and its ecosystem of **QMK/Vial firmware** and **PC software** to control lighting over the network (Home Assistant via MQTT, with a local agent that talks to the keyboard over USB).

## Repository structure

- [`hardware/`](./hardware/) — keyboard/PCB design (BFS9000) and related documentation.
- [`firmware/`](./firmware/) — firmware based on QMK/Vial (configuration, keymaps, etc.).
- [`software/`](./software/) — PC agent/CLI (USB + MQTT + Home Assistant).
- [`hodgepodge/`](./hodgepodge/) — tests, experiments, and auxiliary material.

> Each folder should (or will eventually) have its own README with specific instructions.

## Motivation

Standard keyboards never worked for me: I’d always end up placing them diagonally, and the classic staggered layout eventually took its toll (discomfort and some finger deformation). About ~10 years ago I got into the custom keyboard world and started looking for split, more ergonomic options: first I looked at commercial designs like the ErgoDox, then I tried the Corne, until I finally found the **[Sofle](https://github.com/josefadamcik/SofleKeyboard)**, originally designed and maintained by **Josef Adamčík**.

My first keyboard purchase was from **[Mechboards (UK)](https://mechboards.co.uk/)**. My first build attempt was a disaster, but in my case they were really helpful with support. (Note: their returns/warranty policy may differ today and, as they indicate, it usually doesn’t cover modified/soldered products; check it before buying.)
- Current policy (reference): https://mechboardsuk.reamaze.com/kb/shipping-and-returns/returns-policy

My second Sofle was a **Sofle RGB** from **[KeyHive](https://keyhive.xyz/shop/sofle)**. Within the Sofle ecosystem, the **Sofle RGB** variant (per-key RGB + underglow) is commonly attributed as a contribution by **Dane Evans** to the Sofle project.

Over time I’ve had other keyboards based on Sofle with small modifications and even had custom PCBs manufactured. The change was real: my fingers improved and my speed went up; in fact, when I go back to a staggered keyboard now I feel incredibly clumsy.

Even so, although I set up layers and learned to use them comfortably, I’m not the kind of person who “lives” in layers. After ~8 years using the Sofle I decided to try a **[BFO9000](https://docs.keeb.io/bfo-9000-build-guide)** (de **Keebio**) in its full configuration. Ergonomically I like it much less, but it solved my day-to-day better.

I thought that missing the Sofle would fade with time… but it didn’t. That’s why I’m in the process of creating a **BFS9000**, based on JellyTitan’s latest iteration called **[Sofle-Pico](https://github.com/JellyTitan/Sofle-Pico)**:

## Subprojects (quick links)

- BFS9000 hardware: [`hardware/`](./hardware/)
- Firmware (QMK/Vial): [`firmware/`](./firmware/)
- Software (PC agent / MQTT / HA): [`software/`](./software/)
- Auxiliary material: [`hodgepodge/`](./hodgepodge/)

## References

### Designs
- Sofle (Josef Adamčík): https://github.com/josefadamcik/SofleKeyboard
- Sofle-Pico (JellyTitan): https://github.com/JellyTitan/Sofle-Pico
- Sofle-Pico docs/site: https://www.soflepico.com/
- BFO-9000 (Keebio): https://docs.keeb.io/bfo-9000-build-guide

### Stores
- Mechboards: https://mechboards.co.uk/
- KeyHive (Sofle RGB): https://keyhive.xyz/shop/sofle

### Firmware / HA
- Vial (manual): https://get.vial.today/manual/
- Home Assistant MQTT: https://www.home-assistant.io/integrations/mqtt/
- Home Assistant MQTT Light: https://www.home-assistant.io/integrations/light.mqtt/

## License

**EN:** This repository uses **multiple licenses**. The applicable license(s) for each component are included in a `LICENSE` file within the **relevant directory**.

![Sofle y BFO9000](media/sofle_n_bfo9000.png)