use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lang {
    Es,
    En,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::Es
    }
}

pub fn t<'a>(lang: &Lang, key: &'a str) -> &'a str {
    match (lang, key) {
        // --- COMMON / APP ---
        (_, "app_title") => "🎮 XJEmulator",
        (Lang::Es, "btn_stop_all") => "⏹ Detener Todos",
        (Lang::En, "btn_stop_all") => "⏹ Stop All",
        (Lang::Es, "btn_refresh") => "🔄 Refrescar",
        (Lang::En, "btn_refresh") => "🔄 Refresh",
        (Lang::Es, "lbl_active_emulators") => "emuladores Xbox 360 activos",
        (Lang::En, "lbl_active_emulators") => "active Xbox 360 emulators",

        // --- MAIN PANEL LEFT ---
        (Lang::Es, "lbl_detected_pads") => "Mandos detectados",
        (Lang::En, "lbl_detected_pads") => "Detected gamepads",
        (Lang::Es, "lbl_no_pads") => "(Ningún mando físico detectado)",
        (Lang::En, "lbl_no_pads") => "(No physical gamepad detected)",
        (Lang::Es, "tooltip_play") => "Iniciar emulación con este mando",
        (Lang::En, "tooltip_play") => "Start emulation with this gamepad",
        (Lang::Es, "tooltip_stop") => "Detener emulación",
        (Lang::En, "tooltip_stop") => "Stop emulation",

        // --- PROFILES PANEL RIGHT ---
        (Lang::Es, "lbl_saved_profiles") => "Perfiles guardados",
        (Lang::En, "lbl_saved_profiles") => "Saved profiles",
        (Lang::Es, "tooltip_delete") => "Eliminar perfil",
        (Lang::En, "tooltip_delete") => "Delete profile",
        (Lang::Es, "tooltip_view") => "Ver contenido",
        (Lang::En, "tooltip_view") => "View content",
        (Lang::Es, "btn_save_changes") => "💾 Guardar cambios",
        (Lang::En, "btn_save_changes") => "💾 Save changes",

        // --- RAW MONITOR ---
        (Lang::Es, "lbl_raw_monitor") => "Monitor RAW",
        (Lang::En, "lbl_raw_monitor") => "RAW Monitor",
        (Lang::Es, "lbl_raw_sub") => "(lo que reporta Linux sin mapeo)",
        (Lang::En, "lbl_raw_sub") => "(what Linux reports without mapping)",
        (Lang::Es, "lbl_move_pad") => "Mueve el mando para\nver eventos aquí",
        (Lang::En, "lbl_move_pad") => "Move the gamepad to\nsee events here",

        // --- CALIBRATION WIZARD ---
        (Lang::Es, "lbl_calib_wiz") => "Asistente de Calibración",
        (Lang::En, "lbl_calib_wiz") => "Calibration Wizard",
        (Lang::Es, "lbl_profile_name") => "Nombre del nuevo perfil:",
        (Lang::En, "lbl_profile_name") => "New profile name:",
        (Lang::Es, "btn_start_calib") => "▶ Iniciar calibración",
        (Lang::En, "btn_start_calib") => "▶ Start calibration",
        (Lang::Es, "warning_busy") => {
            "⚠️ ESTE MANDO ESTÁ ACTIVO\nDetén el emulador de este mando\npara poder calibrarlo."
        }
        (Lang::En, "warning_busy") => {
            "⚠️ THIS GAMEPAD IS ACTIVE\nStop the emulator for this gamepad\nto calibrate it."
        }
        (Lang::Es, "warning_select") => "(Selecciona un mando a la izquierda)",
        (Lang::En, "warning_select") => "(Select a gamepad on the left)",

        (Lang::Es, "calib_btn_prompt") => "Presiona y mantén",
        (Lang::En, "calib_btn_prompt") => "Press and hold",
        (Lang::Es, "calib_axis_prompt") => "Mueve a tope el",
        (Lang::En, "calib_axis_prompt") => "Move fully the",
        (Lang::Es, "calib_detected") => "✅ ¡Detectado!",
        (Lang::En, "calib_detected") => "✅ Detected!",
        (Lang::Es, "calib_release") => "Suelta el control y vuelve al centro...",
        (Lang::En, "calib_release") => "Release the control and return to center...",
        (Lang::Es, "calib_btn_skip") => "Saltar (No Mapear)",
        (Lang::En, "calib_btn_skip") => "Skip (Unmap)",
        (Lang::Es, "calib_waiting") => "Esperando pulsación...",
        (Lang::En, "calib_waiting") => "Waiting for input...",

        (Lang::Es, "chk_invert_axis") => "¿Invertir dirección?",
        (Lang::En, "chk_invert_axis") => "Invert direction?",
        (Lang::Es, "btn_next") => "Aceptar y Continuar",
        (Lang::En, "btn_next") => "Accept and Continue",

        (Lang::Es, "lbl_calib_done") => "¡Calibración Completada!",
        (Lang::En, "lbl_calib_done") => "Calibration Completed!",
        (Lang::Es, "btn_save_finish") => "💾 Guardar Perfil y Finalizar",
        (Lang::En, "btn_save_finish") => "💾 Save Profile and Finish",
        (Lang::Es, "btn_cancel_calib") => "❌ Cancelar Calibración",
        (Lang::En, "btn_cancel_calib") => "❌ Cancel Calibration",

        // --- NEW WIZARD STRINGS ---
        (Lang::Es, "lbl_calib_incl") => "La calibración incluirá:",
        (Lang::En, "lbl_calib_incl") => "Calibration will include:",
        (Lang::Es, "lbl_calib_btns") => "botones.",
        (Lang::En, "lbl_calib_btns") => "buttons.",
        (Lang::Es, "lbl_calib_axes") => "ejes.",
        (Lang::En, "lbl_calib_axes") => "axes.",
        (Lang::Es, "lbl_calib_skip") => "Podrás OMITIR cualquier control que no tengas.",
        (Lang::En, "lbl_calib_skip") => "You can SKIP any control you don't have.",
        (Lang::Es, "lbl_buttons") => "Botones",
        (Lang::En, "lbl_buttons") => "Buttons",
        (Lang::Es, "lbl_axes") => "Ejes",
        (Lang::En, "lbl_axes") => "Axes",
        (Lang::Es, "calib_btn_press") => "Presiona o mueve el botón correspondiente:",
        (Lang::En, "calib_btn_press") => "Press or move the corresponding button:",
        (Lang::Es, "lbl_skipped") => "(Omitido)",
        (Lang::En, "lbl_skipped") => "(Skipped)",
        (Lang::Es, "lbl_move_joystick") => "Mueve a tope el joystick indicado:",
        (Lang::En, "lbl_move_joystick") => "Fully move the indicated joystick:",
        (Lang::Es, "calib_detecting") => "Detectando...",
        (Lang::En, "calib_detecting") => "Detecting...",
        (Lang::Es, "btn_cancel") => "Cancelar",
        (Lang::En, "btn_cancel") => "Cancel",

        // --- DEFAULT BUTTONS ---
        (Lang::Es, "btn_a") => "Botón A",
        (Lang::En, "btn_a") => "Button A",
        (Lang::Es, "hint_btn_a") => "El botón inferior (verde/azul)",
        (Lang::En, "hint_btn_a") => "The bottom button (green/blue)",
        (Lang::Es, "btn_b") => "Botón B",
        (Lang::En, "btn_b") => "Button B",
        (Lang::Es, "hint_btn_b") => "El botón derecho (rojo)",
        (Lang::En, "hint_btn_b") => "The right button (red)",
        (Lang::Es, "btn_x") => "Botón X",
        (Lang::En, "btn_x") => "Button X",
        (Lang::Es, "hint_btn_x") => "El botón izquierdo (azul)",
        (Lang::En, "hint_btn_x") => "The left button (blue)",
        (Lang::Es, "btn_y") => "Botón Y",
        (Lang::En, "btn_y") => "Button Y",
        (Lang::Es, "hint_btn_y") => "El botón superior (amarillo)",
        (Lang::En, "hint_btn_y") => "The top button (yellow)",
        (Lang::Es, "btn_lb") => "Bumper Izquierdo (LB)",
        (Lang::En, "btn_lb") => "Left Bumper (LB)",
        (Lang::Es, "hint_btn_lb") => "Gatillo digital superior izquierdo",
        (Lang::En, "hint_btn_lb") => "Top left digital trigger",
        (Lang::Es, "btn_rb") => "Bumper Derecho (RB)",
        (Lang::En, "btn_rb") => "Right Bumper (RB)",
        (Lang::Es, "hint_btn_rb") => "Gatillo digital superior derecho",
        (Lang::En, "hint_btn_rb") => "Top right digital trigger",
        (Lang::Es, "btn_back") => "Botón Back / Select",
        (Lang::En, "btn_back") => "Back / Select Button",
        (Lang::Es, "hint_btn_back") => "Botón pequeño izquierdo del centro",
        (Lang::En, "hint_btn_back") => "Small left button near center",
        (Lang::Es, "btn_start") => "Botón Start / Menú",
        (Lang::En, "btn_start") => "Start / Menu Button",
        (Lang::Es, "hint_btn_start") => "Botón pequeño derecho del centro",
        (Lang::En, "hint_btn_start") => "Small right button near center",
        (Lang::Es, "btn_guide") => "Botón Guía / Xbox",
        (Lang::En, "btn_guide") => "Guide / Xbox Button",
        (Lang::Es, "hint_btn_guide") => "Botón central grande (logo), omite si no existe",
        (Lang::En, "hint_btn_guide") => "Large central button (logo), skip if none",
        (Lang::Es, "btn_l3") => "Click Palanca Izquierda (L3)",
        (Lang::En, "btn_l3") => "Left Stick Click (L3)",
        (Lang::Es, "hint_btn_l3") => "Presiona la palanca izquierda hacia adentro",
        (Lang::En, "hint_btn_l3") => "Press the left stick inward",
        (Lang::Es, "btn_r3") => "Click Palanca Derecha (R3)",
        (Lang::En, "btn_r3") => "Right Stick Click (R3)",
        (Lang::Es, "hint_btn_r3") => "Presiona la palanca derecha hacia adentro",
        (Lang::En, "hint_btn_r3") => "Press the right stick inward",

        // --- DEFAULT AXES ---
        (Lang::Es, "axis_left_stick") => "Palanca Izquierda",
        (Lang::En, "axis_left_stick") => "Left Stick",
        (Lang::Es, "axis_left_trigger") => "Gatillo Izquierdo (LT)",
        (Lang::En, "axis_left_trigger") => "Left Trigger (LT)",
        (Lang::Es, "axis_right_stick") => "Palanca Derecha",
        (Lang::En, "axis_right_stick") => "Right Stick",
        (Lang::Es, "axis_right_trigger") => "Gatillo Derecho (RT)",
        (Lang::En, "axis_right_trigger") => "Right Trigger (RT)",
        (Lang::Es, "axis_dpad") => "Cruz Direccional (D-Pad)",
        (Lang::En, "axis_dpad") => "Directional Pad (D-Pad)",

        (Lang::Es, "dir_right") => "empuja a la DERECHA  ▶",
        (Lang::En, "dir_right") => "push RIGHT  ▶",
        (Lang::Es, "dir_left") => "empuja a la IZQUIERDA ◀",
        (Lang::En, "dir_left") => "push LEFT ◀",
        (Lang::Es, "dir_down") => "empuja hacia ABAJO  ▼",
        (Lang::En, "dir_down") => "push DOWN  ▼",
        (Lang::Es, "dir_up") => "empuja hacia ARRIBA  ▲",
        (Lang::En, "dir_up") => "push UP  ▲",
        (Lang::Es, "dir_press") => "apriétalo a FONDO  ⬇",
        (Lang::En, "dir_press") => "press fully DOWN  ⬇",

        // Default catch
        (_, k) => k,
    }
}
