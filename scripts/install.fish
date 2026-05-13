#!/usr/bin/fish

# XJemulator - Universal Installation Script (Fish Version - PRODUCTION READY)
# Configura permisos, reglas udev, iconos y lanzadores.

set -g APP_NAME "xjemulator"
set -g CURRENT_BRANCH "master"
if set -q BRANCH
    set CURRENT_BRANCH $BRANCH
end

set -g REPO_BASE "https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
set -g BIN_DEST "/usr/local/bin/$APP_NAME"
set -g UDEV_DEST "/etc/udev/rules.d/99-$APP_NAME.rules"
set -g ICON_DEST "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -g DESKTOP_DEST "/usr/share/applications/$APP_NAME.desktop"

# Colores
set -l green (set_color green)
set -l blue (set_color blue)
set -l yellow (set_color yellow)
set -l red (set_color red)
set -l normal (set_color normal)

echo -e "$blue🚀 Iniciando instalacion universal de XJemulator (Fish Edition)...$normal"

# 1. Verificacion de sudo
if not command -v sudo >/dev/null
    echo -e "$red❌ Error: Se requiere 'sudo' para la instalacion.$normal"
    exit 1
end

# 2. Funcion de descarga segura
function download_asset
    set -l remote_path $argv[1]
    set -l local_path $argv[2]
    set -l dest $argv[3]
    
    echo -e "$yellow📥 Procesando $remote_path...$normal"
    if test -f "$local_path"
        sudo cp "$local_path" "$dest"
    else
        sudo curl -fsSL "$REPO_BASE/$remote_path" -o "$dest"
        if test $status -ne 0
            echo -e "$red❌ Error al descargar $remote_path desde $CURRENT_BRANCH$normal"
            exit 1
        end
    end
end

# 3. Configurar Hardware
echo -e "$yellow🔧 Configurando hardware (uinput)...$normal"
sudo modprobe uinput
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null

download_asset "udev/99-xjemulator.rules" "udev/99-xjemulator.rules" "$UDEV_DEST"

sudo udevadm control --reload-rules
sudo udevadm trigger
sudo setfacl -m u:$USER:rw /dev/uinput; or sudo chmod 666 /dev/uinput

# 4. Iconos y Lanzadores
download_asset "assets/xjemulator.svg" "assets/xjemulator.svg" "$ICON_DEST"
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor

download_asset "assets/xjemulator.desktop" "assets/xjemulator.desktop" "$DESKTOP_DEST"
sudo update-desktop-database /usr/share/applications

# 5. Instalacion del Binario
if test -f "Cargo.toml"
    echo -e "$yellow🔨 Detectado codigo fuente. Compilando en modo release...$normal"
    if command -v cargo >/dev/null
        cargo build --release
        sudo cp "target/release/$APP_NAME" "$BIN_DEST"
    else
        echo -e "$red❌ Error: 'cargo' (Rust) no esta instalado. No se puede compilar.$normal"
        exit 1
    end
else if test -f "target/release/$APP_NAME"
    echo -e "$green📦 Usando binario ya compilado...$normal"
    sudo cp "target/release/$APP_NAME" "$BIN_DEST"
else
    echo -e "$yellow📥 No hay codigo fuente ni binario local. Descargando pre-compilado...$normal"
    sudo curl -fsSL "https://github.com/johancy96/XJemulator/releases/latest/download/xjemulator" -o "$BIN_DEST"
    if test $status -ne 0
        echo -e "$red❌ Error: No se pudo descargar ni compilar el binario.$normal"
        exit 1
    end
end

if test -f "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
end

# 6. Estructura XDG
mkdir -p ~/.config/$APP_NAME/profiles

echo -e "$green✅ ¡XJemulator instalado con éxito!$normal"
echo -e "$blueℹ️  Buscalo en tu lanzador de aplicaciones o ejecuta: $APP_NAME$normal"
