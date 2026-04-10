# XJemulator - Official User Guide 📖

[🌍 English Version](file:///home/johancy/Proyectos/XJemulator/GUIDE_EN.md) / [🇪🇸 Versión en Español](file:///home/johancy/Proyectos/XJemulator/GUIDE.md)

Welcome to XJemulator! A graphical generic controller to Xbox 360 emulator for Linux. Here you will learn to detect, calibrate and emulate your gamepads seamlessly for perfect compatibility with Steam, Proton and Wine.

---

## ⚠️ Zero-Step: udev and Permissions
To allow Linux to read your physical controllers and instantiate a new "Virtual Xbox 360 Controller", it needs access to hardware modules.

1. **Add your user to the `input` group**:
   Run this in your terminal and **restart your computer / log out!**
   ```bash
   sudo usermod -aG input $USER
   ```
2. **Verify udev rules**:
   Normally, when installing `XJemulator`, the application configures these rules automatically. 
   **If the automatic installation failed** or you run a manual copy, the application will display a red warning on-screen. In that case, run the following in your terminal to install them manually:
   ```bash
   sudo sh -c 'curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/udev/99-xjemulator.rules > /etc/udev/rules.d/99-xjemulator.rules'
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

---

## 🎮 Short Tutorial: How to Calibrate your First Controller

Calibration "teaches" the application which button of your generic controller translates to which button of the Xbox 360 standard. **You will only have to do this once for each model of controller you have.**

### 1. Select your Controller (Left Panel)
Connect your generic controller via USB or Bluetooth.
On the left side, you'll find the **"🔌 Detected Controllers"** area. Find your controller in the list and press the rectangle in the list to select it. In the central RAW Monitor, codes should start jumping if you press the buttons.

### 2. Start the Calibration Assistant
Once the controller is selected in the left menu (and you make sure it's not already emulating via the red Stop button), press the button located under the central monitor that says **"▶ Start calibration"**.

### 3. Follow the On-Screen Instructions
The system will guide you button by button:
- **Calibrating Buttons:** The name of a button will appear in yellow (e.g., "Button A - The bottom button (green/blue)"). Simply **press it hard** on your controller consecutively. The interface will detect it, beep, and move on to the next one on its own.
  > *Tip: If your controller doesn't have an advanced button (like a "HOME" or "Guide" Button), click the on-screen hyperlink "Skip (Do Not Map)"*.
- **Calibrating Axes (Sticks and Triggers):** Sticks require precision. Follow the textual instruction; if it says "Left Stick -> push DOWN", push and hold it until it stops.
  Then wait for the instruction **"✅ Detected! Release the controller and return to the center..."**. Only then can you loosen the pressure so that the emulator detects the neutral spring point.
  > *After detecting the center, sometimes the system will suggest "¿Invert direction?". Just press "Accept and Continue" unless the joystick behaves strangely in a game.*

### 4. Save your Profile
After finishing all the axes, you'll get a **"Calibration Completed!"** message.
Type the name by which you'll identify your control (e.g., `ps3_blue_controller`) in the top box and press the **💾 Save Profile and Finish** button. This profile will be saved and will appear forever in your right side panel.

---

## 🚀 Daily Use: Activating Emulation

Time to play! Once you've performed the configuration described above, it only takes **one click** each day you want to play:
1. Connect your controller and open XJemulator.
2. Find it in the left list, locate its box, and press the **"▶ (Play) Emulator"** button.

A green indicator will start rotating over the controller profile in the application interface, indicating it as "Active". While the app remains open minimized in your Linux, your computer will believe it has a genuine `Microsoft X-Box 360 pad` connected via USB. Open Steam and enjoy!

---

## ⚕️ Troubleshooting

### The emulator does not create the virtual device (Proton does not respond)
- Make sure you have completed and restarted after following Step 0 ("input group").
- Check the static bar at the top; it should proudly say `✓ udev installed`.

### Proton / Steam / Wine do not detect the Xbox 360 even when emulating
- Go to **Steam Big Picture mode > Settings > Controller** and check if Xbox Controller support is active.
- Disable "Generic Controllers" support in the Steam menu to avoid double-layer interference with the simulated controller. Avoid pressing commands while loading.

### My characters in X game "walk alone" (Extreme Drift / Phantom inputs)
- This occurs when the RAW reading of the controller rests outside limits. Re-calibrate the controller by stopping carefully where the assistant asks *"Release the controller and return to center"*, giving the controller its original neutral point.
