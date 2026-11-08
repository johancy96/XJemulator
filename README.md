# XJemulator 🎮

XJemulator is a modern graphical application for Linux that intercepts any generic controller at the kernel level (`evdev`) and emulates an official Xbox 360 controller locally (`uinput`). It is fully bilingual (Spanish/English) and compatible with Steam, Proton, and Wine.

---

## 🔧 Step 1: Install udev Rules (Required Once)

To allow Linux to create the virtual Xbox 360 controller without requiring root permissions, install the `udev` rules via `curl`:

```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/install-udev.sh | bash
```

Or locally from the repository root:
```bash
bash scripts/install-udev.sh
```

---

## 📦 Step 2: Run or Build XJemulator

### Option A: Portable `.AppImage`
To build the portable `.AppImage` package from source:
```bash
bash scripts/build-appimage.sh
```
This generates `XJemulator-1.0.0-x86_64.AppImage` in the project root.

### Option B: Direct Development (From Source)
1. Install system build dependencies:
   - **Debian / Ubuntu / Linux Mint:**
     ```bash
     sudo apt update && sudo apt install -y build-essential pkg-config libdbus-1-dev libudev-dev curl
     ```
   - **Fedora / RHEL:**
     ```bash
     sudo dnf install -y gcc pkgconf-pkg-config dbus-devel systemd-devel curl
     ```
   - **Arch Linux / Manjaro:**
     ```bash
     sudo pacman -Sy --needed base-devel dbus systemd pkgconf curl
     ```
2. Run directly with Cargo:
   ```bash
   cargo run --release
   ```
