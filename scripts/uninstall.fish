#!/usr/bin/fish

# XJemulator - Modern Uninstaller Script (Fish Edition - PRODUCTION READY)
# Reverts installation and cleans up the system professionally.

set -g APP_NAME "xjemulator"

# --- Configuration (Must match install.fish) ---
set -l BIN_PATH "/usr/local/bin/$APP_NAME"
set -l UDEV_PATH "/etc/udev/rules.d/99-$APP_NAME.rules"
set -l MODULE_PATH "/etc/modules-load.d/$APP_NAME.conf"
set -l ICON_PATH "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -l DESKTOP_PATH "/usr/share/applications/$APP_NAME.desktop"
set -l CONFIG_DIR "$HOME/.config/$APP_NAME"

# Colors
set -l red (set_color red)
set -l green (set_color green)
set -l yellow (set_color yellow)
set -l blue (set_color blue)
set -l normal (set_color normal)

echo -e "$blue🗑️ Starting XJemulator uninstallation (Fish Edition)...$normal"

# 1. Sudo check
if not command -v sudo >/dev/null
    echo -e "$red❌ Error: 'sudo' is required to remove system files.$normal"
    exit 1
end

# 2. Function for safe removal
function remove_file
    set -l file_path $argv[1]
    set -l description $argv[2]
    
    set -l green (set_color green)
    set -l yellow (set_color yellow)
    set -l normal (set_color normal)

    if test -f "$file_path"
        echo -e "$yellow📂 Removing $description...$normal"
        sudo rm -f "$file_path"
        echo -e "  $green✔ Removed: $file_path$normal"
    end
end

# 3. System Cleanup
remove_file "$BIN_PATH" "binary"
remove_file "$UDEV_PATH" "udev rules"
remove_file "$MODULE_PATH" "kernel module config"
remove_file "$ICON_PATH" "application icon"
remove_file "$DESKTOP_PATH" "desktop launcher"

# Reload system databases
echo -e "$yellow🔄 Refreshing system databases...$normal"
sudo udevadm control --reload-rules; and sudo udevadm trigger
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
sudo update-desktop-database /usr/share/applications

# 4. Optional User Data Cleanup
if contains -- "--full" $argv
    echo -e "$red⚠️  Full cleanup requested. Removing user data...$normal"
    if test -d "$CONFIG_DIR"
        rm -rf "$CONFIG_DIR"
        echo -e "  $green✔ Removed: $CONFIG_DIR$normal"
    end
else
    echo -e "\n$blueℹ️  User profiles kept at $CONFIG_DIR$normal"
    echo -e "$blueℹ️  Run with '$yellow--full$blue' to remove them as well.$normal"
end

echo -e "\n$green✅ XJemulator has been successfully uninstalled.$normal"
