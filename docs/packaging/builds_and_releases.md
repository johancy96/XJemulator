# Infraestructura de Empaquetados / Distribuciones

XJemulator automatiza la dispersión de código hacia artefactos que los usuarios finales usarán en el mundo real en Linux, utilizando un catálogo de scripts en la carpeta `dist`.

## Flujo de Trabajo (La Carga Payload)
Tanto para DEB como para sistemas de paquetería RPM, se hace uso directo de metadatos acoplados dinámicamente al archivo nativo `Cargo.toml`.
Cuando el desarrollador ejecuta `dist/build_packages.sh`, el script busca los campos de `package.metadata.generate-rpm` e inyecta dinámicamente los recursos `.desktop` y el `.svg` oficial en las rutas requeridas de las carpetas abstractas de UNIX (`/usr/share/applications` para los visores como Gnome y KDE).

## Flujo AppImage
Se usa `linuxdeploy` en lugar de una herramienta nativa de Crate para una compilación más universal en el archivo `build_appimage.sh`. Para que esto funcione, el script inicializa un despliegue falso llamado "AppDir", clona los assets `.desktop` nativos, y los acopla en el compilador. Opcionalmente inserta por duplicado el SVG dentro de la raíz para resolver *bugs* de lectura detectados en algunos intérpretes antiguos de AppImageLauncher.

## AUR
El formato arch es extremadamente purista, por lo se optó por adjuntar nativamente el compilador directo en `dist/PKGBUILD` que compilará a partir de `.tar.gz` remoto extraído del repositorio de `v0.1.0`. Este compilador evita inmiscuir a Arch Linux en dependencias desmesuradas y delega a su gestor PACMAN el trabajo final de copia en el FHS.
