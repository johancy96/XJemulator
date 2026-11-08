# XJemulator - Official User Guide 📖

Welcome to XJemulator! A graphical generic controller to Xbox 360 emulator for Linux. Here you will learn to detect, calibrate and emulate your gamepads seamlessly for compatibility with Steam, Proton and Wine.

---

## ⚠️ Zero-Step: udev and Permissions
To allow Linux to read your physical controllers and instantiate a virtual Xbox 360 controller, configure the `udev` rules and `uinput` permissions once:

**Via `curl`:**
```bash
curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/scripts/install-udev.sh | bash
```

**Or from the local repository:**
```bash
bash scripts/install-udev.sh
```
> *Note: If your user was added to the `input` group for the first time, log out and log back in.*

---

## 🎮 Short Tutorial: How to Calibrate your First Controller

Calibration teaches the application which button and axis of your generic controller maps to the Xbox 360 standard. **You only need to do this once per controller model.**

### 1. Select your Controller (Left Panel)
Connect your generic controller via USB or Bluetooth.
On the left side, locate your controller under **"Detected gamepads"** and click it to select it.

### 2. Start the Calibration Assistant
With your controller selected (and not currently emulating), click the **"▶ Start calibration"** button in the center panel.

### 3. Follow the On-Screen Instructions
- **Step 1 — Buttons:** Press the highlighted button on your controller. If your controller lacks a specific button (such as the Guide/Xbox button), click **"⏭ Skip"**.
- **Step 2 — Axes (Sticks, Triggers, D-Pad):** Push and hold the requested stick direction or trigger until the confirmation bar reaches `100%`, then release it to return to its neutral resting point. You can also click **"⏭ Skip"** for any axis your controller does not have.

### 4. Name and Save your Profile
In the final review screen, enter a descriptive name for your profile (it defaults to your controller's name) and press **"💾 SAVE PROFILE"**.

---

## 🚀 Daily Use: Activating Emulation

1. Connect your controller and open XJemulator.
2. Select your controller in the left list (your saved profile will auto-load by `VID:PID`).
3. Click the **Play (`▶`)** button on the controller card.

While active, your system exposes an official `Microsoft X-Box 360 pad` (`045e:028e`) via USB `uinput` and hides the generic physical device (`EVIOCGRAB`) to prevent double input in games.
