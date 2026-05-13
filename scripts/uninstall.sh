#!/bin/bash

# XJemulator - Modern Uninstaller Script
# Reverts installation and cleans up the system professionally.

set -e

# --- Modular Configuration ---
APP_NAME="xjemulator"
UDEV_RULE="/etc/udev/rules.d/99-$APP_NAME.rules"
MODULE_CONF="/etc/modules-load.d/$APP_NAME.conf"
ICON_PATH="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_PATH="/usr/share/applications/$APP_NAME.desktop"
BIN_PATH="/usr/local/bin/$APP_NAME"
CONFIG_DIR="$HOME/.config/$APP_NAME"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🗑️ Starting XJemulator uninstallation...${NC}"

# 1. Sudo check
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}❌ Error: 'sudo' is required to remove system files.${NC}"
    exit 1
fi

# 2. Remove Binary
echo -e "${YELLOW}📂 Removing binary...${NC}"
[ -f "$BIN_PATH" ] && sudo rm -f "$BIN_PATH" && echo "  ✔ $BIN_PATH removed."

# 3. Revert System Configuration (Root)
echo -e "${YELLOW}🔧 Reverting system configuration...${NC}"

# Udev Rules
if [ -f "$UDEV_RULE" ]; then
    sudo rm -f "$UDEV_RULE"
    echo "  ✔ Udev rules removed."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
fi

# Kernel Modules
if [ -f "$MODULE_CONF" ]; then
    sudo rm -f "$MODULE_CONF"
    echo "  ✔ Kernel module configuration removed."
fi

# Icons and Launchers
if [ -f "$ICON_PATH" ]; then
    sudo rm -f "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
    echo "  ✔ Icon removed."
fi

if [ -f "$DESKTOP_PATH" ]; then
    sudo rm -f "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications || true
    echo "  ✔ .desktop launcher removed."
fi

# 4. Cleanup User Data (Optional)
if [[ "$1" == "--full" ]]; then
    echo -e "${RED}⚠️  Full cleanup requested. Removing user configuration...${NC}"
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo "  ✔ $CONFIG_DIR removed."
    fi
else
    echo -e "${BLUE}ℹ️  User profiles kept at $CONFIG_DIR${NC}"
    echo -e "${BLUE}ℹ️  Use '${YELLOW}--full${BLUE}' if you want to delete them as well.${NC}"
fi

echo -e "\n${GREEN}✅ XJemulator has been successfully uninstalled.${NC}"
