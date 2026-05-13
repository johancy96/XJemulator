#!/bin/bash

# XJemulator - Modern Installation Script
# Configura permisos, reglas udev y binarios sin necesidad de reiniciar.

set -e

# --- Configuracion Modular ---
APP_NAME="xjemulator"

# Detectar rama actual (Prioridad: Env Var BRANCH > Git Local > Default master)
if [ -n "$BRANCH" ]; then
    CURRENT_BRANCH="$BRANCH"
elif git rev-parse --abbrev-ref HEAD &>/dev/null; then
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
else
    CURRENT_BRANCH="master"
fi

REPO_RAW="https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
# Nota: La lógica de descarga de udev ya busca en udev/, no necesita cambiar REPO_RAW aquí 
# pero nos aseguramos de que sea consistente.
BIN_DEST="/usr/local/bin/$APP_NAME"
UDEV_RULE="99-$APP_NAME.rules"
CONFIG_DIR="$HOME/.config/$APP_NAME/profiles"

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Iniciando instalacion profesional de XJemulator...${NC}"
echo -e "${BLUE}📍 Rama detectada: ${YELLOW}$CURRENT_BRANCH${NC}"

# 1. Verificacion de requisitos
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}❌ Error: Se requiere 'sudo' para configurar los permisos del sistema.${NC}"
    exit 1
fi

# 2. Configuración del Kernel (uinput)
echo -e "${YELLOW}🔧 Configurando modulo uinput...${NC}"
sudo modprobe uinput || true
# Asegurar persistencia del modulo tras reinicios
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null

# 3. Instalacion de Reglas Udev (Acceso instantaneo)
echo -e "${YELLOW}📜 Instalando reglas udev (TAG+=uaccess)...${NC}"
# Descargar regla directamente desde el repo si no existe localmente
if [ -f "udev/$UDEV_RULE" ]; then
    sudo cp "udev/$UDEV_RULE" "/etc/udev/rules.d/"
else
    curl -sSL "$REPO_RAW/udev/$UDEV_RULE" | sudo tee "/etc/udev/rules.d/$UDEV_RULE" > /dev/null
fi

# Aplicar reglas sin reiniciar
sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Permisos inmediatos para el usuario actual (Fallback de seguridad)
echo -e "${YELLOW}🔑 Aplicando permisos de acceso inmediato...${NC}"
if command -v setfacl &> /dev/null; then
    sudo setfacl -m u:$USER:rw /dev/uinput || true
else
    # Fallback si acl no está instalado
    sudo chmod 666 /dev/uinput || true
fi

# 5. Instalación del Icono y Lanzador
echo -e "${YELLOW}🖼️ Instalando icono y lanzador en el sistema...${NC}"
ICON_PATH="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_PATH="/usr/share/applications/$APP_NAME.desktop"

if [ -f "assets/$APP_NAME.svg" ]; then
    sudo mkdir -p /usr/share/icons/hicolor/scalable/apps/
    sudo cp "assets/$APP_NAME.svg" "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
fi

if [ -f "assets/$APP_NAME.desktop" ]; then
    sudo cp "assets/$APP_NAME.desktop" "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications || true
fi

# 6. Preparar estructura XDG
echo -e "${YELLOW}📁 Creando directorios de configuracion en ~/.config...${NC}"
mkdir -p "$CONFIG_DIR"

# 6. Instalacion del Binario (Solo si se solicita o no existe)
if [[ "$1" == "--bin" ]]; then
    echo -e "${YELLOW}📥 Descargando binario mas reciente...${NC}"
    # Aquí iría la lógica de descarga del release (ejemplo)
    # curl -L "URL_DEL_RELEASE" -o /tmp/$APP_NAME
    # sudo install -m 755 /tmp/$APP_NAME "$BIN_DEST"
fi

echo -e "${GREEN}✅ Instalacion completada con exito!${NC}"
echo -e "${BLUE}ℹ️  Ya puedes ejecutar XJemulator sin necesidad de reiniciar el equipo.${NC}"
echo -e "${BLUE}ℹ️  Los perfiles se guardaran en: $CONFIG_DIR${NC}"
