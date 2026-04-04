# Motor de Calibración (Calibration Engine)

## El Problema Inicial
Los ejes de múltiples controles, particularmente de modelos de Switch (Joy-Cons), PS4 y clónicos chinos, padecen de "drift" constante, zonas muertas agresivas e inputs parásitos donde un botón gatilla o envía eventos de eje (`ABS_X` con variaciones menores). Esto genera falsos positivos durante la calibración donde a la aplicación le parece que movimos la palanca `Y` pero realmente solo se introdujo "ruido" microscópico.

## El Sistema Delta-Threshold
La calibración ya **no** opera evaluando un valor de pico estático, sino utilizando *Deltas* a partir del valor de reposo ("Resting Values").

```rust
let delta = current_value - resting_value;
let threshold = calib_delta_threshold(axis_name);

if delta.abs() >= threshold && delta.abs() > best_delta.abs() {
   // eje ganador detectado
}
```

### Tolerancias Asimétricas
Se incorporó en `backend.rs` la lógica `calib_delta_threshold()` que implementa barreras ajustadas a cada tipo de sensor físico:
- **D-PAD (Cruceta)**: Sensibilidad alta (salto de 1).
- **Gatillos Z**: Sensibilidad media (salto de 30); cubre gatillos que operan de `0` a `255` y de `0` a `32767`.
- **Joysticks Normales**: Sensibilidad baja (salto de 40 / ~31% del rango). Inmune al peor nivel de *stick drift* durante reposo, evitando capturas sucias.
  
## Escalado Automático
En lugar de forzar todos los mandos a emitir `32767`, se guardan los parámetros máximos del control y se implementa una escala fraccional en la generación TOML (`generate_profile_toml` en `backend.rs`). Redondea factores liminales (`~0.95` a `1.05`) para corregir desperfectos marginales conservando matemática pura.
