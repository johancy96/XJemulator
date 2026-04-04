# Arquitectura del Proyecto (XJemulator v0.1.0)

## Resumen del Proyecto
XJemulator es una aplicación gráfica escrita en Rust mediante `egui` que permite remapear y emular mandos virtuales genéricos en Linux presentándolos como dispositivos "Microsoft X-Box 360 pad".

## Estructura de Directorios

- `src/main.rs`: Punto de entrada, inicializa logs y lanza la aplicación.
- `src/gui/`: Contiene toda la lógica de la interfaz y la máquina de estados.
  - `app.rs`: Estado central `App` y funciones principales de renderizado EGUI.
  - `types.rs`: Definición de slots de calibración (`BtnSlot`, `AxisSlot`) y estado crudo (`RawCapture`).
  - `backend.rs`: Bucles de procesamiento independientes (lector de eventos evdevil, emulación asíncrona, thresholds de calibración).
  - `udev_setup.rs`: Lógica para instalar automáticamente reglas udev desde la GUI usando elevación pkexec.
- `src/i18n.rs`: Motor de traducción para soporte multilingüe.
- `src/virtual_device.rs`: Interacción con kernel usando `uinput` para registrar el mando virtual.
- `src/mapper.rs`: Lógica de transposición, interpolación de curvas y perfiles TOML.
- `dist/`: Scripts de empaquetado para `.deb`, `.rpm`, `AppImage` y dependencias adicionales como iconos o archivos `.desktop`.
- `assets/`: Recursos estáticos (PNG, .desktop) requeridos para los paquetes.
- `docs/`: Documentación técnica de desarrollo orientada a futuros agentes / desarrolladores.

## Ciclos y Multithreading
La aplicación usa multithreading pesado para evitar bloquear la interfaz EGUI.
- **Hilo EGUI**: Renderizado constante independiente de la entrada del usuario.
- **Hilo de Captura Cruda (`raw_reader_loop`)**: Bloqueante en `/dev/input/eventX`, captura al vuelo y deposita variables en una memoria compartida (`Arc<Mutex<RawCapture>>`).
- **Hilo de Emulación (`emulation_loop`)**: Reemplaza el input natural con mapeos convertidos y lo inyecta a uinput.

## Lógica del Backend de Interfaz
El refactor en `v0.1.0` redujo el peso de `app.rs` fragmentando lógicas en `backend.rs` y `types.rs`. Las próximas actualizaciones deberían considerar extraer cada *Panel* de la interfaz (Izquierdo, Derecho, Centro) en su propio archivo de estructura.
