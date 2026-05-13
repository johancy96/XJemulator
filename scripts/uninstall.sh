#!/bin/bash

# XJemulator - Modern Uninstaller Script (PRODUCTION READY)
# Reverts installation and cleans up the system professionally.

set -e

# --- Configuration (Must match install.sh) ---
APP_NAME="xjemulator"
BIN_PATH="/usr/local/bin/$APP_NAME"
UDEV_PATH="/etc/udev/rules.d/99-$APP_NAME.rules"
MODULE_PATH="/etc/modules-load.d/$APP_NAME.conf"
ICON_PATH="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_PATH="/usr/share/applications/$APP_NAME.desktop"
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

# 2. Function for safe removal
remove_file() {
    local file_path="$1"
    local description="$2"
    if [ -f "$file_path" ]; then
        echo -e "${YELLOW}📂 Removing $description...${NC}"
        sudo rm -f "$file_path"
        echo -e "  ${GREEN}✔ Removed: $file_path${NC}"
    fi
}

# 3. System Cleanup
remove_file "$BIN_PATH" "binary"
remove_file "$UDEV_PATH" "udev rules"
remove_file "$MODULE_PATH" "kernel module config"
remove_file "$ICON_PATH" "application icon"
remove_file "$DESKTOP_PATH" "desktop launcher"

# Reload system databases
echo -e "${YELLOW}🔄 Refreshing system databases...${NC}"
sudo udevadm control --reload-rules && sudo udevadm trigger || true
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
sudo update-desktop-database /usr/share/applications || true

# 4. Optional User Data Cleanup
if [[ "$1" == "--full" ]]; then
    echo -e "${RED}⚠️  Full cleanup requested. Removing user data...${NC}"
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo -e "  ${GREEN}✔ Removed: $CONFIG_DIR${NC}"
    fi
else
    echo -e "\n${BLUE}ℹ️  User profiles kept at $CONFIG_DIR${NC}"
    echo -e "${BLUE}ℹ️  Run with '${YELLOW}--full${BLUE}' to remove them as well.${NC}"
fi

echo -e "\n${GREEN}✅ XJemulator has been successfully uninstalled.${NC}"
