use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Default)]
pub(crate) struct RawCapture {
    pub key_queue: VecDeque<String>,
    pub pressed_keys: HashSet<String>,
    pub axis_values: HashMap<String, i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CalibStep {
    Idle,
    Buttons(usize),
    Axes(usize),
    Review,
}

pub(crate) struct BtnSlot {
    pub xbox_key: &'static str,
    pub label: String,
    pub hint: String,
    pub source: Option<String>,
}

pub(crate) struct AxisSlot {
    pub xbox_axis: &'static str,
    pub label: String,
    pub direction_label: String,
    pub positive_expected: bool,
    pub source: Option<String>,
    pub invert: bool,
    pub max_val: Option<i32>,
}

pub(crate) fn default_btn_slots(lang: &crate::i18n::Lang) -> Vec<BtnSlot> {
    vec![
        BtnSlot {
            xbox_key: "BTN_A",
            label: crate::i18n::t(lang, "btn_a").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_a").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_B",
            label: crate::i18n::t(lang, "btn_b").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_b").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_X",
            label: crate::i18n::t(lang, "btn_x").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_x").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_Y",
            label: crate::i18n::t(lang, "btn_y").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_y").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_TL",
            label: crate::i18n::t(lang, "btn_lb").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_lb").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_TR",
            label: crate::i18n::t(lang, "btn_rb").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_rb").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_SELECT",
            label: crate::i18n::t(lang, "btn_back").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_back").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_START",
            label: crate::i18n::t(lang, "btn_start").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_start").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_MODE",
            label: crate::i18n::t(lang, "btn_guide").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_guide").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_THUMBL",
            label: crate::i18n::t(lang, "btn_l3").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_l3").to_string(),
            source: None,
        },
        BtnSlot {
            xbox_key: "BTN_THUMBR",
            label: crate::i18n::t(lang, "btn_r3").to_string(),
            hint: crate::i18n::t(lang, "hint_btn_r3").to_string(),
            source: None,
        },
    ]
}

pub(crate) fn default_axis_slots(lang: &crate::i18n::Lang) -> Vec<AxisSlot> {
    vec![
        // ── Palanca Izquierda ─ 4 direcciones ──────────────────────────────
        AxisSlot {
            xbox_axis: "ABS_X",
            label: crate::i18n::t(lang, "axis_left_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_right").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_X",
            label: crate::i18n::t(lang, "axis_left_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_left").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_Y",
            label: crate::i18n::t(lang, "axis_left_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_down").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_Y",
            label: crate::i18n::t(lang, "axis_left_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_up").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
        // ── Gatillo Izquierdo ───────────────────────────────────────────────
        AxisSlot {
            xbox_axis: "ABS_Z",
            label: crate::i18n::t(lang, "axis_left_trigger").to_string(),
            direction_label: crate::i18n::t(lang, "dir_press").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        // ── Palanca Derecha ─ 4 direcciones ────────────────────────────────
        AxisSlot {
            xbox_axis: "ABS_RX",
            label: crate::i18n::t(lang, "axis_right_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_right").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_RX",
            label: crate::i18n::t(lang, "axis_right_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_left").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_RY",
            label: crate::i18n::t(lang, "axis_right_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_down").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_RY",
            label: crate::i18n::t(lang, "axis_right_stick").to_string(),
            direction_label: crate::i18n::t(lang, "dir_up").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
        // ── Gatillo Derecho ─────────────────────────────────────────────────
        AxisSlot {
            xbox_axis: "ABS_RZ",
            label: crate::i18n::t(lang, "axis_right_trigger").to_string(),
            direction_label: crate::i18n::t(lang, "dir_press").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        // ── Cruz Direccional (D-Pad) ─ 4 direcciones ───────────────────────
        AxisSlot {
            xbox_axis: "ABS_HAT0X",
            label: crate::i18n::t(lang, "axis_dpad").to_string(),
            direction_label: crate::i18n::t(lang, "dir_right").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_HAT0X",
            label: crate::i18n::t(lang, "axis_dpad").to_string(),
            direction_label: crate::i18n::t(lang, "dir_left").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_HAT0Y",
            label: crate::i18n::t(lang, "axis_dpad").to_string(),
            direction_label: crate::i18n::t(lang, "dir_down").to_string(),
            positive_expected: true,
            source: None,
            invert: false,
            max_val: None,
        },
        AxisSlot {
            xbox_axis: "ABS_HAT0Y",
            label: crate::i18n::t(lang, "axis_dpad").to_string(),
            direction_label: crate::i18n::t(lang, "dir_up").to_string(),
            positive_expected: false,
            source: None,
            invert: false,
            max_val: None,
        },
    ]
}
