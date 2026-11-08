# 🎮 XJemulator v1.0.0 — Release Notes

---

## 🇪🇸 Notas de la Versión (Español)

### ✨ Novedades Principales

- **Soporte para Sensores Magnéticos (Hall Effect) y Ejes Continuos**:
  - Nuevo motor de calibración progresiva ([`detect_axis_movement`](src/gui/backend.rs)) que mide el recorrido completo desde el punto de reposo hasta el tope físico (`100%`) tanto en gatillos analógicos como en joysticks.
  - **Zona Muerta Re-escalada Continua (`Continuous Scaled Deadzone`)** en [`Mapper::map_axis`](src/mapper.rs): elimina el salto brusco inicial permitiendo controlar la velocidad del personaje de forma gradual (caminar lento, trotar, correr) y aceleración lineal `0..255` en gatillos.
- **Compatibilidad Híbrida con Mandos Antiguos / Digitales (`0` y `1`)**:
  - Soporte nativo para mandos clásicos cuyos gatillos (`LT`/`RT`) o cruceta (`D-Pad`) funcionan como botones digitales (`EV_KEY`) mediante la nueva tabla `[[button_axes]]` ([`ButtonAxisMapping`](src/mapper.rs)).
  - Posibilidad de **omitir ejes** (`⏭ Omitir`) durante el Paso 2 de calibración para mandos que no disponen de segundo stick o gatillos analógicos.
- **Empaquetado Portable `.AppImage` e Instalador Simplificado de `udev`**:
  - Nuevo generador portable [`scripts/build-appimage.sh`](scripts/build-appimage.sh).
  - Nuevo script dedicado [`scripts/install-udev.sh`](scripts/install-udev.sh) para configurar permisos de kernel (`uinput` + `udev`) vía `curl` en un solo paso.

### ⚡ Rendimiento y Reducción de Latencia (`500Hz`)

- **Cero bloqueo de UI al iniciar la emulación**: el tiempo de asentamiento para detección en SDL2/Proton (`1000 ms`) ahora se ejecuta de forma asíncrona en el hilo secundario ([`VirtualXbox360::wait_for_sdl2_detection`](src/virtual_device.rs)).
- **Bucle de emulación `500Hz` (`2 ms`) optimizado**: escritura por lotes (`write_batch`) fuera del `Mutex` de interfaz, cero asignaciones en el *heap* durante el bucle caliente y eliminación de eventos `SYN_REPORT` redundantes.
- **Lector de calibración reactivo a `500Hz`**: despierta inmediatamente el renderizador de `egui` (`request_repaint`) al detectar pulsaciones o movimientos del mando.

### 🎨 Interfaz y Experiencia de Usuario (`egui`)

- **Esquema del mando rediseñado** ([`draw_xbox_controller`](src/gui/widgets/controller_view.rs)): mayor separación visual (`> 15–28 px`) entre gatillos, bumpers, sticks, cruceta y botones `ABXY`.
- **Corrección visual en transición de `R3` a ejes**: la barra verde de progreso de movimiento solo aparece cuando el mando está estabilizado en reposo dentro de la fase de ejes y el usuario mueve activamente un stick o gatillo.
- **Selector de nombre de perfil**: añadido campo de texto al finalizar la calibración (autocompletado con el nombre del mando) y persistencia automática del idioma (`ES` / `EN`) en `config.toml`.

### 🔒 Seguridad y Estabilidad

- **Reglas `udev` endurecidas** ([`udev/99-xjemulator.rules`](udev/99-xjemulator.rules)): reemplazado `MODE="0666"` por `MODE="0660", GROUP="input", TAG+="uaccess"`.
- **Prevención de bucles de auto-detección**: [`scan_gamepads`](src/scanner.rs) filtra los dispositivos virtuales `"Microsoft X-Box 360 pad"` propios de `uinput`.
- **Sincronización determinista de hilos (`EVIOCGRAB`)**: liberación garantizada (`ungrab`) al cambiar entre calibración y emulación o al cerrar la aplicación.

---

## 🌍 Release Notes (English)

### ✨ Highlights

- **Hall-Effect (Magnetic) & Continuous Analog Axis Support**:
  - Redesigned progressive calibration engine ([`detect_axis_movement`](src/gui/backend.rs)) that tracks the full physical travel from neutral rest to the physical limit (`100%`) on both analog triggers and thumbsticks.
  - **Continuous Scaled Deadzone** in [`Mapper::map_axis`](src/mapper.rs): eliminates deadzone step-jumps, preserving smooth velocity gradients (slow walk, jog, full sprint) and linear `0..255` trigger response.
- **Hybrid Support for Legacy / Digital Controllers (`0` and `1`)**:
  - Seamless support for retro/generic gamepads whose triggers (`LT`/`RT`) or `D-Pad` emit digital button events (`EV_KEY`) via `[[button_axes]]` ([`ButtonAxisMapping`](src/mapper.rs)).
  - Enabled **Axis Skipping** (`⏭ Skip`) during Step 2 for controllers without a right stick or analog triggers.
- **Portable `.AppImage` Builder & Simplified `udev` Installer**:
  - Added [`scripts/build-appimage.sh`](scripts/build-appimage.sh) to package a standalone `.AppImage`.
  - Added [`scripts/install-udev.sh`](scripts/install-udev.sh) for one-line `udev` and `uinput` setup via `curl`.

### ⚡ Performance & Input Latency (`500Hz`)

- **Non-blocking Emulator Startup**: moved the `1000 ms` SDL2/Proton settle delay off the main UI thread into the worker thread ([`VirtualXbox360::wait_for_sdl2_detection`](src/virtual_device.rs)).
- **Zero-Allocation `500Hz` (`2 ms`) Emulation Loop**: pre-allocated event batching outside the UI `Mutex` and atomic `SYN_REPORT` emission per burst.
- **Reactive `500Hz` Calibration Reader**: triggers immediate `egui` repaints upon hardware input arrival.

### 🎨 UI & UX Refinements (`egui`)

- **Spaced Controller Layout** ([`draw_xbox_controller`](src/gui/widgets/controller_view.rs)): expanded spacing (`> 15–28 px`) across triggers, bumpers, sticks, D-Pad, and `ABXY` buttons.
- **Clean `R3` to Axes Transition**: the green axis progress bar now renders strictly during active axis deflection after full stick stabilization.
- **Profile Naming & Persistent Language**: added profile name input on the final calibration review screen and persisted `ES`/`EN` selection in `config.toml`.

### 🔒 Security & Reliability

- **Hardened `udev` Rules** ([`udev/99-xjemulator.rules`](udev/99-xjemulator.rules)): scoped `/dev/uinput` permissions to `MODE="0660", GROUP="input", TAG+="uaccess"`.
- **Virtual Device Loop Prevention**: [`scan_gamepads`](src/scanner.rs) ignores virtual `"Microsoft X-Box 360 pad"` nodes created by `XJemulator`.
- **Deterministic `EVIOCGRAB` Lifecycle**: synchronous thread join and `ungrab()` on reader/emulator transitions and application exit.
