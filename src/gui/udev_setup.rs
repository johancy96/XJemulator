use std::path::Path;
use tracing::{info, warn};

const UDEV_RULES_CONTENT: &str = r#"# XJEmulator udev rules (auto-installed)
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
SUBSYSTEM=="input", ATTRS{name}=="Microsoft X-Box 360 pad", ENV{ID_INPUT_JOYSTICK}="1", TAG+="uaccess"
"#;

const RULES_PATH: &str = "/etc/udev/rules.d/99-xjemulator.rules";

/// Check if udev rules are installed
pub fn rules_installed() -> bool {
    Path::new(RULES_PATH).exists()
}

/// Check if uinput is accessible
pub fn uinput_accessible() -> bool {
    Path::new("/dev/uinput").exists() && {
        // Try to check if current user can write
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/uinput")
            .is_ok()
    }
}

/// Status of udev setup
#[derive(Debug, Clone)]
pub struct UdevStatus {
    pub rules_installed: bool,
    pub uinput_accessible: bool,
}

impl UdevStatus {
    pub fn check() -> Self {
        Self {
            rules_installed: rules_installed(),
            uinput_accessible: uinput_accessible(),
        }
    }

    pub fn all_ok(&self) -> bool {
        self.rules_installed && self.uinput_accessible
    }
}

/// Attempt to install udev rules using pkexec (GUI sudo)
pub fn try_install_rules() -> Result<(), String> {
    // Write rules to temp file
    let tmp = "/tmp/99-xjemulator.rules";
    std::fs::write(tmp, UDEV_RULES_CONTENT)
        .map_err(|e| format!("Error escribiendo reglas temporales: {}", e))?;

    // Try pkexec first (GUI auth), then sudo
    let result = std::process::Command::new("pkexec")
        .args(["cp", tmp, RULES_PATH])
        .status();

    match result {
        Ok(status) if status.success() => {
            let _ = std::process::Command::new("pkexec")
                .args(["udevadm", "control", "--reload-rules"])
                .status();
            let _ = std::process::Command::new("pkexec")
                .args(["udevadm", "trigger"])
                .status();
            info!("Reglas udev instaladas correctamente");
            Ok(())
        }
        _ => {
            let result = std::process::Command::new("sudo")
                .args(["cp", tmp, RULES_PATH])
                .status();
            match result {
                Ok(status) if status.success() => {
                    let _ = std::process::Command::new("sudo")
                        .args(["udevadm", "control", "--reload-rules"])
                        .status();
                    let _ = std::process::Command::new("sudo")
                        .args(["udevadm", "trigger"])
                        .status();
                    info!("Reglas udev instaladas con sudo");
                    Ok(())
                }
                _ => Err("No se pudieron instalar las reglas. Ejecuta manualmente:\nsudo cp udev/99-xjemulator.rules /etc/udev/rules.d/".into()),
            }
        }
    }
}

/// Remove udev rules (uninstall)
pub fn try_uninstall_rules() -> Result<(), String> {
    if !rules_installed() {
        return Ok(());
    }
    let result = std::process::Command::new("pkexec")
        .args(["rm", "-f", RULES_PATH])
        .status();
    let ok = match result {
        Ok(s) if s.success() => true,
        _ => std::process::Command::new("sudo")
            .args(["rm", "-f", RULES_PATH])
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
    };
    if ok {
        let _ = std::process::Command::new("udevadm")
            .args(["control", "--reload-rules"])
            .status();
        warn!("Reglas udev desinstaladas");
        Ok(())
    } else {
        Err(format!(
            "No se pudo desinstalar. Ejecuta manualmente:\nsudo rm {}",
            RULES_PATH
        ))
    }
}
