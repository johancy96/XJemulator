# Arquitectura General de XJemulator

## Introducción al Sistema
`XJemulator` tiene como propósito principal interceptar eventos crudos (RAW) de gamepads genéricos en Linux (usando la librería de Linux `evdev`) y simular un dispositivo de juego oficial de Microsoft Xbox 360 mediante controladores en userspace (`uinput`). Esto asegura que capas de compatibilidad como Wine, Proton o Steam Input detecten el controlador y funcionen de inmediato ("out-of-the-box").

## Flujo de Eventos (Event Pipeline)
1. **Detección (`scanner.rs`)**
   El sistema examina `/dev/input/` usando un enumerador udev asíncrono o manual.
2. **Adquisición (`reader.rs`)**
   Mediante bloqueos mutuos de sistema (grab mutuxes), el software adquiere potestad exclusiva o simplemente lectura en tiempo real del periférico de hardware, transformando los binarios base C `linux/input.h` en Rust.
3. **Mapeo y Transformación (`mapper.rs`)**
   Traduce el ID raw o la lectura geométrica según el Perfil configurado (`.toml`) por el usuario a un esquema de botones fijo que Xbox 360 entiende (Ej. el botón RAW 299 a `BTN_A`).
4. **Despacho / Emulación (`virtual_device.rs` y `xbox_descriptor.rs`)**
   Aprovechando `uinput`, el programa levanta un dispositivo HID "falso", inyectando las traducciones del Mapper al núcleo de Linux disfrazado del VID y PID de Microsoft, generando reacciones directas en el OS en sub-milisegundos.

## Modificaciones Posteriores
Recientemente se añadió una instancia gráfica (`eframe/egui`) en `src/gui/app.rs` que permite aglomerar las lógicas nativas CCLI y presentar un **Monitor RAW**, un **Editor de Perfiles Visual**, y **Calibración Guiada**.
