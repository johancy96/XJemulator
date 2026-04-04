# XJemulator 🎮

[🌍 English Version](file:///home/johancy/Proyectos/XJemulator/README_EN.md) / [🇪🇸 Versión en Español](file:///home/johancy/Proyectos/XJemulator/README.md)

XJemulator is a modern graphical application for Linux that allows you to intercept any generic controller and emulate an official Xbox 360 controller locally. It is fully bilingual (Spanish/English) and configured to work at the kernel level using `uinput` and `udev` utilities.

---

## 🛠 Installation and Build Guide

We have integrated automatic tools to make compiling XJemulator and generating installers directly for your distribution a hassle-free process.

### Step 1: Install System Dependencies
Before you can build the application on your PC, you must have your system's base build tools.

**Select your distribution and run the command in the terminal:**

- 🟠 **Debian / Ubuntu / Linux Mint / Pop!_OS:**
  ```bash
  sudo apt update
  sudo apt install -y build-essential curl
  ```
- 🔵 **Fedora / RHEL / CentOS:**
  ```bash
  sudo dnf groupinstall "Development Tools"
  sudo dnf install curl
  ```
- 🟣 **Arch Linux / Manjaro / EndeavourOS:**
  ```bash
  sudo pacman -Sy base-devel curl
  ```

### Step 2: Programming Environment (Rust)
We will rely on Rust to compile. This command will install the compiler toolchain on your machine without requiring administrator permissions:
```bash
# Run and install with default options (by pressing the Enter key)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload the terminal to enable the 'cargo' base work command
source "$HOME/.cargo/env"
```

---

## 🚀 Automatic Packaging (Building the installers!)

Once you have cloned or downloaded this repository to your computer, open the terminal inside this same folder.

Depending on the base format of your operating system, run the packaging script that will automate all the dirty work for you:

### 📦 Option A: Debian Systems (.deb)
Recommended if you want to achieve maximum desktop integration on Ubuntu, Mint, or Debian.

1. Run the Bash file:
   ```bash
   ./dist/build_packages.sh
   ```
2. **Output:** A packaged `.deb` file will be magically generated in the internal path `target/debian/`.
3. **Installation:** To install this generator, double-click it in the file manager, or run `sudo dpkg -i target/debian/*.deb`.

### 📦 Option B: Fedora and RedHat Systems (.rpm)
Just like in Debian, you can use the same base script.
1. Run:
   ```bash
   ./dist/build_packages.sh
   ```
2. **Output:** You can find the resulting `.rpm` file waiting in `target/generate-rpm/`.
3. **Installation:** Double-click it or use `sudo dnf localinstall target/generate-rpm/*.rpm` in the console.

### 🌍 Option C: Portable Format (.AppImage)
Ideal if you use heterogeneous distributions, Immutable OS (like Fedora Silverblue / Kinoite), or simply prefer to take the program on a Pendrive to another PC with Linux without having to recompile anything.
1. Run the AppImage generator:
   ```bash
   ./dist/build_appimage.sh
   ```
2. **Output and Installation:** It will download a LinuxDeploy AppImage and create the encapsulated application called **`XJemulator-x86_64.AppImage`** exactly where you are. Grant permissions by right-clicking > Properties > "Allow executing as a program" (Or via terminal `chmod +x XJemulator*.AppImage`) and open it in any modern Linux with a double-click.

### 🐧 Option D: Arch Linux and AUR
Thanks to the included `PKGBUILD` file, Arch users don't need to compile via external packages; they can do it automatically via makepkg.
```bash
# Access the folder with the file
cd dist

# Download libraries, package and install it natively (pacman)
makepkg -si
```

---

## 💻 Direct Development (Do not generate installers)

If you are not interested in installing it but rather opening the visualizer locally or modifying its content:
```bash
# Run application skipping debugging
cargo run --release
```

---

## 🖥 Application Launcher Integration (Manual Installation)

If you decided to skip the automatic installers (`.deb` / `.rpm`) and used direct compilation or the portable `.AppImage` format, you might want XJemulator to appear alongside your other programs in the Linux desktop menu (e.g., GNOME, Plasma, Cinnamon). For this, we have attached the native links in the `assets/` folder.

1. **Install the Icon:**
   ```bash
   sudo cp assets/xjemulator.svg /usr/share/icons/hicolor/scalable/apps/
   ```
2. **Copy the .desktop file to the Linux menu path:**
   ```bash
   # If you want it to appear only for your user:
   mkdir -p ~/.local/share/applications/
   cp assets/xjemulator.desktop ~/.local/share/applications/
   
   # Or if you prefer to install the menu for all PC profiles:
   sudo cp assets/xjemulator.desktop /usr/share/applications/
   ```

> *Note: Open the copied `.desktop` file with a text editor and make sure to modify the line that says `Exec=xjemulator` so that it points directly to the path of your new executable (e.g., `Exec=/home/user/downloads/XJemulator-x86_64.AppImage`).*
