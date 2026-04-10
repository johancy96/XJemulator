# Empaquetado y Distribución

Durante el desarrollo de XJemulator, establecimos un pipeline unificado diseñado en `dist/`. La idea es emitir ejecutables para múltiples ecosistemas Linux de manera atómica y simple.

## Scripts de Empaquetado (`dist/`)

### 1. Sistema Debian (`build_packages.sh`)
Usa `cargo-deb`. Inyecta el archivo de reglas `.rules` y un script temporal `postinst` que garantiza:
- Cargar la regla udev automáticamente en el sistema mediante `udevadm control --reload-rules` y `udevadm trigger`.
*Ruta de salida:* `target/debian/*.deb`

### 2. Sistema RPM / RedHat (`build_packages.sh`)
Usa `cargo-generate-rpm`. Traduce la semántica a dependencias `dnf`. Instala el `.desktop`, los íconos, las reglas `udev` y el binario de forma canónica.
*Ruta de salida:* `target/generate-rpm/*.rpm`

### 3. AppImage (`build_appimage.sh`)
Construcción en una jerarquía AppDir. Proporciona independencia de la distro empaquetando también el propio script ejecutable y los íconos de la release. Este pipeline depende de la descarga del `appimagetool`.
*Ruta de salida:* `/dist/XJemulator-x86_64.AppImage`

### 4. Arch Linux / AUR (`PKGBUILD`)
Se redactó un archivo `PKGBUILD` listo para Arch Linux. Recompila desde fuentes usando la instrucción de `cargo build --release` y coloca los assets. Para subir a AUR es indispensable clonar el repositorio AUR del usuario, ubicar allí este archivo junto con un archivo `.SRCINFO` generado con `makepkg --printsrcinfo` y empujar cambios (git push).

## Guía de usuario de compilación
Los usuarios encuentran un script integral abstracto. En el `README.md` se promueve el siguiente flujo:
```bash
cd dist
./build_appimage.sh  # Genera el universal portátil.
```
