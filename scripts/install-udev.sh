#!/usr/bin/env bash
# ==============================================================================
# XJemulator — Instalador Simplificado de Reglas udev y Permisos de Kernel
# ==============================================================================
# Configura únicamente el módulo `uinput`, las reglas udev (99-xjemulator.rules)
# y el grupo `input` para permitir la emulación del mando Xbox 360 sin root.
#
# Uso vía curl:
#   curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/install-udev.sh | bash
#
# Uso local:
#   bash scripts/install-udev.sh
# ==============================================================================

set -euo pipefail

APP_NAME="xjemulator"
BRANCH="${BRANCH:-master}"
UDEV_URL="https://raw.githubusercontent.com/johancy96/XJemulator/${BRANCH}/udev/99-xjemulator.rules"
UDEV_DEST="/etc/udev/rules.d/99-${APP_NAME}.rules"
MODULE_CONF="/etc/modules-load.d/${APP_NAME}.conf"
TARGET_USER="${SUDO_USER:-$USER}"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔧 Configurando reglas udev y permisos de kernel para XJemulator...${NC}"

if ! command -v sudo &>/dev/null; then
    echo -e "${RED}❌ Error: Se requiere 'sudo' para instalar las reglas udev en /etc/udev/rules.d/.${NC}"
    exit 1
fi

# 1. Cargar módulo uinput en el kernel actual y persistirlo en el arranque
echo -e "${YELLOW}➡ Habilitando módulo del kernel 'uinput'...${NC}"
sudo modprobe uinput || true
echo "uinput" | sudo tee "$MODULE_CONF" >/dev/null

# 2. Instalar archivo de reglas udev (local si existe, o vía curl desde el repositorio)
if [ -f "udev/99-xjemulator.rules" ]; then
    echo -e "${YELLOW}➡ Copiando reglas locales desde udev/99-xjemulator.rules...${NC}"
    sudo cp "udev/99-xjemulator.rules" "$UDEV_DEST"
else
    echo -e "${YELLOW}➡ Descargando reglas udev desde GitHub (${BRANCH})...${NC}"
    sudo curl -fsSL "$UDEV_URL" -o "$UDEV_DEST"
fi

sudo chmod 644 "$UDEV_DEST"

# 3. Asegurar que el usuario pertenezca al grupo 'input'
if [ -n "$TARGET_USER" ] && [ "$TARGET_USER" != "root" ]; then
    echo -e "${YELLOW}➡ Añadiendo al usuario '${TARGET_USER}' al grupo 'input'...${NC}"
    sudo groupadd -f input
    sudo usermod -aG input "$TARGET_USER"
fi

# 4. Recargar reglas udev y aplicar permisos inmediatos sin requerir reinicio
echo -e "${YELLOW}➡ Recargando reglas udev...${NC}"
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=misc --subsystem-match=input || sudo udevadm trigger

if [ -e /dev/uinput ] && [ -n "$TARGET_USER" ] && command -v setfacl &>/dev/null; then
    sudo setfacl -m "u:${TARGET_USER}:rw" /dev/uinput || true
fi

echo -e "${GREEN}✅ Reglas udev instaladas correctamente en ${UDEV_DEST}${NC}"
echo -e "${BLUE}ℹ️  Si es la primera vez que se añade tu usuario al grupo 'input', cierra sesión y vuelve a entrar.${NC}"
