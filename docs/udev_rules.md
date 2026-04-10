# Reglas udev y Emulación uinput

## Seguridad vs Emulación Virtual
Históricamente, cualquier utilidad que inyecta código HID o manipula input virtual en Linux requería ejecución mediante `sudo`. Desde el desarrollo `v0.1.0` eliminamos esta limitante radical estableciendo delegación a través del subsistema `udev`.

## Instalación Automática
La carga subyacente para evadir root es proveer permisos rw a `/dev/uinput` y permitir acceso local a los input crudos en el grupo `input`. 

Ya no se realiza una escalada de privilegios dinámica ni interactiva desde la GUI con `pkexec`. 
En su lugar, el archivo `99-xjemulator.rules` se distribuye e instala automáticamente a través de los sistemas de empaquetado y scripts de instalación (`.deb`, `.rpm`, `PKGBUILD` o `install.sh`).

### Contenido de la regla

```bash
# Otorgar acceso a uinput a usuarios comunes sin necesidad de sudo
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"

# Exponer el dispositivo virtual creado por el emulador a todos los entornos locales (Steam/Proton)
SUBSYSTEM=="input", ATTRS{name}=="Microsoft X-Box 360 pad", MODE="0666", ENV{ID_INPUT_JOYSTICK}="1"
```

## Beneficios
- Total compatibilidad con Steam Input y Wine/Proton.
- Interfaz no corre bajo "sandox restrictiva" o root previniendo riesgos críticos del desktop manager.
- Elimina cualquier script post-instalación destructivo.
