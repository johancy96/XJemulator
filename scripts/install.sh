#!/bin/bash

# XJemulator - Universal Installation Script (PRODUCTION READY)
# Configura permisos, reglas udev, iconos y lanzadores.

set -e

# --- Configuracion ---
APP_NAME="xjemulator"
# Por defecto master, pero se puede sobrescribir con BRANCH=nombre
CURRENT_BRANCH="${BRANCH:-master}"
REPO_BASE="https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
BIN_DEST="/usr/local/bin/$APP_NAME"
UDEV_DEST="/etc/udev/rules.d/99-$APP_NAME.rules"
ICON_DEST="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_DEST="/usr/share/applications/$APP_NAME.desktop"

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Iniciando instalacion universal de XJemulator...${NC}"

# 1. Verificacion de sudo
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}❌ Error: Se requiere 'sudo' para la instalacion.${NC}"
    exit 1
fi

# 2. Funcion de descarga segura
download_asset() {
    local remote_path="$1"
    local local_path="$2"
    local dest="$3"
    
    echo -e "${YELLOW}📥 Procesando $remote_path...${NC}"
    if [ -f "$local_path" ]; then
        sudo cp "$local_path" "$dest"
    else
        # Descarga desde GitHub con flag -f para fallar en 404
        sudo curl -fsSL "$REPO_BASE/$remote_path" -o "$dest" || {
            echo -e "${RED}❌ Error al descargar $remote_path desde $CURRENT_BRANCH${NC}"
            exit 1
        }
    fi
}

# 3. Configurar Kernel y Reglas Udev
echo -e "${YELLOW}🔧 Configurando hardware (uinput)...${NC}"
sudo modprobe uinput || true
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null

download_asset "udev/99-xjemulator.rules" "udev/99-xjemulator.rules" "$UDEV_DEST"

sudo udevadm control --reload-rules
sudo udevadm trigger
# Permiso inmediato
sudo setfacl -m u:$USER:rw /dev/uinput || sudo chmod 666 /dev/uinput

# 4. Iconos y Lanzadores
download_asset "assets/xjemulator.svg" "assets/xjemulator.svg" "$ICON_DEST"
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true

download_asset "assets/xjemulator.desktop" "assets/xjemulator.desktop" "$DESKTOP_DEST"
sudo update-desktop-database /usr/share/applications || true

# 5. Instalacion del Binario
if [ -f "Cargo.toml" ]; then
    echo -e "${YELLOW}🔨 Detectado codigo fuente. Compilando en modo release...${NC}"
    if command -v cargo &> /dev/null; then
        cargo build --release
        sudo cp "target/release/$APP_NAME" "$BIN_DEST"
    else
        echo -e "${RED}❌ Error: 'cargo' (Rust) no esta instalado. No se puede compilar.${NC}"
        exit 1
    fi
elif [ -f "target/release/$APP_NAME" ]; then
    echo -e "${GREEN}📦 Usando binario ya compilado...${NC}"
    sudo cp "target/release/$APP_NAME" "$BIN_DEST"
else
    echo -e "${YELLOW}📥 No hay codigo fuente ni binario local. Descargando pre-compilado...${NC}"
    sudo curl -fsSL "https://github.com/johancy96/XJemulator/releases/latest/download/xjemulator" -o "$BIN_DEST" || {
        echo -e "${RED}❌ Error: No se pudo descargar ni compilar el binario.${NC}"
        exit 1
    }
fi

if [ -f "$BIN_DEST" ]; then
    sudo chmod +x "$BIN_DEST"
fi

# 6. Estructura XDG
mkdir -p ~/.config/$APP_NAME/profiles

echo -e "${GREEN}✅ ¡XJemulator instalado con éxito!${NC}"
echo -e "${BLUE}ℹ️  Buscalo en tu lanzador de aplicaciones o ejecuta: $APP_NAME${NC}"
