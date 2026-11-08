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

## 📦 Step 2: Portable `.AppImage`

### Run the `.AppImage`
Download the `.AppImage` from Releases, grant execution permissions, and run it:
```bash
chmod +x XJemulator-1.0.0-x86_64.AppImage
./XJemulator-1.0.0-x86_64.AppImage
```

### Build the `.AppImage` Locally
To generate `XJemulator-1.0.0-x86_64.AppImage` in the project root:
```bash
bash scripts/build-appimage.sh
```
