# XJemulator 🎮

[🌍 English Version](file:///home/johancy/Proyectos/XJemulator/README_EN.md) / [🇪🇸 Versión en Español](file:///home/johancy/Proyectos/XJemulator/README.md)

XJemulator is a modern graphical application for Linux that allows you to intercept any generic controller and emulate an official Xbox 360 controller locally. It is fully bilingual (Spanish/English) and configured to work at the kernel level using `uinput` and `udev` utilities.

---

## 🚀 Fast Installation (Recommended)

If you just want to install the program without downloading the entire repository, run this command in your terminal:

**Standard Installation (Master):**
```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/install.sh | bash
```

**Specific Branch Installation (Remote):**
If you want to test a development branch (e.g., `updates-testing`), use this command to ensure all assets are pulled from the correct branch:
```bash
BRANCH=updates-testing bash -c "$(curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/updates-testing/scripts/install.sh)"
```

**For Fish:**
```fish
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/install.fish | fish
```

*For Fish specific branches:*
```fish
set -x BRANCH updates-testing; curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/updates-testing/scripts/install.fish | fish
```

---

## 🗑 Fast Uninstallation

If you want to completely remove XJemulator from your system:

**Standard Uninstallation:**
```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/uninstall.sh | bash
```

**From Specific Branch:**
```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/updates-testing/scripts/uninstall.sh | bash
```
> *Note: To also remove your profiles and local configurations, add `--full` at the end: `| bash -s -- --full`*

---

---

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

---

## 💻 Direct Development (Do not generate installers)

If you are not interested in installing it but rather opening the visualizer locally or modifying its content:
```bash
# Run application skipping debugging
cargo run --release
```

---

## 🖥 Local Installation (From Source)

If you have cloned the repository and want to install the program natively on your system:
```bash
# Run the installation script from the project root
bash scripts/install.sh
```

To uninstall:
```bash
bash scripts/uninstall.sh
```
