use thiserror::Error;

#[derive(Error, Debug)]
pub enum XjError {
    #[error("No se encontro el dispositivo: {0}")]
    DeviceNotFound(String),

    #[error("Dispositivo no soportado como gamepad: {0}")]
    NotAGamepad(String),

    #[error("Permiso denegado accediendo a {0}")]
    PermissionDenied(String),

    #[error("Error de uinput: {0}")]
    UinputError(String),

    #[error("Error de configuracion: {0}")]
    ConfigError(String),

    #[error("Mapeo invalido: eje origen {source_axis} no encontrado en dispositivo")]
    InvalidMapping { source_axis: String },

    #[error("Dispositivo desconectado: {0}")]
    DeviceDisconnected(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
