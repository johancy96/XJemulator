#!/usr/bin/fish

# XJemulator - Modern Uninstaller Script (Fish Version)
# Revierte la instalacion# --- Configuracion Modular ---
set -g APP_NAME "xjemulator"

# Detección de rama (Por simetría con install.fish)
if test -n "$BRANCH"
    set CURRENT_BRANCH "$BRANCH"
else if git rev-parse --abbrev-ref HEAD >/dev/null 2>&1
    set CURRENT_BRANCH (git rev-parse --abbrev-ref HEAD)
else
    set CURRENT_BRANCH "master"
end

set -l UDEV_RULE "/etc/udev/rules.d/99-$APP_NAME.rules"
set -l MODULE_CONF "/etc/modules-load.d/$APP_NAME.conf"
set -l ICON_PATH "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -l DESKTOP_PATH "/usr/share/applications/$APP_NAME.desktop"
set -l BIN_PATH "/usr/local/bin/$APP_NAME"
set -l CONFIG_DIR "$HOME/.config/$APP_NAME"

# Colores (Fish Style)
set -l red (set_color red)
set -l green (set_color green)
set -l yellow (set_color yellow)
set -l blue (set_color blue)
set -l normal (set_color normal)

echo -e "$blue🗑️ Iniciando desinstalacion de XJemulator (Fish Edition)...$normal"

# 1. Verificacion de sudo
if not command -v sudo >/dev/null
    echo -e "$red❌ Error: Se requiere 'sudo' para eliminar archivos del sistema.$normal"
    exit 1
end

# 2. Eliminar Binario
echo -e "$yellow📂 Eliminando binario...$normal"
if test -f "$BIN_PATH"
    sudo rm -f "$BIN_PATH"
    echo "  ✔ $BIN_PATH eliminado."
end

# 3. Revertir Configuracion del Sistema
echo -e "$yellow🔧 Revirtiendo configuracion del sistema...$normal"

# Reglas Udev
if test -f "$UDEV_RULE"
    sudo rm -f "$UDEV_RULE"
    echo "  ✔ Reglas udev eliminadas."
    sudo udevadm control --reload-rules
    sudo udevadm trigger
end

# Modulos
if test -f "$MODULE_CONF"
    sudo rm -f "$MODULE_CONF"
    echo "  ✔ Configuracion de modulos eliminada."
end

# Iconos y Lanzadores
if test -f "$ICON_PATH"
    sudo rm -f "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
    echo "  ✔ Icono eliminado."
end

if test -f "$DESKTOP_PATH"
    sudo rm -f "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications
    echo "  ✔ Lanzador .desktop eliminado."
end

# 4. Limpieza de Datos de Usuario (Opcional)
if contains -- "--full" $argv
    echo -e "$red⚠️  Limpieza completa solicitada. Eliminando configuracion de usuario...$normal"
    if test -d "$CONFIG_DIR"
        rm -rf "$CONFIG_DIR"
        echo "  ✔ $CONFIG_DIR eliminado."
    end
else
    echo -e "$blueℹ️  Se han conservado tus perfiles en $CONFIG_DIR$normal"
    echo -e "$blueℹ️  Usa '$yellow--full$blue' si deseas borrarlos tambien.$normal"
end

echo -e "\n$green✅ XJemulator ha sido desinstalado correctamente.$normal"
