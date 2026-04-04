# Internacionalización (i18n)

Implementada satisfactoriamente en `v0.1.0`.

## Racionalización (Por qué no gettext)
Integrar bibliotecas C-binded como `gettext` arruinaba la promesa de XJemulator de poder ser compilado de manera rápida y sin acarrear dependencias del sistema externo pesadas (ideal para compilaciones cross-distrò).

Se optó por generar la librería nativa minimalista alojada en `src/i18n.rs`. Esta engloba la enumeración persistente `Lang` y ejecuta conversiones O(1) vía una función estandarizada de match.

## Actualización en Vivo
Las interfaces del GUI se actualizan inmediatamente ya que en la barra de tareas (TopBar) se muta el idioma activo `self.config.lang`. En la función de Update del Render loop de Egui, componentes enteros se redibujan usando referenciación a los strings `crate::i18n::t(&self.config.lang, "calib_btn")`, re-esculpiendo por completo los UI blocks dinámicamente y evitando interrupciones molestas de la aplicación al usuario.
