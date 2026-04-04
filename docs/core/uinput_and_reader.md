# Creación de Dispositivos Virtuales (Virtual Device) / Reader

Alojada en `src/virtual_device.rs` y `src/reader.rs`.

## Modus Operandi (El Reader)
El módulo `reader` toma el control de un `Gamepad` real conectado. 
Su ciclo (event loop asincrónico) lee el path (ej: `/dev/input/event12`). Si el identificativo general es aceptado, intercepta con GRAB (Exclusividad Mutua). 

## Output Engine a Xbox360 (Virtual Device)
El control `virtual_device.rs` tiene un constructor atado a una especificación de periférico oficial, incrustando los IDs (`VendorID`, `ProductID` y `DeviceName`) extraídos del catálogo de reposición histórica mediante el `xbox_descriptor.rs`.

A través del acceso nativo a `uinput` se crea este nodo en la memoria y kernel de Linux, instruyendo al sistema a re-interpretar el input procesado mediante el _Mapper_ en salidas legítimas enviadas en la misma latencia, sin lag.
