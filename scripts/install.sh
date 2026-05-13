#!/bin/bash

# XJemulator - Universal Installation Script (HYBRID BUILD VERSION)
# Configures permissions, udev rules, icons and launchers with remote auto-compilation.

set -e

# --- Configuration ---
APP_NAME="xjemulator"
CURRENT_BRANCH="${BRANCH:-master}"
REPO_URL="https://github.com/johancy96/XJemulator.git"
REPO_RAW="https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
BIN_DEST="/usr/local/bin/$APP_NAME"
UDEV_DEST="/etc/udev/rules.d/99-$APP_NAME.rules"
ICON_DEST="/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
DESKTOP_DEST="/usr/share/applications/$APP_NAME.desktop"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

show_error_help() {
    echo -e "\n${RED}❌ Error: Could not obtain XJemulator binary.${NC}"
    echo -e "${YELLOW}To compile from source, you need to install dependencies:${NC}"
    echo -e "  - Rust & Cargo (https://rustup.rs)"
    echo -e "  - DBus development files"
    echo -e "  - GCC / Build Essentials"
    echo -e "\n${BLUE}Suggested commands:${NC}"
    echo -e "  Arch Linux: ${GREEN}sudo pacman -S rust dbus pkgconf base-devel${NC}"
    echo -e "  Ubuntu/Debian: ${GREEN}sudo apt install cargo libdbus-1-dev pkg-config build-essential${NC}"
    echo -e "  Fedora: ${GREEN}sudo dnf install cargo dbus-devel pkgconf-pkg-config development-tools${NC}"
    exit 1
}

echo -e "${BLUE}🚀 Starting XJemulator Universal Installation...${NC}"

# 1. Sudo check
if ! command -v sudo &> /dev/null; then
    echo -e "${RED}❌ Error: 'sudo' is required for installation.${NC}"
    exit 1
fi

# 2. Secure download function
download_asset() {
    local remote_path="$1"
    local local_path="$2"
    local dest="$3"
    
    echo -e "${YELLOW}📥 Processing $remote_path...${NC}"
    if [ -f "$local_path" ]; then
        sudo cp "$local_path" "$dest"
    else
        sudo curl -fsSL "$REPO_RAW/$remote_path" -o "$dest" || {
            echo -e "${RED}❌ Error downloading $remote_path${NC}"
            exit 1
        }
    fi
}

# 3. Configure Hardware
echo -e "${YELLOW}🔧 Configuring hardware (uinput)...${NC}"
sudo modprobe uinput || true
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null
download_asset "udev/99-xjemulator.rules" "udev/99-xjemulator.rules" "$UDEV_DEST"
sudo udevadm control --reload-rules && sudo udevadm trigger
sudo setfacl -m u:$USER:rw /dev/uinput || sudo chmod 666 /dev/uinput

# 4. Icons and Launchers
download_asset "assets/xjemulator.svg" "assets/xjemulator.svg" "$ICON_DEST"
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor || true
download_asset "assets/xjemulator.desktop" "assets/xjemulator.desktop" "$DESKTOP_DEST"
sudo update-desktop-database /usr/share/applications || true

# 5. Binary Management (Hybrid)
install_binary() {
    if [ -f "Cargo.toml" ]; then
        echo -e "${YELLOW}🔨 Compiling locally...${NC}"
        cargo build --release || show_error_help
        sudo cp "target/release/$APP_NAME" "$BIN_DEST"
    elif command -v cargo &> /dev/null && command -v git &> /dev/null; then
        echo -e "${YELLOW}🔨 No local source found but Rust is present. Cloning and compiling in /tmp...${NC}"
        local tmp_dir=$(mktemp -d)
        git clone --depth 1 -b "$CURRENT_BRANCH" "$REPO_URL" "$tmp_dir"
        cd "$tmp_dir"
        cargo build --release || { cd - >/dev/null; rm -rf "$tmp_dir"; show_error_help; }
        sudo cp "target/release/$APP_NAME" "$BIN_DEST"
        cd - > /dev/null
        rm -rf "$tmp_dir"
    else
        echo -e "${YELLOW}📥 No Rust/Git found. Attempting to download pre-compiled binary...${NC}"
        sudo curl -fsSL "https://github.com/johancy96/XJemulator/releases/latest/download/xjemulator" -o "$BIN_DEST" || show_error_help
    fi
}

install_binary

if [ -f "$BIN_DEST" ]; then
    sudo chmod +x "$BIN_DEST"
fi

# 6. XDG Structure
mkdir -p ~/.config/$APP_NAME/profiles

echo -e "${GREEN}✅ XJemulator installed successfully!${NC}"
echo -e "${BLUE}ℹ️  Look for it in your application launcher or run: $APP_NAME${NC}"
