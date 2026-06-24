# BFS9000 Hardware

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

This directory contains the hardware files for **BFS9000**, a custom split keyboard designed by Jesús Guillén.

The current stable hardware revision is **0.4**.

---

## 1) Directory structure

```text
hardware/
├── BFS9000/              KiCad source files for the main PCB
├── production/           Manufacturing-ready files
│   └── rev0.4/
│       ├── pcb/          Main PCB Gerbers and drill files
│       └── plates/       Optional switch and bottom plate files
└── enclosures/           Case, enclosure and laser-cutting files
    └── rev0.4/
```

The `BFS9000/` directory contains the editable KiCad project files.

The `production/` directory contains files intended to be sent directly to a PCB manufacturer.

The `enclosures/` directory contains optional case/enclosure files and CNC/laser-cutting files.

---

## 2) Hardware revisions

### Revision 0.2

Revision **0.2** used a **four-layer PCB**.

This revision had a fatal hardware issue caused by an incorrect LED footprint. It is kept only as historical reference and should not be used for new boards.

### Revision 0.3

Revision **0.3** corrected the critical issues found in revision 0.2.

It also changed the PCB from **four layers to two layers**, making the design simpler and easier to manufacture.

### Revision 0.4

Revision **0.4** is the current stable hardware revision.

It keeps the fixes from revision 0.3 and generally optimises the PCB layout.

---

## 3) Production files

Manufacturing-ready files for revision 0.4 are stored in:

```text
hardware/production/rev0.4/
```

### Main PCB

```text
hardware/production/rev0.4/pcb/
└── BFS9000-rev0.4-pcb-gerbers-drill.zip
```

This archive contains the Gerber and drill files for the main BFS9000 PCB.

The following preview images are also included:

```text
BFS9000-rev0.4-gerber-preview.png
BFS9000-rev0.4-drill-preview.png
```

### Optional plates

```text
hardware/production/rev0.4/plates/
├── BFS9000-rev0.4-switch-plate.zip
└── BFS9000-rev0.4-bottom-plate.zip
```

The **BFS9000 switch plate** and **BFS9000 bottom plate** are optional.

They are normally manufactured from **PMMA**.

---

## 4) Enclosures and CNC files

Case and enclosure files for revision 0.4 are stored in:

```text
hardware/enclosures/rev0.4/
```

### A series — sectionable enclosure

Files starting with `A_` belong to the **sectionable** enclosure version.

Use:

```text
A_BFS9000_sectionable_switch.3mf
```

Then choose only one bottom part:

```text
A_BFS9000_sectionable_bottom.3mf
A_BFS9000_sectionable_bottom_tps65.3mf
```

Use the `*_tps65` version if the keyboard build includes the **TPS65 trackpad**.

### B series — full enclosure

Files starting with `B_` belong to the **full**, non-sectionable enclosure version.

This version is intended for the complete keyboard:

```text
B_BFS9000_full_switch.3mf
B_BFS9000_full_bottom.3mf
B_BFS9000_full_bottom_tps65.3mf
```

Use the `*_tps65` bottom if the keyboard build includes the **TPS65 trackpad**.

### C series — full enclosure with window

Files starting with `C_` belong to the **full**, non-sectionable enclosure version with a window.

The window is intended to expose the PCB decoration area, such as the logo, QR code, dedication or other artwork:

```text
C_BFS9000_full_with_window_switch.3mf
C_BFS9000_full_with_window_bottom.3mf
C_BFS9000_full_bottom_with_window_tps65.3mf
```

Use the `*_tps65` bottom if the keyboard build includes the **TPS65 trackpad**.

### X series — TPS65 enclosure parts

Files starting with `X_` are specific to the **TPS65 trackpad** enclosure:

```text
X_tps65_bottom.3mf
X_tps65_frame.3mf
X_tps65_main_bottom_addon.3mf
```

### CNC / laser-cutting files

Files starting with `CNC_` are intended for CNC or laser cutting, for example in **PMMA**:

```text
CNC_bottom.dxf
CNC_switch.dxf
CNC_screen.dxf
```

---

## 5) Stable hardware tag

The stable hardware revision 0.4 is intended to be marked with this Git tag:

```text
hardware/bfs9000-rev0.4
```

Git tags always point to a full repository commit, not to an individual folder.
However, the hardware folder can be restored independently from that tag.

To restore only the hardware directory to revision 0.4:

```bash
git restore --source hardware/bfs9000-rev0.4 -- hardware
```

To compare the current hardware files against revision 0.4:

```bash
git diff hardware/bfs9000-rev0.4 -- hardware
```

---

## 6) Recommended workflow

With a clean repository:

```bash
git status --short
```

Create an annotated tag:

```bash
git tag -a hardware/bfs9000-rev0.4 -m "BFS9000 hardware revision 0.4 stable"
```

Push the tag to GitHub:

```bash
git push origin hardware/bfs9000-rev0.4
```

---

## 7) Licence

Unless stated otherwise, the files in this directory are distributed under the repository licence.

See the root [LICENSE](../LICENSE) file for details.
