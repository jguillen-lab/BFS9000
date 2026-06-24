# Hardware BFS9000

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

Esta carpeta contiene los ficheros de hardware de **BFS9000**, un teclado split diseñado por Jesús Guillén.

La revisión estable actual de hardware es la **0.4**.

---

## 1) Estructura de carpetas

```text
hardware/
├── BFS9000/              Ficheros fuente KiCad de la PCB principal
├── production/           Ficheros listos para fabricación
│   └── rev0.4/
│       ├── pcb/          Gerbers y taladros de la PCB principal
│       └── plates/       Ficheros de switch plate y bottom plate opcionales
└── enclosures/           Ficheros de carcasa y corte láser/CNC
    └── rev0.4/
```

La carpeta `BFS9000/` contiene el proyecto KiCad editable.

La carpeta `production/` contiene ficheros pensados para enviarse directamente a fabricar.

La carpeta `enclosures/` contiene ficheros opcionales de carcasa y ficheros para corte CNC/láser.

---

## 2) Revisiones de hardware

### Revisión 0.2

La revisión **0.2** usaba una **PCB de cuatro capas**.

Esta revisión tenía un fallo fatal de hardware causado por una huella incorrecta en los LEDs. Se conserva únicamente como referencia histórica y no debería usarse para fabricar nuevas placas.

### Revisión 0.3

La revisión **0.3** corrigió los fallos críticos encontrados en la revisión 0.2.

Además, cambió la PCB de **cuatro capas a dos capas**, simplificando el diseño y facilitando su fabricación.

### Revisión 0.4

La revisión **0.4** es la revisión estable actual de hardware.

Mantiene las correcciones de la revisión 0.3 y optimiza la PCB de forma general.

---

## 3) Ficheros de producción

Los ficheros listos para fabricar de la revisión 0.4 están en:

```text
hardware/production/rev0.4/
```

### PCB principal

```text
hardware/production/rev0.4/pcb/
└── BFS9000-rev0.4-pcb-gerbers-drill.zip
```

Este archivo contiene los Gerbers y taladros de la PCB principal de BFS9000.

También se incluyen estas imágenes de previsualización:

```text
BFS9000-rev0.4-gerber-preview.png
BFS9000-rev0.4-drill-preview.png
```

### Placas opcionales

```text
hardware/production/rev0.4/plates/
├── BFS9000-rev0.4-switch-plate.zip
└── BFS9000-rev0.4-bottom-plate.zip
```

La **BFS9000 switch plate** y la **BFS9000 bottom plate** son opcionales.

Normalmente se fabrican en **PMMA**.

---

## 4) Carcasas y ficheros CNC

Los ficheros de carcasa de la revisión 0.4 están en:

```text
hardware/enclosures/rev0.4/
```

### Serie A — carcasa seccionable

Los ficheros que empiezan por `A_` pertenecen a la versión de carcasa **seccionable**.

Se usa:

```text
A_BFS9000_sectionable_switch.3mf
```

Y después se elige solo una pieza inferior:

```text
A_BFS9000_sectionable_bottom.3mf
A_BFS9000_sectionable_bottom_tps65.3mf
```

Usa la versión `*_tps65` si el montaje del teclado incluye el **trackpad TPS65**.

### Serie B — carcasa completa

Los ficheros que empiezan por `B_` pertenecen a la versión de carcasa **completa**, no seccionable.

Esta versión solo sirve para la versión completa del teclado:

```text
B_BFS9000_full_switch.3mf
B_BFS9000_full_bottom.3mf
B_BFS9000_full_bottom_tps65.3mf
```

Usa la pieza inferior `*_tps65` si el montaje del teclado incluye el **trackpad TPS65**.

### Serie C — carcasa completa con ventana

Los ficheros que empiezan por `C_` pertenecen a la versión de carcasa **completa**, no seccionable, con ventana.

La ventana permite ver la zona decorativa de la PCB, como el logo, el QR, la dedicatoria u otros detalles:

```text
C_BFS9000_full_with_window_switch.3mf
C_BFS9000_full_with_window_bottom.3mf
C_BFS9000_full_bottom_with_window_tps65.3mf
```

Usa la pieza inferior `*_tps65` si el montaje del teclado incluye el **trackpad TPS65**.

### Serie X — piezas para TPS65

Los ficheros que empiezan por `X_` son específicos de la carcasa del **trackpad TPS65**:

```text
X_tps65_bottom.3mf
X_tps65_frame.3mf
X_tps65_main_bottom_addon.3mf
```

### Ficheros CNC / corte láser

Los ficheros que empiezan por `CNC_` están pensados para corte CNC o corte láser, por ejemplo en **PMMA**:

```text
CNC_bottom.dxf
CNC_switch.dxf
CNC_screen.dxf
```

---

## 5) Tag estable de hardware

La revisión estable 0.4 del hardware se marcará con este tag de Git:

```text
hardware/bfs9000-rev0.4
```

Los tags de Git siempre apuntan a un commit completo del repositorio, no a una carpeta individual.
Aun así, se puede restaurar solo la parte de hardware desde ese tag.

Para restaurar únicamente la carpeta de hardware a la revisión 0.4:

```bash
git restore --source hardware/bfs9000-rev0.4 -- hardware
```

Para comparar los ficheros actuales de hardware contra la revisión 0.4:

```bash
git diff hardware/bfs9000-rev0.4 -- hardware
```

---

## 6) Flujo recomendado

Con el repositorio limpio:

```bash
git status --short
```

Crear un tag anotado:

```bash
git tag -a hardware/bfs9000-rev0.4 -m "BFS9000 hardware revision 0.4 stable"
```

Subir el tag a GitHub:

```bash
git push origin hardware/bfs9000-rev0.4
```

---

## 7) Licencia

Salvo que se indique lo contrario, los ficheros de esta carpeta se distribuyen bajo la licencia del repositorio.

Consulta el fichero [LICENSE](../LICENSE) en la raíz del repositorio para más detalles.
