use evdevil::event::InputEvent;
use evdevil::uinput::UinputDevice;
use evdevil::{Bus, InputId};
use tracing::info;

use crate::xbox_descriptor;

/// Name that SDL2/Proton expects for Xbox 360 controllers.
/// Must match exactly for SDL2 to auto-detect as XInput device.
const XBOX360_DEVICE_NAME: &str = "Microsoft X-Box 360 pad";

/// Time to wait after device creation for SDL2/udev detection
const DETECTION_DELAY_MS: u64 = 1000;

/// Virtual Xbox 360 controller created via uinput.
///
/// This device spoofs the Xbox 360 Wireless Controller VID/PID and
/// exact device name so that SDL2 (used by Proton/Wine) detects it
/// as a native Xbox 360 controller and exposes it via XInput.
pub struct VirtualXbox360 {
    device: UinputDevice,
}

impl VirtualXbox360 {
    /// Create a new virtual Xbox 360 controller.
    ///
    /// The device will be created with:
    /// - VID: 0x045e (Microsoft)
    /// - PID: 0x028e (Xbox 360 Wireless Controller)
    /// - Bus: USB
    /// - Name: "Microsoft X-Box 360 pad"
    ///
    /// After creation, a delay is applied to allow SDL2 and udev
    /// to detect and configure the device before we start sending events.
    pub fn new() -> Result<Self, std::io::Error> {
        let axes = xbox_descriptor::xbox360_abs_axes();

        // Xbox 360 input ID: spoof VID/PID so SDL2 recognizes us
        let xbox_id = InputId::new(
            Bus::USB,
            xbox_descriptor::XBOX360_VENDOR_ID,
            xbox_descriptor::XBOX360_PRODUCT_ID,
            0x0114, // bcdXbox360 - Xbox 360 HID version
        );

        let device = UinputDevice::builder()?
            .with_device_id(xbox_id)?
            .with_keys(xbox_descriptor::XBOX360_BUTTONS.iter().copied())?
            .with_abs_axes(axes)?
            .build(XBOX360_DEVICE_NAME)?;

        info!(
            "Controlador virtual Xbox 360 creado (VID:PID={:04x}:{:04x})",
            xbox_descriptor::XBOX360_VENDOR_ID,
            xbox_descriptor::XBOX360_PRODUCT_ID
        );

        // Wait for SDL2/udev to detect and configure the device.
        // Without this delay, Proton/Wine may miss the device entirely.
        info!("Esperando deteccion de SDL2 ({}ms)...", DETECTION_DELAY_MS);
        std::thread::sleep(std::time::Duration::from_millis(DETECTION_DELAY_MS));

        // Check if device node exists
        let sysname = device
            .sysname()
            .map(|s: std::ffi::OsString| s.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());

        info!("Dispositivo listo: /sys/devices/virtual/input/{}", sysname);
        info!(
            "Verifica con: evtest /dev/input/ (busca '{}')",
            XBOX360_DEVICE_NAME
        );

        Ok(Self { device })
    }

    /// Write a batch of events at once (more efficient).
    /// The kernel automatically appends SYN_REPORT after the batch.
    pub fn write_batch(&self, events: &[InputEvent]) -> Result<(), std::io::Error> {
        self.device.write(events)?;
        Ok(())
    }
}
