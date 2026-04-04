#!/bin/bash

# XJemulator - Fast Installation Script (curl | bash)
# This script automates the installation of XJemulator on Linux systems.

set -e

# Configuration
REPO_URL="https://raw.githubusercontent.com/johancy96/XJemulator/master"
RELEASE_URL="https://github.com/johancy96/XJemulator/releases/latest/download/xjemulator"
INSTALL_DIR="/usr/local/bin"

# ANSI Color Codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}      XJemulator - Rapid Installer       ${NC}"
echo -e "${BLUE}=========================================${NC}"

# Check for sudo
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}Error: 'sudo' is required to install XJemulator.${NC}"
    exit 1
fi

# 1. Obtain Binary
echo -e "\n${YELLOW}[1/4] Acquiring XJemulator Binary...${NC}"

# Try downloading pre-compiled binary first
if curl -sLf "$RELEASE_URL" -o xjemulator_tmp; then
    echo "Using pre-compiled binary from GitHub Releases."
    sudo mv xjemulator_tmp "$INSTALL_DIR/xjemulator"
    sudo chmod +x "$INSTALL_DIR/xjemulator"
elif command -v cargo &> /dev/null; then
    echo "Releases not found or unavailable. Compiling from source..."
    git clone https://github.com/johancy96/XJemulator.git /tmp/xjemulator_src
    cd /tmp/xjemulator_src
    cargo build --release
    sudo mv target/release/xjemulator "$INSTALL_DIR/xjemulator"
    cd - && rm -rf /tmp/xjemulator_src
else
    echo -e "${RED}Error: Could not find pre-compiled binary and 'cargo' is not installed.${NC}"
    echo "Please visit https://github.com/johancy96/XJemulator to download it manually."
    exit 1
fi

# 2. Desktop Integration
echo -e "\n${YELLOW}[2/4] Integrating with Desktop environment...${NC}"

# Download and install icon
sudo mkdir -p /usr/share/icons/hicolor/scalable/apps/
sudo curl -sLf "$REPO_URL/assets/xjemulator.svg" -o /usr/share/icons/hicolor/scalable/apps/xjemulator.svg

# Download and install desktop entry
sudo curl -sLf "$REPO_URL/assets/xjemulator.desktop" -o /usr/share/applications/xjemulator.desktop
echo "Desktop launcher installed in /usr/share/applications/"

# 3. udev & Permissions
echo -e "\n${YELLOW}[3/4] Configuring Hardware Permissions (udev)...${NC}"

# Download and install udev rules
sudo curl -sLf "$REPO_URL/udev/99-xjemulator.rules" -o /etc/udev/rules.d/99-xjemulator.rules
echo "udev rules installed in /etc/udev/rules.d/"

# Reload udev
sudo udevadm control --reload-rules && sudo udevadm trigger || true

# Add user to input group
if ! groups $USER | grep -q "\binput\b"; then
    echo "Adding $USER to the 'input' group..."
    sudo usermod -aG input $USER
    REBOOT_REQUIRED=true
fi

# 4. Finalizing
echo -e "\n${YELLOW}[4/4] Finishing installation...${NC}"

if command -v update-desktop-database &> /dev/null; then
    sudo update-desktop-database /usr/share/applications/ &> /dev/null || true
fi

echo -e "\n${GREEN}✔ XJemulator v0.1.0 installed successfully!${NC}"

if [ "$REBOOT_REQUIRED" = true ]; then
    echo -e "${RED}IMPORTANT: Please log out and back in (or restart) for the 'input' group changes to take effect.${NC}"
fi

echo -e "${BLUE}=========================================${NC}"
echo -e "You can now launch XJemulator from your application menu or by typing 'xjemulator' in the terminal."
