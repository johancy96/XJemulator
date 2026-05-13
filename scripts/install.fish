#!/usr/bin/fish

# XJemulator - Universal Installation Script (Fish Hybrid Version)
# Configures permissions, udev rules, icons and launchers with remote auto-compilation.

set -g APP_NAME "xjemulator"
set -g CURRENT_BRANCH "master"
if set -q BRANCH
    set CURRENT_BRANCH $BRANCH
end

set -g REPO_URL "https://github.com/johancy96/XJemulator.git"
set -g REPO_RAW "https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
set -g BIN_DEST "/usr/local/bin/$APP_NAME"
set -g UDEV_DEST "/etc/udev/rules.d/99-$APP_NAME.rules"
set -g ICON_DEST "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -g DESKTOP_DEST "/usr/share/applications/$APP_NAME.desktop"

# Colors
set -l green (set_color green)
set -l blue (set_color blue)
set -l yellow (set_color yellow)
set -l red (set_color red)
set -l normal (set_color normal)

function show_error_help
    set -l green (set_color green)
    set -l blue (set_color blue)
    set -l yellow (set_color yellow)
    set -l red (set_color red)
    set -l normal (set_color normal)

    echo -e "\n$red❌ Error: Could not obtain XJemulator binary.$normal"
    echo -e "$yellowTo compile from source, you need to install dependencies:$normal"
    echo -e "  - Rust & Cargo (https://rustup.rs)"
    echo -e "  - DBus development files"
    echo -e "  - GCC / Build Essentials"
    echo -e "\n$blueSuggested commands:$normal"
    echo -e "  Arch Linux: $green"sudo pacman -S rust dbus pkgconf base-devel"$normal"
    echo -e "  Ubuntu/Debian: $green"sudo apt install cargo libdbus-1-dev pkg-config build-essential"$normal"
    echo -e "  Fedora: $green"sudo dnf install cargo dbus-devel pkgconf-pkg-config development-tools"$normal"
    exit 1
end

echo -e "$blue🚀 Starting XJemulator Universal Installation (Fish Edition)...$normal"
echo -e "$blue📍 Detected branch: $yellow$CURRENT_BRANCH$normal"

# 1. Sudo check
if not command -v sudo >/dev/null
    echo -e "$red❌ Error: 'sudo' is required for installation.$normal"
    exit 1
end

# 2. Secure download function
function download_asset
    set -l remote_path $argv[1]
    set -l local_path $argv[2]
    set -l dest $argv[3]
    
    echo -e "$yellow📥 Processing $remote_path...$normal"
    if test -f "$local_path"
        sudo cp "$local_path" "$dest"
    else
        sudo curl -fsSL "$REPO_RAW/$remote_path" -o "$dest"
        if test $status -ne 0
            echo -e "$red❌ Error downloading $remote_path$normal"
            exit 1
        end
    end
end

# 3. Configure Hardware
echo -e "$yellow🔧 Configuring hardware (uinput)...$normal"
sudo modprobe uinput
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null
download_asset "udev/99-xjemulator.rules" "udev/99-xjemulator.rules" "$UDEV_DEST"
sudo udevadm control --reload-rules; and sudo udevadm trigger
sudo setfacl -m u:$USER:rw /dev/uinput; or sudo chmod 666 /dev/uinput

# 4. Icons and Launchers
download_asset "assets/xjemulator.svg" "assets/xjemulator.svg" "$ICON_DEST"
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
download_asset "assets/xjemulator.desktop" "assets/xjemulator.desktop" "$DESKTOP_DEST"
sudo update-desktop-database /usr/share/applications

# 5. Binary Management (Hybrid)
if test -f "Cargo.toml"
    echo -e "$yellow🔨 Compiling locally...$normal"
    cargo build --release; and sudo cp "target/release/$APP_NAME" "$BIN_DEST"; or show_error_help
else if command -v cargo >/dev/null; and command -v git >/dev/null
    echo -e "$yellow🔨 No local source found but Rust is present. Cloning and compiling in /tmp...$normal"
    set -l tmp_dir (mktemp -d)
    git clone --depth 1 -b "$CURRENT_BRANCH" "$REPO_URL" "$tmp_dir"
    pushd "$tmp_dir"
    cargo build --release; and sudo cp "target/release/$APP_NAME" "$BIN_DEST"; or begin; popd; rm -rf "$tmp_dir"; show_error_help; end
    popd
    rm -rf "$tmp_dir"
else
    echo -e "$yellow📥 No Rust/Git found. Attempting to download pre-compiled binary...$normal"
    sudo curl -fsSL "https://github.com/johancy96/XJemulator/releases/latest/download/xjemulator" -o "$BIN_DEST"
    if test $status -ne 0
        show_error_help
    end
end

if test -f "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
end

# 6. XDG Structure
mkdir -p ~/.config/$APP_NAME/profiles

echo -e "$green✅ XJemulator installed successfully!$normal"
echo -e "$blueℹ️  Look for it in your application launcher or run: $APP_NAME$normal"
