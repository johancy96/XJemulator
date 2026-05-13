#!/usr/bin/fish

# XJemulator - Modern Uninstaller Script (Fish Version)
# Reverts installation and cleans up the system professionally.

set -g APP_NAME "xjemulator"
set -l UDEV_RULE "/etc/udev/rules.d/99-$APP_NAME.rules"
set -l MODULE_CONF "/etc/modules-load.d/$APP_NAME.conf"
set -l ICON_PATH "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -l DESKTOP_PATH "/usr/share/applications/$APP_NAME.desktop"
set -l BIN_PATH "/usr/local/bin/$APP_NAME"
set -l CONFIG_DIR "$HOME/.config/$APP_NAME"

# Colors (Fish Style)
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

# 2. Remove Binary
echo -e "$yellow📂 Removing binary...$normal"
if test -f "$BIN_PATH"
    sudo rm -f "$BIN_PATH"
    echo "  ✔ $BIN_PATH removed."
end

# 3. Revert System Configuration
echo -e "$yellow🔧 Reverting system configuration...$normal"

# Udev Rules
if test -f "$UDEV_RULE"
    sudo rm -f "$UDEV_RULE"
    echo "  ✔ Udev rules removed."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
end

# Modules
if test -f "$MODULE_CONF"
    sudo rm -f "$MODULE_CONF"
    echo "  ✔ Kernel module configuration removed."
end

# Icons and Launchers
if test -f "$ICON_PATH"
    sudo rm -f "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
    echo "  ✔ Icon removed."
end

if test -f "$DESKTOP_PATH"
    sudo rm -f "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications
    echo "  ✔ .desktop launcher removed."
end

# 4. Cleanup User Data (Optional)
if contains -- "--full" $argv
    echo -e "$red⚠️  Full cleanup requested. Removing user configuration...$normal"
    if test -d "$CONFIG_DIR"
        rm -rf "$CONFIG_DIR"
        echo "  ✔ $CONFIG_DIR removed."
    end
else
    echo -e "$blueℹ️  User profiles kept at $CONFIG_DIR$normal"
    echo -e "$blueℹ️  Use '$yellow--full$blue' if you want to delete them as well.$normal"
end

echo -e "\n$green✅ XJemulator has been successfully uninstalled.$normal"
