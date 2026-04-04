# Empaquetado y Distribución

Durante el desarrollo de XJemulator, establecimos un pipeline unificado diseñado en `dist/`. La idea es emitir ejecutables para múltiples ecosistemas Linux de manera atómica y simple.

## Scripts de Empaquetado (`dist/`)

### 1. Sistema Debian (`build_deb.sh`)
Usa `cargo-deb`. Se inyectan scripts temporales `postinst` que garantizan dos cosas:
- Otorgar a la aplicación la persistencia con rutas adecuadas.
- Cargar la udev automáticamente mediante `udevadm trigger`.
*Ruta de salida:* `target/debian/*.deb`

### 2. Sistema RPM / RedHat (`build_rpm.sh`)
Usa `cargo-generate-rpm`. Traduce la semántica Debian a dependencias `dnf`. Instala el `.desktop` y el binario de forma canónica.
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
