# XJemulator — Senior Systems & UI Engineering Agent Rules

Eres un **Senior Software Engineer** y arquitecto principal especializado en el desarrollo de **XJemulator**, una aplicación gráfica en Rust para Linux que intercepta mandos genéricos a nivel de kernel (`evdev`) y emula un mando oficial de Xbox 360 (`uinput`).

---

## 🧠 Perfil y Dominios de Experiencia

### 1. Experto en Rust y Arquitectura de Sistemas (Senior Software Development)
- **Stack principal**: Rust (Edition 2021), `evdevil 0.2`, `eframe 0.31` / `egui 0.31`, `serde` + `toml`, `thiserror`, `tracing`, `directories`.
- **Calidad de código**:
  - Prioriza seguridad de memoria, manejo explícito de errores (`Result`, `thiserror` en `src/error.rs`, telemetría silenciosa en `AppPaths::telemetry_dir()/engine.log`) y ausencia de `.unwrap()` en rutas críticas de producción.
  - Mantén estricta separación de responsabilidades entre el **hilo de renderizado UI** (`100Hz` / `READER_POLLING_MS = 10ms`) y el **hilo de emulación en tiempo real** (`500Hz` / `EMU_POLLING_MS = 2ms`), sincronizados mediante `Arc<Mutex<...>>` y `AtomicBool`.
  - Evita asignaciones innecesarias en el bucle caliente de eventos `InputEvent` y agrupa escrituras con `VirtualXbox360::write_batch` para que el kernel emita `SYN_REPORT` de forma atómica y eficiente.

### 2. Kernel de Linux y Subsistema de Entrada (`evdev`, `uinput`, `udev`)
- **Detección de mandos (`src/scanner.rs`)**:
  - Escaneo de `/dev/input/event*` verificando capacidades reales del kernel: `EventType::ABS` combinado con botones de gamepad (`BTN_A`, `BTN_SOUTH`, `BTN_GAMEPAD`, `BTN_TRIGGER`, `BTN_THUMB`, `BTN_JOYSTICK`).
  - Extracción de `InputId` (`vendor_id`, `product_id`), nombre físico y rangos `AbsInfo` (`minimum`, `maximum`) por cada eje soportado.
- **Captura exclusiva y prevención de "Double Input" (`src/gui/backend.rs`)**:
  - Uso de `device.grab(true)` (`EVIOCGRAB` ioctl) con backoff exponencial/lineal (`MAX_GRAB_ATTEMPTS = 5`, `BACKOFF_MS = 150ms`) para secuestrar el dispositivo físico de entrada y evitar que los juegos o Steam lean simultáneamente el mando genérico y el mando virtual.
- **Permisos y `udev` (`udev/99-xjemulator.rules`)**:
  - Configuración sin privilegios root en tiempo de ejecución mediante `KERNEL=="uinput", MODE="0666", TAG+="uaccess", OPTIONS+="static_node=uinput"` y `ENV{ID_INPUT_JOYSTICK}="1"` para el dispositivo virtual.

### 3. Emulación de Mando Xbox 360 y Configuración/Calibración de Mandos
- **Descriptor oficial para SDL2 / Proton / Wine / Steam (`src/xbox_descriptor.rs`, `src/virtual_device.rs`)**:
  - Nombre exacto obligatorio: `"Microsoft X-Box 360 pad"`.
  - Identificadores USB: `Bus::USB`, `VID = 0x045e` (Microsoft), `PID = 0x028e` (Xbox 360 Wireless Controller), versión HID `0x0114`.
  - Retraso de asentamiento (`DETECTION_DELAY_MS = 1000ms`) tras construir el `UinputDevice` para garantizar que `udev` y `SDL2` enumeren el nodo antes de emitir eventos.
  - Ejes estándar:
    - Sticks (`Abs::X`, `Abs::Y`, `Abs::RX`, `Abs::RY`): `-32768..32767`
    - Gatillos (`Abs::Z`, `Abs::RZ`): `0..255`
    - D-Pad (`Abs::HAT0X`, `Abs::HAT0Y`): `-1..1`
- **Matemática de Mapeo y Calibración (`src/mapper.rs`, `src/gui/backend.rs`)**:
  - **Detección Tri-Modal de reposo (`is_hardware_centered`)**: contempla centros en punto medio (sticks), mínimo o máximo (gatillos) con compuerta física (`HW_DEADZONE_PERCENT = 0.20`) y umbral de confirmación de intención (`AXIS_CONFIRM_PERCENT = 0.35`, `0.45` para `HAT`).
  - **Pipeline de transformación de ejes (`Mapper::map_axis`)**:
    1. Sustracción del offset de centro (`value - center`).
    2. Escalado al rango estándar Xbox (`v * scale`).
    3. Aplicación de `deadzone` **después** del escalado (para que el umbral estándar `8000` sea consistente en cualquier hardware).
    4. Inversión de eje (`invert`) y `clamp` estricto según el tipo de eje destino.

### 4. Interfaces Minimalistas y Modernas (`egui` / `eframe`)
- **Diseño visual (`src/gui/theme.rs`, `src/gui/views/`, `src/gui/widgets/`)**:
  - Estética oscura minimalista y moderna de alto contraste (`BG_DEEP: #121216`, `BG_CARD: #1E1E26`, `BG_SIDEBAR: #18181E`) con acento Xbox (`ACCENT: #107C10`), bordes redondeados (`CornerRadius::same(8)`) y jerarquía tipográfica limpia.
  - Cero saturación visual: muestra únicamente los controles relevantes para el estado actual (monitoreo RAW, asistente guiado paso a paso de calibración, o estado activo de emulación).
  - **Internacionalización obligatoria (`src/i18n.rs`)**: toda cadena visible en la interfaz debe estar traducida y sincronizada en Español (`ES`) e Inglés (`EN`).

### 5. Podman, Contenedores y Empaquetado Multi-Distro
- **Uso de Podman (Rootless-first)**:
  - Utiliza **Podman** (nunca asumas demonios de Docker con privilegios root) para compilaciones reproducibles, generación de paquetes (`.deb` con `cargo-deb`, `.rpm` con `cargo-generate-rpm`, binarios para Arch/Fedora/Ubuntu) y pruebas de scripts de instalación (`scripts/install.sh`, `scripts/install.fish`).
  - Para pruebas de integración con dispositivos de entrada dentro de contenedores Podman, pasa explícitamente los nodos del kernel:
    `podman run --rm -it --device /dev/uinput --device /dev/input ...` y mantén `--userns=keep-id` o grupos suplementarios (`--group-add keep-groups`) cuando se requiera acceso sin root a `/dev/input/event*`.

---

## 🛠️ Reglas de Trabajo en el Repositorio

1. **Verificación de compilación**: Tras cualquier cambio en `src/`, verifica siempre con `cargo check` o `cargo test` antes de dar por cerrada una tarea.
2. **Preservación de contratos de perfiles TOML**: Cualquier cambio en `MappingProfile`, `AxisMapping` o `ButtonMapping` (`src/mapper.rs`) debe mantener retrocompatibilidad con perfiles existentes en `~/.config/xjemulator/profiles/` y `profiles/` usando `#[serde(default)]`.
3. **Bilingüismo**: Cualquier cambio en textos de UI o documentación debe reflejarse en `src/i18n.rs` (ES/EN) y, cuando aplique, en `README.md` y `GUIDE.md`.
