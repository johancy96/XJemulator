# XJemulator - Guía Oficial de Usuario 📖

[🌍 English Version](file:///home/johancy/Proyectos/XJemulator/GUIDE_EN.md) / [🇪🇸 Versión en Español](file:///home/johancy/Proyectos/XJemulator/GUIDE.md)

¡Bienvenido a XJemulator! Emulador gráfico de mandos genéricos a Xbox 360 para Linux. Aquí aprenderás a detectar, calibrar y emular de forma transparente tus gamepads para su perfecta compatibilidad con Steam, Proton y Wine.

---

## ⚠️ Paso Crítico Cero: udev y Permisos
Para que Linux permita que XJemulator lea tus mandos físicos e instancie un nuevo "Mando Virtual de Xbox 360", necesita acceso a los módulos de hardware.

1. **Agrega a tu usuario al grupo `input`**:
   Ejecuta esto en tu terminal y **¡reinicia tu computadora / cierra sesión!**
   ```bash
   sudo usermod -aG input $USER
   ```
2. **Verificar reglas udev**:
   Normalmente, al instalar `XJemulator` la aplicación configura estas reglas de manera automática. 
   **Si la instalación automática falló** o corres una copia manual, la aplicación mostrará una advertencia roja en pantalla. En dicho caso, ejecuta lo siguiente en tu terminal para instalarlas manualmente:
   ```bash
   sudo sh -c 'curl -fsSL https://raw.githubusercontent.com/johancy96/XJemulator/master/udev/99-xjemulator.rules > /etc/udev/rules.d/99-xjemulator.rules'
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

---

## 🎮 Tutorial Corto: Cómo Calibrar tu Primer Mando

La calibración le "enseña" a la aplicación cuál botón de tu mando genérico se traduce a qué botón del estándar de Xbox 360. **Solo tendrás que hacer esto una vez por cada modelo de mando que tengas.**

### 1. Selecciona tu Mando (Panel Izquierdo)
Conecta tu control genérico por USB o Bluetooth.
En la parte izquierda encontrarás el área **"🔌 Mandos Detectados"**. Busca tu mando en la lista y presiona el rectángulo en la lista para seleccionarlo. En el Monitor RAW central deberían empezar a saltar códigos si oprimes los botones.

### 2. Inicia el Asistente de Calibración
Una vez que el mando esté seleccionado en el menú izquierdo (y te asegures de que no esté emulando todavía mediante el botón rojo Detener), presiona el botón situado bajo el monitor central que dice **"▶ Iniciar calibración"**.

### 3. Sigue las Instrucciones en Pantalla
El sistema bloqueó y te guiará botón por botón:
- **Calibrando Botones:** Aparecerá en color amarillo el nombre de un botón (Ej: "Botón A - El botón inferior (verde/azul)"). Simplemente **presiónalo fuerte** en tu mando de forma consecutiva. La interfaz lo detectará, pitará, y pasará al siguiente por su cuenta. 
  > *Truco: Si tu mando no tiene algún botón avanzado (como un Botón "HOME" o "Guía"), haz clic en el hipervínculo en pantalla "Saltar (No Mapear)"*.
- **Calibrando Ejes (Palancas y Gatillos):** Las palancas requieren de precisión. Sigue la instrucción textual, si dice "Palanca Izquierda -> empuja hacia ABAJO", la empujas y sostienes hasta hacer tope.
  Luego espera la instrucción **"✅ ¡Detectado! Suelta el control y vuelve al centro..."**. Solo entonces puedes aflojar la presión para que el emulador detecte el punto neutro de resorte.
  > *Después de detectar el centro, en ocasiones el sistema te sugerirá "¿Invertir dirección?". Presiona simplemente "Aceptar y Continuar" salvo que el joystick se porte extraño en un juego.*

### 4. Guarda tu Perfil
Al finalizar todos los ejes, te saldrá un mensaje de **"¡Calibración Completada!"**. 
Escribe el nombre con el cual identificarás a tu control (Ej: `mando_ps3_azul`) en el cuadro superior y presiona el botón **💾 Guardar Perfil y Finalizar**. Este perfil se guardará y aparecerá para siempre en tu panel lateral derecho.

---

## 🚀 Uso Diario: Activar la Emulación

¡A jugar! Una vez hayas realizado la configuración descrita, solo te toma **un clic** cada día que desees jugar:
1. Conecta tu mando y abre XJemulator.
2. Búscalo en la lista izquierda, localiza su recuadro y presiona el botón de **"▶ (Play) Emulador "**.

Un indicador verde comenzará a rotar sobre el perfil del mando en la interfaz de la aplicación, indicándolo como "Activo". Mientras la app permanezca abierta minimizada en tu Linux, tu computadora creerá tener conectado por USB un genuino `Microsoft X-Box 360 pad`. ¡Abre Steam y disfruta!.

---

## ⚕️ Resolución de Problemas (Troubleshooting)

### El emulador no crea el dispositivo virtual (Proton no responde)
- Asegúrate de haber completado y reiniciado tras seguir el Paso 0 ("Grupo input").
- Verifica en la barra estática de arriba que diga con orgullo `✓ udev instalado`. 

### Proton / Steam / Wine no detectan el Xbox 360 incluso cuando emula
- Ve al modo **Steam Big Picture > Parámetros > Controlador** y verifica si está activa la compatibilidad con Mandos de Xbox.
- Desactiva el soporte a "Mandos genéricos" del menú de Steam para evitar interferencia de capa doble con el mando falso. Evita presionar comandos mientras carga.

### Mis personajes en X juego "caminan solos" (Drift Extremo / Phantom inputs)
- Esto ocurre cuando la lectura RAW del mando descansa fuera de límites. Vuelve a calibrar el mando deteniéndote minuciosamente donde el asistente pide *"Suelta el control y vuelve al centro"*, dándole el punto inerte original al controlador.
