# Sistema de Internacionalización (i18n)

Para soportar múltiples lenguajes de la UI sin añadir dependencias pesadas de ecosistemas como `fluent` o `gettext`, implementamos un motor ligero con el modelo "Hardcoded Dictionary".

## Componentes Técnicos
1. **El Motor (`src/i18n.rs`)**: 
   Contiene un mapeo directo `HashMap<&'static str, &'static str>` administrado localmente según el lenguaje provisto (`Lang::Es` vs `Lang::En`). 
2. **Método Transmisor**: `crate::i18n::t(&self.config.language, "key_string")`
   Proporciona resolución instantánea "O(1)". Se evitan allocations durante el path crítico gráfico excepto si el render de widget lo reclama estrictamente.
3. **Persistencia de Configuración**: Las preferencias del usuario de idioma operan leyendo y escribiendo en el `~/.local/share/XJemulator/config.toml`. Un reinicio parcial de la UI se realiza si se mutan en ejecución gracias al re-render inherente a los frames de `egui`.

## Diccionarios Implementados
La matriz base actual abarca:
- `Español (Es)`: Diccionario por defecto, tono informal, descripciones claras como "Palanca Izquierda".
- `English (En)`: Traducción uno a uno con tono amigable. "Left Stick".

## Proyección para Incorporación
Para agregar Francés por ejemplo, basta con expandir el `enum Lang` y rellenar en `i18n.rs` un brazo completo en el *Match statement*. Todo el UI es absolutamente dependiente del método `t(...)`. No quedan literales quemados en la UI.
