#!/usr/bin/env bash
# ==============================================================================
# XJemulator — Generador de Paquete Portable (.AppImage)
# ==============================================================================
# Compila XJemulator en modo release, construye la estructura estándar AppDir
# y empaqueta el binario autocontenido `XJemulator-x86_64.AppImage`.
#
# Uso:
#   bash scripts/build-appimage.sh
# ==============================================================================

set -euo pipefail

APP_NAME="xjemulator"
APP_TITLE="XJemulator"
ARCH="${ARCH:-x86_64}"

# Posicionarse siempre en la raíz del repositorio
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$PROJECT_ROOT"

VERSION="$(grep -E '^version\s*=' Cargo.toml | head -n1 | cut -d'"' -f2)"
APPDIR="${PROJECT_ROOT}/target/AppDir"
OUTPUT_APPIMAGE="${PROJECT_ROOT}/${APP_TITLE}-${VERSION}-${ARCH}.AppImage"
APPIMAGETOOL_CACHE="${PROJECT_ROOT}/target/appimagetool-${ARCH}.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${ARCH}.AppImage"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 Iniciando construcción de ${APP_TITLE} v${VERSION} (${ARCH}.AppImage)...${NC}"

# 1. Verificar dependencias básicas
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}❌ Error: 'cargo' no está instalado o no se encuentra en el PATH.${NC}"
    exit 1
fi

# 2. Compilar el proyecto en modo Release
echo -e "${YELLOW}🔨 Compilando binario en modo release...${NC}"
cargo build --release

BIN_SOURCE="${PROJECT_ROOT}/target/release/${APP_NAME}"
if [ ! -f "$BIN_SOURCE" ]; then
    echo -e "${RED}❌ Error: No se encontró el binario compilado en ${BIN_SOURCE}${NC}"
    exit 1
fi

# Optimizar tamaño del binario eliminando símbolos de depuración si strip está disponible
if command -v strip &>/dev/null; then
    strip -s "$BIN_SOURCE" || true
fi

# 3. Preparar estructura limpia de AppDir
echo -e "${YELLOW}📂 Creando estructura AppDir en ${APPDIR}...${NC}"
rm -rf "$APPDIR"
mkdir -p \
    "${APPDIR}/usr/bin" \
    "${APPDIR}/usr/share/applications" \
    "${APPDIR}/usr/share/icons/hicolor/scalable/apps" \
    "${APPDIR}/usr/share/udev/rules.d"

# 4. Copiar binario, iconos, reglas udev y lanzador .desktop
cp "$BIN_SOURCE" "${APPDIR}/usr/bin/${APP_NAME}"
chmod 755 "${APPDIR}/usr/bin/${APP_NAME}"

cp "${PROJECT_ROOT}/assets/${APP_NAME}.desktop" "${APPDIR}/usr/share/applications/${APP_NAME}.desktop"
cp "${PROJECT_ROOT}/assets/${APP_NAME}.desktop" "${APPDIR}/${APP_NAME}.desktop"

cp "${PROJECT_ROOT}/assets/${APP_NAME}.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/${APP_NAME}.svg"
cp "${PROJECT_ROOT}/assets/${APP_NAME}.svg" "${APPDIR}/${APP_NAME}.svg"
ln -sf "${APP_NAME}.svg" "${APPDIR}/.DirIcon"

cp "${PROJECT_ROOT}/udev/99-${APP_NAME}.rules" "${APPDIR}/usr/share/udev/rules.d/99-${APP_NAME}.rules"

# 5. Crear el punto de entrada AppRun
cat > "${APPDIR}/AppRun" <<'EOF'
#!/usr/bin/env bash
set -e
SELF_DIR="$(dirname "$(readlink -f "${0}")")"
export PATH="${SELF_DIR}/usr/bin:${PATH}"
export XDG_DATA_DIRS="${SELF_DIR}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${SELF_DIR}/usr/bin/xjemulator" "$@"
EOF
chmod 755 "${APPDIR}/AppRun"

# 6. Localizar o descargar appimagetool
if command -v appimagetool &>/dev/null; then
    APPIMAGETOOL_CMD="appimagetool"
else
    if [ ! -f "$APPIMAGETOOL_CACHE" ]; then
        echo -e "${YELLOW}📥 Descargando appimagetool (${ARCH}) en ${APPIMAGETOOL_CACHE}...${NC}"
        curl -fsSL "$APPIMAGETOOL_URL" -o "$APPIMAGETOOL_CACHE"
        chmod +x "$APPIMAGETOOL_CACHE"
    fi
    # Usar --appimage-extract-and-run para funcionar incluso sin FUSE (ej. dentro de contenedores)
    APPIMAGETOOL_CMD="${APPIMAGETOOL_CACHE} --appimage-extract-and-run"
fi

# 7. Generar el archivo .AppImage final
echo -e "${YELLOW}📦 Empaquetando ${OUTPUT_APPIMAGE}...${NC}"
ARCH="$ARCH" $APPIMAGETOOL_CMD "$APPDIR" "$OUTPUT_APPIMAGE"
chmod +x "$OUTPUT_APPIMAGE"

echo -e "${GREEN}✅ AppImage generado con éxito: ${OUTPUT_APPIMAGE}${NC}"
