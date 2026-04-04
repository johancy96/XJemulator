# Implementación Gráfica (Egui Frontend)

El desarrollo del panel de UX en XJemulator requirió prescindir de frameworks pesados e invocar `eframe` (egui) retenido con OpenGL. La interfaz existe por tanto alojada en `src/gui/app.rs`.

## `App` Struct
El estado principal recae bajo el estructurado `App`. Maneja cosas sensibles como:
- Un monitor de un sub-hilo (background thread) para la recepción raw en tiempo real de los controles (`Arc<Mutex<RawCapture>>`).
- HashMaps de Emuladores Activos (`HashMap<String, VirtualJoystick>`), habilitando la funcionalidad concurrente de múltiples gamepads conectados emulandose en paralelo como jugadores #1, #2, #3, etc.

## Consideraciones Históricas
Durante el desarrollo temprano enfrentamos el *issue de Collision ID* en paneles laterales y elementos iterables. Se mitigó con robustos aislamientos semánticos forzados mediante `.id_salt("key")` propiciados por Egui para evitar interrupciones de repintado.
