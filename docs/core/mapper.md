# Mapeo y Configuración de Perfiles

XJemulator funciona traduciendo estructuras de evdev de un lado hacia uinput usando el `src/mapper.rs`.

## Persistencia
El sistema emplea TOML y Serde para una recolección nativa rápida. Los perfiles generados por el asistente de calibración se asientan permanentemente en `$HOME/.config/xjemulator/profiles/` de la computadora local. Adicionalmente, el estado general de configuración o idiomas viaja guardado por `AppConfig`.

## Event Mapping Rule (Bloque Transaccional)
Todo evento original USB consta de tres unidades en `evdev::InputEvent`: Identificador de Evento (Tipo Absoluto, Relativo o Key), Target Axis/Button y la Magnitude (1 = pulsado, 0 = soltado, etc).
El Mapper lo atrapa e interroga contra el diccionario del archivo `.toml` actualmente en carga activa de la interfaz. Si coincide con una regla designada durante el calibrado, el mapper despacha este evento re-etiquetado.
Si adicionalmente se especificó una regla de inversión en `invert = true` para un Axis en específico durante la calibración, el valor se invierte matemáticamente en vivo antes de enviarse.
