# Asistente de Calibración
El módulo de detección y calibración reside dentro de la instancia de Egui en `src/gui/app.rs`.

## Problema Original de Lectura Constante
Inicialmente enfrentamos un Bug Crítico (Phantom Issue): Los ejes con estado inerte diferente a 0 (`RESTING != 0`) causaban de inmediato un registro erróneo. Por ejemplo: ejes de acelerómetro, o algunos drivers baratos que tienen su punto medio base de gatillos seteado arbitrariamente en `127` provocaban falsos positivos interrumpiendo un mapeo nativo.

## Solución en Base a Diferenciales
Creamos el método `capture_resting()` que extrae temporalmente una fotocopia del Hashmap en descanso de la sesión actual de `/dev/input/X`. Desde ahí, no evalúa qué botón se apretó, sino el _**Grosor de la Diferencia de Fuerza**_ o Treshold. Si la diferencia de estática relativa `delta.abs() >= thr`, el evento se procesa y el bloque ignora en su totalidad sensores inactivos "ruidosos".
