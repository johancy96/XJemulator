use evdevil::event::{Abs, Key};
use evdevil::uinput::AbsSetup;
use evdevil::AbsInfo;

/// Xbox 360 Wireless Controller USB IDs
pub const XBOX360_VENDOR_ID: u16 = 0x045e;
pub const XBOX360_PRODUCT_ID: u16 = 0x028e;

/// Xbox 360 Controller button mapping
pub const XBOX360_BUTTONS: &[Key] = &[
    Key::BTN_A,
    Key::BTN_B,
    Key::BTN_X,
    Key::BTN_Y,
    Key::BTN_TL,
    Key::BTN_TR,
    Key::BTN_SELECT,
    Key::BTN_START,
    Key::BTN_MODE,
    Key::BTN_THUMBL,
    Key::BTN_THUMBR,
];

/// Xbox 360 Controller absolute axes
pub fn xbox360_abs_axes() -> Vec<AbsSetup> {
    vec![
        AbsSetup::new(Abs::X, AbsInfo::new(-32768, 32767)),
        AbsSetup::new(Abs::Y, AbsInfo::new(-32768, 32767)),
        AbsSetup::new(Abs::Z, AbsInfo::new(0, 255)), // Left trigger
        AbsSetup::new(Abs::RX, AbsInfo::new(-32768, 32767)),
        AbsSetup::new(Abs::RY, AbsInfo::new(-32768, 32767)),
        AbsSetup::new(Abs::RZ, AbsInfo::new(0, 255)), // Right trigger
        AbsSetup::new(Abs::HAT0X, AbsInfo::new(-1, 1)),
        AbsSetup::new(Abs::HAT0Y, AbsInfo::new(-1, 1)),
    ]
}
