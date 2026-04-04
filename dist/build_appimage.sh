#!/bin/bash
set -e

cd "$(dirname "$0")/.."

APP="XJemulator"
LOWER_APP="xjemulator"

echo "==> Building release binary..."
cargo build --release

echo "==> Preparing AppDir..."
APPDIR="AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"

cp "target/release/$LOWER_APP" "$APPDIR/usr/bin/"
cp "assets/$LOWER_APP.desktop" "$APPDIR/usr/share/applications/"
cp "assets/$LOWER_APP.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/"
# Fallback icon root location required by some AppImages
cp "assets/$LOWER_APP.svg" "$APPDIR/"

echo "==> Downloading linuxdeploy..."
if [ ! -f "linuxdeploy-x86_64.AppImage" ]; then
    wget -q https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
fi

echo "==> Assembling AppImage..."
export ARCH=x86_64
./linuxdeploy-x86_64.AppImage --appdir "$APPDIR" --output appimage -d "assets/$LOWER_APP.desktop" -i "assets/$LOWER_APP.svg"

rm -rf "$APPDIR"
echo "==> Done. AppImage generated!"
