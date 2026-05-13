#!/usr/bin/fish

# XJemulator - Modern Installation Script (Fish Version)
# Configura permisos, reglas udev y binarios sin necesidad de reiniciar.

# Detectar rama actual
# 1. Prioridad: Variable de entorno BRANCH
# 2. Si es local: git branch
# 3. Fallback: master (con aviso)
if test -n "$BRANCH"
    set CURRENT_BRANCH "$BRANCH"
else if git rev-parse --abbrev-ref HEAD >/dev/null 2>&1
    set CURRENT_BRANCH (git rev-parse --abbrev-ref HEAD)
else
    set CURRENT_BRANCH "master"
end

set -g REPO_RAW "https://raw.githubusercontent.com/johancy96/XJemulator/$CURRENT_BRANCH"
set -g BIN_DEST "/usr/local/bin/$APP_NAME"
set -g UDEV_RULE "99-$APP_NAME.rules"
set -g CONFIG_DIR "$HOME/.config/$APP_NAME/profiles"

# Colores (Fish Style)
set -l green (set_color green)
set -l blue (set_color blue)
set -l yellow (set_color yellow)
set -l red (set_color red)
set -l normal (set_color normal)

echo -e "$blue🚀 Iniciando instalacion profesional de XJemulator (Fish Edition)...$normal"
echo -e "$blue📍 Rama detectada: $yellow$CURRENT_BRANCH$normal"

# 1. Verificacion de requisitos
if not command -v sudo >/dev/null
    echo -e "$red❌ Error: Se requiere 'sudo' para configurar los permisos del sistema.$normal"
    exit 1
end

# 2. Configuración del Kernel (uinput)
echo -e "$yellow🔧 Configurando modulo uinput...$normal"
sudo modprobe uinput
echo "uinput" | sudo tee /etc/modules-load.d/$APP_NAME.conf > /dev/null

# 3. Instalacion de Reglas Udev
echo -e "$yellow📜 Instalando reglas udev (TAG+=uaccess)...$normal"
if test -f "udev/$UDEV_RULE"
    sudo cp "udev/$UDEV_RULE" "/etc/udev/rules.d/"
else
    curl -sSL "$REPO_RAW/udev/$UDEV_RULE" | sudo tee "/etc/udev/rules.d/$UDEV_RULE" > /dev/null
end

sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Permisos inmediatos
echo -e "$yellow🔑 Aplicando permisos de acceso inmediato...$normal"
if command -v setfacl >/dev/null
    sudo setfacl -m u:$USER:rw /dev/uinput
else
    sudo chmod 666 /dev/uinput
end

# 5. Icono y Lanzador
echo -e "$yellow🖼️ Instalando icono y lanzador en el sistema...$normal"
set -l ICON_PATH "/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
set -l DESKTOP_PATH "/usr/share/applications/$APP_NAME.desktop"

if test -f "assets/$APP_NAME.svg"
    sudo mkdir -p /usr/share/icons/hicolor/scalable/apps/
    sudo cp "assets/$APP_NAME.svg" "$ICON_PATH"
    sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor
end

if test -f "assets/$APP_NAME.desktop"
    sudo cp "assets/$APP_NAME.desktop" "$DESKTOP_PATH"
    sudo update-desktop-database /usr/share/applications
end

# 6. Estructura XDG
echo -e "$yellow📁 Creando directorios de configuracion en ~/.config...$normal"
mkdir -p "$CONFIG_DIR"

echo -e "$green✅ Instalacion completada con exito!$normal"
echo -e "$blueℹ️  Ya puedes ejecutar XJemulator sin necesidad de reiniciar el equipo.$normal"
