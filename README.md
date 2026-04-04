# XJemulator 🎮

[🌍 English Version](file:///home/johancy/Proyectos/XJemulator/README_EN.md) / [🇪🇸 Versión en Español](file:///home/johancy/Proyectos/XJemulator/README.md)

XJemulator es una aplicación gráfica moderna para Linux que permite interceptar cualquier mando genérico y emular un controlador oficial de Xbox 360 de forma local. Es totalmente bilingüe (Español/Inglés) y está configurado para funcionar a nivel de kernel utilizando las utilidades `uinput` y `udev`.

---

## 🚀 Instalación Rápida (Recomendado)

Si solo quieres instalar el programa sin descargar todo el repositorio, ejecuta este comando en tu terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/dist/install.sh | bash
```

---

## 🛠 Instalación y Guía de Compilación

Hemos integrado herramientas automáticas para que compilar XJemulator y generar instaladores directamente para tu distribución sea un proceso libre de problemas.

### Paso 1: Instalar Dependencias del Sistema
Antes de poder construir la aplicación en tu PC, deberás tener las herramientas base de compilación de tu sistema. 

**Selecciona tu distribución y ejecuta el comando en la terminal:**

- 🟠 **Debian / Ubuntu / Linux Mint / Pop!_OS:**
  ```bash
  sudo apt update
  sudo apt install -y build-essential curl
  ```
- 🔵 **Fedora / RHEL / CentOS:**
  ```bash
  sudo dnf groupinstall "Development Tools"
  sudo dnf install curl
  ```
- 🟣 **Arch Linux / Manjaro / EndeavourOS:**
  ```bash
  sudo pacman -Sy base-devel curl
  ```

### Paso 2: Entorno de Programación (Rust)
Nos apoyaremos en Rust para compilar. Este comando instalará sin requerir permisos de administrador la cadena de compiladores en tu máquina:
```bash
# Ejecuta e instala con las opciones por defecto (presionando la tecla Enter)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Recarga la terminal para habilitar el comando base de trabajo 'cargo'
source "$HOME/.cargo/env"
```

---

## 🚀 Empaquetado Automático (¡Armando los instaladores!)

Una vez tengas clonado o descargado este repositorio a tu computadora, abre la terminal dentro de esta misma carpeta.

Dependiendo del formato base de tu sistema operativo, ejecuta el script de empaquetado que automatizará todo el trabajo sucio por ti:

### 📦 Opción A: Sistemas Debian (.deb)
Recomendado si quieres lograr la máxima integración del escritorio en Ubuntu, Mint o Debian. 

1. Ejecuta el archivo Bash:
   ```bash
   ./dist/build_packages.sh
   ```
2. **Descarga:** Un archivo empaquetado `.deb` será generado por arte de magia en la ruta interna `target/debian/`.
3. **Instalación:** Para instalar este generador hazle doble clic en el administrador de archivos, o ejecuta `sudo dpkg -i target/debian/*.deb`.

### 📦 Opción B: Sistemas Fedora y RedHat (.rpm)
Al igual que en Debian, puedes utilizar el mismo script base.
1. Ejecuta:
   ```bash
   ./dist/build_packages.sh
   ```
2. **Descarga:** Podrás localizar el archivo resultante `.rpm` esperando en `target/generate-rpm/`.
3. **Instalación:** Hazle doble clic o emplea en consola `sudo dnf localinstall target/generate-rpm/*.rpm`.

### 🌍 Opción C: Formato Portable (.AppImage)
Ideal si usas distribuciones heterogéneas, Immutable OS (como Fedora Silverblue / Kinoite) o simplemente prefieres llevarte el programa en un Pendrive a otra PC que tenga Linux sin tener que volver a compilar nada.
1. Ejecuta el generador AppImage:
   ```bash
   ./dist/build_appimage.sh
   ```
2. **Descarga e Instalación:** Descargará un AppImage de LinuxDeploy y creará el aplicativo encapsulado llamado **`XJemulator-x86_64.AppImage`** exactamente donde estás parado. Entrégale permisos dando clic derecho > Propiedades > "Permitir ejecutar como un programa" (O mediante terminal `chmod +x XJemulator*.AppImage`) y ábrelo en cualquier Linux moderno a doble clic.

### 🐧 Opción D: Arch Linux y AUR
Gracias al archivo `PKGBUILD` incluido, los usuarios de Arch no necesitan compilar por paquetes externos, pueden hacerlo de forma automatizada mediante makepkg.
```bash
# Accede a la carpeta con el archivo
cd dist

# Descarga las librerías, empaqueta e instálalo nativamente (pacman)
makepkg -si
```

---

## 💻 Desarrollo Directo (No generar instaladores)

Si no te interesa instalarlo sino abrir el visualizador de forma local o modificar su contenido:
```bash
# Correr aplicativo saltándose depuraciones
cargo run --release
```

---

## 🖥 Integración al Lanzador de Aplicaciones (Instalación Manual)

Si decidiste saltarte los instaladores automáticos (`.deb` / `.rpm`) y usaste compilación directa o el formato portátil `.AppImage`, es posible que quieras que XJemulator aparezca junto a tus demás programas en el menú de escritorio de Linux (Ej: GNOME, Plasma, Cinnamon). Para ello, hemos adjuntado los vínculos nativos en la carpeta `assets/`.

1. **Instala el Ícono:**
   ```bash
   sudo cp assets/xjemulator.svg /usr/share/icons/hicolor/scalable/apps/
   ```
2. **Copia el archivo .desktop a la ruta del menú de Linux:**
   ```bash
   # Si quieres que aparezca solo para tu usuario:
   mkdir -p ~/.local/share/applications/
   cp assets/xjemulator.desktop ~/.local/share/applications/
   
   # O si prefieres instalar el menú para todos los perfiles de la PC:
   sudo cp assets/xjemulator.desktop /usr/share/applications/
   ```

> *Nota: Abre el archivo copiado `.desktop` con un editor de texto y asegúrate de modificar la línea que dice `Exec=xjemulator` para que apunte directamente hacia la ruta de tu nuevo ejecutable (Ej: `Exec=/home/usuario/descargas/XJemulator-x86_64.AppImage`).*
