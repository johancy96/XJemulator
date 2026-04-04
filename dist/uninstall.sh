#!/bin/bash

# XJemulator - Universal Uninstaller Script
# This script removes XJemulator from any Linux distribution.

set -e

# ANSI Color Codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}      XJemulator - Universal Uninstaller   ${NC}"
echo -e "${BLUE}=========================================${NC}"

# 1. Detect Package Manager and Attempt Native Uninstall
echo -e "\n${YELLOW}[1/4] Detecting Package Manager...${NC}"

if [ -f /etc/debian_version ]; then
    echo -e "Detected Debian-based system. Trying 'apt'..."
    if dpkg -l | grep -q xjemulator; then
        sudo apt remove -y xjemulator || true
    else
        echo "No 'xjemulator' package found in apt."
    fi
elif [ -f /etc/fedora-release ] || [ -f /etc/redhat-release ]; then
    echo -e "Detected Fedora/RedHat-based system. Trying 'dnf'..."
    if rpm -q xjemulator &>/dev/null; then
        sudo dnf remove -y xjemulator || true
    else
        echo "No 'xjemulator' package found in dnf."
    fi
elif [ -f /etc/arch-release ]; then
    echo -e "Detected Arch-based system. Trying 'pacman'..."
    if pacman -Qi xjemulator &>/dev/null; then
        sudo pacman -Rs --noconfirm xjemulator || true
    else
        echo "No 'xjemulator' package found in pacman."
    fi
else
    echo "Unknown or non-standard package manager. Skipping native uninstall."
fi

# 2. Cleanup Manual Global Files
echo -e "\n${YELLOW}[2/4] Cleaning up Global System Files...${NC}"

FILES_TO_REMOVE=(
    "/usr/bin/xjemulator"
    "/usr/local/bin/xjemulator"
    "/usr/share/applications/xjemulator.desktop"
    "/usr/share/icons/hicolor/scalable/apps/xjemulator.svg"
    "/usr/share/pixmaps/xjemulator.png"
    "/etc/udev/rules.d/99-xjemulator.rules"
    "/home/$USER/.local/share/applications/xjemulator.desktop"
)

for file in "${FILES_TO_REMOVE[@]}"; do
    if [ -f "$file" ]; then
        echo -e "Removing: $file"
        sudo rm -f "$file"
    fi
done

# 3. Reload System Configurations
echo -e "\n${YELLOW}[3/4] Reloading System Configurations...${NC}"

if command -v udevadm &>/dev/null; then
    echo "Reloading udev rules..."
    sudo udevadm control --reload-rules && sudo udevadm trigger || true
fi

if command -v update-desktop-database &>/dev/null; then
    echo "Updating desktop database..."
    update-desktop-database ~/.local/share/applications/ &>/dev/null || true
    sudo update-desktop-database /usr/share/applications/ &>/dev/null || true
fi

# 4. Optional User Data Cleanup
echo -e "\n${YELLOW}[4/4] Finishing up...${NC}"

if [ "$1" == "--full" ]; then
    echo -e "${RED}Full cleanup requested. Removing local configuration artifacts...${NC}"
    rm -f config.toml mi_mando.toml
    rm -rf profiles/
    echo "Local workspace artifacts removed."
fi

echo -e "\n${GREEN}✔ XJemulator has been successfully uninstalled from your system.${NC}"
echo -e "${YELLOW}Note: If you have manual binary files in custom folders, you may need to delete them manually.${NC}"
echo -e "${BLUE}=========================================${NC}\n"
