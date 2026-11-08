---
name: podman-input-lab
description: >-
  Guía y flujos de trabajo con Podman para compilar XJemulator, generar paquetes
  (.deb / .rpm) en entornos limpios multi-distro y probar reglas udev, evdev y
  uinput pasando dispositivos del kernel al contenedor.
---

# Podman Input & Packaging Lab (XJemulator)

Usa esta skill cuando el usuario pida construir, empaquetar, probar instaladores (`scripts/install.sh` / `scripts/install.fish`), o depurar detección de mandos (`evdev`/`uinput`) utilizando **Podman**.

## 1. Compilación y Empaquetado Reproducible con Podman

Para compilar o generar paquetes `.deb` y `.rpm` sin contaminar el sistema anfitrión:

### Generar binario Release y paquete `.deb` (Debian/Ubuntu)
```bash
podman run --rm -it \
  -v "$(pwd):/workspace:Z" \
  -w /workspace \
  docker.io/library/rust:latest \
  bash -c "apt-get update && apt-get install -y libdbus-1-dev pkg-config && cargo install cargo-deb && cargo build --release && cargo deb"
```

### Generar paquete `.rpm` (Fedora/RHEL)
```bash
podman run --rm -it \
  -v "$(pwd):/workspace:Z" \
  -w /workspace \
  registry.fedoraproject.org/fedora:latest \
  bash -c "dnf install -y rust cargo dbus-devel pkgconf-pkg-config gcc && cargo install cargo-generate-rpm && cargo build --release && strip -s target/release/xjemulator && cargo generate-rpm"
```

## 2. Passthrough del Kernel (`evdev` y `uinput`) en Podman

Para probar la detección de mandos (`/dev/input/event*`) y la creación del mando virtual Xbox 360 (`/dev/uinput`) dentro de un contenedor Podman:

```bash
podman run --rm -it \
  --userns=keep-id \
  --group-add keep-groups \
  --device /dev/uinput \
  --device /dev/input \
  -v "$(pwd):/workspace:Z" \
  -w /workspace \
  docker.io/library/rust:latest \
  cargo run --release
```

### Diagnóstico rápido del subsistema de entrada en el Host o Contenedor
1. **Verificar módulo `uinput` cargado y permisos**:
   ```bash
   lsmod | grep uinput
   ls -l /dev/uinput
   getfacl /dev/uinput
   ```
2. **Inspeccionar dispositivos detectados por el kernel**:
   ```bash
   cat /proc/bus/input/devices
   ```
3. **Confirmar que el mando virtual Xbox 360 fue creado correctamente**:
   - Debe aparecer un nodo con `Name="Microsoft X-Box 360 pad"` y `Vendor=045e Product=028e Version=0114` en `/proc/bus/input/devices` y bajo `/sys/devices/virtual/input/`.
