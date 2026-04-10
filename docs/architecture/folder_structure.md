# Estructura del Árbol de Proyecto (Directorio)

Para que agentes futuros mantengan o extiendan XJemulator, esta es la disposición explícita de archivos creada al final del ciclo de desarrollo `v0.1.0`.

```text
XJemulator/
├── assets/                     # Archivos estáticos de distribución de Linux
│   ├── xjemulator.desktop      # Archivo para lanzador / menú del sistema
│   └── xjemulator.svg          # Logo vectorial de la emulación nativa
├── dist/                       # Scripts de empaquetado para distribución
│   ├── build_appimage.sh       # Construye encapsulamiento universal AppImage
│   ├── build_packages.sh       # Script wrapper para DEB y RPM
│   └── PKGBUILD                # Fórmula oficial de AUR para Arch Linux
├── src/
│   ├── gui/                    # Toda la Interfaz Gráfica (Egui)
│   │   ├── app.rs              # Monolito del Fronted: Lógica UI principal
│   │   ├── calibration.rs      # [DEPRECADO] Archivo viejo para el formato interno de mapeo (movido)
│   │   └── controller.rs       # Entidad SVG renderizada en memoria para UX de Gamepad 360
│   ├── config.rs               # Modelo de `AppConfig`, almacena y escribe configuraciones persistentes (toml)
│   ├── error.rs                # Propagación unificada de Errores Result en Rust
│   ├── i18n.rs                 # Diccionarios de Internacionalización para Multi-lenguaje (ES/EN)
│   ├── main.rs                 # Entry-point (solo llama a lib / gui)
│   ├── mapper.rs               # Motor transductor del Evento Raw -> Xbox 360 Event
│   ├── reader.rs               # Event-loop asincrónico para interceptar /dev/input/
│   ├── scanner.rs              # Listado de joypads reales conectados al SO
│   ├── virtual_device.rs       # Módulo uinput: creador del driver virtual
│   └── xbox_descriptor.rs      # Hardcodes PID USB oficiales de la Microsoft Xbox 360 
├── .gitignore
├── Cargo.toml                  # Dependencias y Metadata de extensiones (cargo-deb, cargo-generate-rpm)
├── GUIDE.md                    # Guía para End-Users a nivel técnico
├── LICENSE                     # Licencia legal MIT
└── README.md                   # Welcome Screen Repository / Manual de Empaquetado
```
