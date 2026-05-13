#!/bin/bash

# XJemulator - Modern Uninstaller Script
# Revierte la instalacion y limpia el sistema de forma profesional.

set -e

# --- Configuracion Modular ---
APP_NAME="xjemulator"
UDEV_RULE="/etc/udev/rules.d/99-$APP_NAME.rules"
MODULE_CONF="/etc/modules-load.d/$APP_NAME.conf"
ICON_PATH="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_PATH="/usr/share/applications/$APP_NAME.desktop"
BIN_PATH="/usr/local/bin/$APP_NAME"
CONFIG_DIR="$HOME/.config/$APP_NAME"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🗑️ Iniciando desinstalacion de XJemulator...${NC}"

# 1. Verificacion de sudo
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}❌ Error: Se requiere 'sudo' para eliminar archivos del sistema.${NC}"
    exit 1
fi

# 2. Eliminar Binario
echo -e "${YELLOW}📂 Eliminando binario...${NC}"
[ -f "$BIN_PATH" ] && sudo rm -f "$BIN_PATH" && echo "  ✔ $BIN_PATH eliminado."

# 3. Revertir Configuracion del Sistema (Root)
echo -e "${YELLOW}🔧 Revirtiendo configuracion del sistema...${NC}"

# Reglas Udev
if [ -f "$UDEV_RULE" ]; then
    sudo rm -f "$UDEV_RULE"
    echo "  ✔ Reglas udev eliminadas."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
fi

# Modulos del Kernel
if [ -f "$MODULE_CONF" ]; then
    sudo rm -f "$MODULE_CONF"
    echo "  ✔ Configuracion de modulos eliminada."
fi

# Iconos y Lanzadores
if [ -f "$ICON_PATH" ]; then
    sudo rm -f "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    echo "  ✔ Icono eliminado."
fi

if [ -f "$DESKTOP_PATH" ]; then
    sudo rm -f "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications || true
    echo "  ✔ Lanzador .desktop eliminado."
fi

# 4. Limpieza de Datos de Usuario (Opcional)
if [[ "$1" == "--full" ]]; then
    echo -e "${RED}⚠️  Limpieza completa solicitada. Eliminando configuracion de usuario...${NC}"
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo "  ✔ $CONFIG_DIR eliminado."
    fi
else
    echo -e "${BLUE}ℹ️  Se han conservado tus perfiles en $CONFIG_DIR${NC}"
    echo -e "${BLUE}ℹ️  Usa '${YELLOW}--full${BLUE}' si deseas borrarlos tambien.${NC}"
fi

echo -e "\n${GREEN}✅ XJemulator ha sido desinstalado correctamente.${NC}"
