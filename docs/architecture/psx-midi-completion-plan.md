# Finalización del conversor MIDI y de la reproducción musical PSX

Estado: plan de implementación, 2026-09-13. Este documento no declara implementadas las funciones propuestas.

**Actualización de alcance:** el usuario aceptó el resultado escuchado y pidió
desplegar la función usable, hacer una comprobación básica y push, documentando
las pruebas extensas pendientes. Este plan conserva los gates originales como
backlog; no se declaran superados. Véase [la entrega acotada](psx-midi-delivery.md),
incluido el bloqueo de RAM de Ironwood con Title y las escenas de bosque.

Caso de aceptación: `D:/GitProjects/GameEngines/Epok/EpokDemos/Ironwood/assets/sounds/bgm/opening_02.mid`, reproducido desde Explorer y como música de `Title` en PSX.

Continúa el [prompt de implementación](cross-platform-audio-agent-prompt.md), el [plan A–E](cross-platform-audio-implementation-plan.md) y la [arquitectura multiplataforma](multi-platform-architecture.md). El [registro de progreso](cross-platform-audio-progress.md) distingue lo ya demostrado de los pendientes. Se mantienen sus gates, protección de cambios ajenos y restricciones de plataforma.

## 1. Resultado que debe poder usar el usuario

1. Seleccionar o importar un MIDI y abrir su conversión para PSX desde el flujo existente de importación/Inspector.
2. Elegir una biblioteca de instrumentos y un preset de calidad. Con una biblioteca predeterminada instalada y compatible, las asignaciones se resuelven automáticamente.
3. Ver qué instrumentos necesita, qué efectos se pueden representar, el coste estimado y los ajustes efectivos. Abrir opciones avanzadas únicamente si necesita afinarlos.
4. Escuchar y comparar `Source Preview` y `Target Preview`, con la misma biblioteca y volumen de comparación. El segundo utiliza los datos realmente convertidos.
5. Aplicar los ajustes y obtener un `MusicSequence` con su dependencia `SoundBank` correctamente resuelta. El conversor prepara los samples, mappings y derivados PSX; no exige construir a mano cada instrumento.
6. Asignar ese asset al `AudioSource` de `Title Music`, activar el inicio automático y escuchar la canción completa, sus loops y los SFX del menú.
7. Mantener el cursor unos dos segundos sobre **cualquier opción del conversor** para leer un tooltip que explique qué cambia, las unidades, el coste y las limitaciones relevantes.

El nombre de producto será **conversión de música para PSX**. Se reutilizan los payloads internos versionados `EPSQ` y `EPSB`. No es necesario elegir entre nombres de formatos de secuencia para importar un MIDI.

El flujo técnico es `MIDI + biblioteca de instrumentos → IR musical y de instrumentos → cooker PSX → secuencia + banco SPU`. La calidad depende de la interpretación musical, los samples, sus envelopes/loops y la conversión. Cambiar el contenedor de la secuencia no aporta por sí solo instrumentos ni efectos.

### Qué significa terminar al 100 %

Se deben cerrar todos los gates de este plan para el perfil PSX publicado, incluida la integración real de Ironwood, las opciones de conversión, la UX y los pendientes de fidelidad de los adaptadores PSX previamente iniciados. La validación física se declara completa únicamente con evidencia física.

El perfil tendrá una matriz explícita de operaciones admitidas. MIDI 2.0, extensiones arbitrarias de fabricantes y certificación completa GM2/GS no se anuncian como soportados. Los eventos fuera del perfil conservan su información y producen diagnósticos precisos. Para `opening_02.mid` no se permite necesitar `Ignore unsupported events`, sustituciones mudas ni instrumentos genéricos de emergencia.

N64, PS2 y GameCube continúan fuera del alcance. No se implementan T27, T28 ni T42, ni cookers, SDKs, preview modes o backends simulados para esas plataformas. Fuente, IR, identidad de caché y overrides permanecen independientes del target.

## 2. Evidencia de partida

Engine inspeccionado: commit `26cdef3810250c434550710802a6042f5a1d102e`, rama `develop`.

Había cambios previos en `docs/assets.md`, `src/asset_inspector.rs`, `src/content_preview.rs` y `src/project_browser.rs`: corrección para publicar fallos de preview una vez por intento en Console. Deben conservarse y revisarse como un cambio separado. Ironwood no es un repositorio Git; su integración requiere snapshot de sus archivos antes de editarlos.

### Auditoría del MIDI real

Se ejecutó el parser Rust de producción mediante el harness local `artifacts/midi-event-audit/`. El informe es `artifacts/midi-event-audit/opening_02-report.json`. Es una lectura del contenido, no una prueba de reproducción.

| Dato medido | Resultado |
|---|---|
| Fuente | 13 946 bytes; SMF formato 1; 9 tracks; PPQN 480 |
| SHA-256 del MIDI | `5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe` |
| Asset existente | UUID `84ad8004-1cd4-44ac-8acb-3ccd06cd84f2` |
| Duración musical | 107 520 ticks; 112 segundos hasta el fin de la secuencia; colas de samples aparte |
| Notas | 1508 Note On; pico lógico 11, sin las colas de release ni capas de instrumentos |
| Programas melódicos, base cero | 44, 52, 70, 71, 75, 104, 105 |
| Percusión, canal 10 | Programa 0; teclas 52, 70, 73, 74, 84, 86, 87 |
| Pitch bend observado | 8192, 8832 y 9600 |
| Settings actuales | Music, Resident, Whole loop; `ignore_unsupported=false`; SoundBank sin asignar |

La biblioteca debe cubrir también las teclas 84, 86 y 87 con un mapa de percusión documentado. No basta con etiquetar una biblioteca como «GM» y asumir esa cobertura.

Los 280 diagnósticos actuales corresponden a mensajes de inicialización en tick 0:

| Grupo | Mensajes | Trabajo necesario |
|---|---:|---|
| Selección RPN y Data Entry | 205 | Resolver selección, valores, afinación y rango de pitch bend |
| Bank Select CC0=0 | 15 | Resolver identidad del banco y programa |
| CC91/92/93/95=0 | 60 | Interpretar explícitamente el estado de efectos desactivados |

El archivo configura una sensibilidad de pitch bend de 12 semitonos y afinación fina/gruesa central. El runtime inicial aplica ±2 semitonos: ignorar los mensajes cambia el resultado audible. RPN, Data Entry y los controladores de efectos deben interpretarse según el [contrato MIDI publicado](https://midi.org/midi-1-0-control-change-messages).

El pico 11 permite investigar una conversión con margen para SFX, pero no prueba que el banco final quepa: faltan samples, capas, colas y medición física. El banco `Retro Starter` actual solo contiene el programa 0 y no cubre esta canción.

### Estado de Ironwood

- `assets/scenes/Title.epokmap` tiene un `AudioSource` activo llamado `Title Music`, volumen 0,7 y autoplay. Apunta al asset de `Field1`, no a `opening_02`.
- El controlador del menú busca la entidad por el nombre `Title Music` para aplicar el volumen de Options. Conservar nombre, identidad y contrato de volumen.
- Los sonidos de mover, aceptar y cancelar del menú tienen prioridad 220. Deben seguir funcionando.
- `startup_scene` ya apunta a Title, pero la configuración de Play usa serial/host, empieza en ForestClearing y selecciona ForestBattle/ForestClearing. Title queda fuera de esa selección. La prueba debe configurar explícitamente una ejecución que incluya e inicie Title.

### Evidencia anterior que debe preservarse

- Suite Rust A–E: 399 tests pasados, 35 ignorados y un fallo GUI previamente reproducido en baseline. Clippy: 18 diagnósticos baseline, sin delta. No presentar esas suites como totalmente verdes.
- 21 ejecutables nativos y regresiones PSX SPU/XA pasaron en el checkpoint D; E volvió a verificar SPU/XA con el ejecutable final.
- Stress de emulador D: máximo observado de servicio 2922,572 µs, intervalo 3024 µs, 16 robos de voz y 75 notas denegadas en un caso de agotamiento deliberado. No son resultados de Ironwood ni límites garantizados.
- Fixture mínima D: secuencia 76 bytes, banco 1472 bytes, samples SPU 1344 bytes. Estos tamaños no estiman el banco que requiere `opening_02`.
- Hardware físico, picos de heap/stack y costes completos de esta canción: pendientes.

## 3. Contrato del conversor y de su UI

### Flujo y responsabilidades

La conversión amplía el importador y el Inspector actuales. No crea un segundo pipeline de assets ni obliga a usar herramientas externas.

La biblioteca predeterminada será una biblioteca versionada con fuentes y licencia de redistribución verificadas, instalada con el editor o mediante un paquete explícito. En P2 se selecciona y registra la biblioteca concreta; no se declara disponible antes. El usuario también podrá importar una biblioteca SF2 compatible o seleccionar un `SoundBank` propio. General MIDI define asociaciones de instrumentos, pero no su timbre acústico exacto; la referencia de escucha se fija junto con la biblioteca. Véase [General MIDI Level 1](https://midi.org/general-midi-level-1).

El conversor analiza bancos/programas, teclas, velocidades, controles y loops; crea las dependencias necesarias y cocina solo los recursos alcanzables. La biblioteca fuente sigue siendo autoritativa y reutilizable. El banco reducido por canción es un derivado de cook: su identidad incluye la secuencia que determinó la selección. No se poda destructivamente una biblioteca compartida.

Detectar un archivo, calcular una estimación o escuchar un borrador no publica assets. La acción explícita de importar/aplicar publica la secuencia y las nuevas dependencias mediante una transacción; reimportar conserva los UUID existentes. Cocinar un target incompatible mantiene válido el authoring y devuelve el error del target.

### Presets iniciales propuestos

| Preset | Resolución inicial de samples PSX | Política |
|---|---|---|
| Compact | Máximo 11 025 Hz, mono | Prioriza espacio; conserva notas, tempo y semántica musical |
| Balanced, predeterminado | Máximo 22 050 Hz, mono | Punto de partida para escuchar y medir en Ironwood |
| High | Máximo 44 100 Hz, mono | Conserva más contenido de alta frecuencia cuando existe en la fuente |
| Custom | Parámetros explícitos | Muestra diferencias respecto al preset de partida |

Son máximos de conversión, no una obligación de aumentar la frecuencia de una fuente inferior. Cada preset es una receta versionada y muestra su resolución efectiva. Mantienen las capas y articulaciones requeridas; cualquier reducción adicional requiere una opción explícita y un informe de lo alterado. Los valores se validarán auditivamente en P3 antes de publicarlos como presets finales.

Si High no cabe, se muestra el exceso. El botón explícito `Optimize to budget` calcula una propuesta reproducible, con límites elegidos por el usuario y comparación previa de las modificaciones. Aplicarla es una acción diferenciada. Nunca cambia Resident por Stream, elimina notas o sustituye instrumentos silenciosamente.

### Opciones y contenido mínimo de sus ayudas

Las primeras cuatro opciones forman el panel básico. El resto pertenece a Advanced o al informe contextual. Todas las opciones visibles deben funcionar en el perfil seleccionado; una opción deshabilitada muestra el prerequisite que falta.

| Opción | Qué debe explicar el tooltip |
|---|---|
| Target/profile | Para qué consola se prepara el resultado y qué límites aplica el perfil. Solo PSX implementado. |
| Instrument library | De dónde procede el sonido de los instrumentos, qué programas cubre y cómo cambiar la biblioteca altera el timbre. |
| Quality preset | Qué ajustes cambia; qué se conserva; cuándo aumenta la memoria. Al editar un ajuste, pasa a Custom. |
| Loop | Off, Whole o markers; punto de inicio/fin y comportamiento de notas activas/colas al repetir. |
| Sample rate | Frecuencia de los samples: una menor puede perder brillo y ahorrar memoria. No cambia el tempo ni la afinación de la canción. |
| Encoder effort | Más búsqueda del encoder puede mejorar el error de ADPCM y tardar más al convertir. No cambia la tasa fija del formato SPU ni añade CPU equivalente durante playback. |
| Sample channel policy | El perfil inicial usa samples mono con paneo de voces. Explicar el downmix; conservar estéreo requiere un perfil real de dos voces, no una casilla sin implementación. |
| Instrument mapping / drum map | Correspondencia banco/programa/tecla, numeración visible y sustituciones explícitas. Indicar exactamente qué falta. |
| Sample/zone reduction | Qué multisamples o capas se podrían reducir o precalcular, qué detalle se pierde y cómo cambia el número de voces. Desactivado por defecto. |
| Sample loops | Mantener o ajustar loops de los instrumentos; diferenciarlos del loop de la canción; informar desplazamientos de los bloques ADPCM. |
| Envelope/release policy | Cómo se representa el ataque y la cola; prolongar release puede aumentar las voces simultáneas aunque no aumente la secuencia. |
| Effects policy | Qué efectos pide la fuente/biblioteca, cuáles son exactos o adaptados por el perfil y su coste. Dry u otro cambio explícito indica la diferencia musical. |
| Gain/headroom | Nivel de salida y margen para sumar música y SFX. Ganancia adicional no mejora calidad; evitar recortes y conservar los niveles relativos de los instrumentos. |
| Music voice limit | Voces físicas máximas de música y relación con notas, capas, colas y otras secuencias. Explicar la prioridad frente a SFX; no prometer una reserva que el allocator no aplique. |
| Bank memory budget | Presupuesto SPU disponible después de SFX y reservas, con unidades. Distinguirlo de main RAM y tamaño en disco. |
| Optimization limits | Cambios que el optimizador tiene permitido proponer; nunca descartar eventos por optimización. |
| Unsupported-event policy | Qué operaciones concretas impiden conversión. La excepción legacy no habilita formatos/bancos bloqueados ni es necesaria para esta canción. |
| Source / Target Preview | Qué representación se oye y qué limitaciones conserva la simulación host respecto de la SPU física. |

Role y Load Mode conservan sus controles y ayudas cuando aparezcan en este flujo: intención musical y residencia son independientes; los eventos son residentes y el banco tiene su política separada.

Ejemplo de texto para Sample rate: «Controla el detalle de las muestras de instrumentos. 22 050 Hz suele ocupar menos memoria que 44 100 Hz para una misma duración, a costa de las frecuencias altas. La melodía mantiene su tempo y afinación. El coste actualizado aparece en el informe de conversión». Adaptar el texto al idioma y terminología existentes del editor.

Implementar una ayuda común sobre el control, su etiqueta y un marcador de ayuda. Retardo inicial de 2,0 segundos con reloj de UI, identidad estable por opción y párrafos con ancho limitado. Reiniciar al salir/cambiar de opción, evitar parpadeo al moverse dentro de la misma opción y permitir consultar la ayuda mediante foco/activación del marcador. Los controles deshabilitados también deben explicar su estado. El tooltip no debe ocultar errores que el usuario necesita ver sin esperar.

### Informe y preview

- Mostrar duración, instrumentos, operaciones musicales, cobertura de notas/velocidades, coste de secuencia/banco, main RAM, SPU, voces y efectos.
- Distinguir estimación de análisis, tamaño exacto tras cook y máximo observado en ejecución. Indicar qué falta medir.
- Incluir la demanda del proyecto: los bancos actuales se cargan completos y permanecen residentes en main RAM y SPU; sumar otros bancos y SFX. No presentar el presupuesto de una canción aislada como presupuesto del juego.
- Actualizar análisis/cook en workers cancelables; una sola audition; descartar resultados tardíos; permitir cancelar una búsqueda de optimización acotada.
- Cachear por fuente, biblioteca, selección de instrumentos, settings, versión del renderer/cooker, target y profile. A/B debe identificar siempre los parámetros del audio que se está escuchando.
- Usar Console una vez por intento fallido y un resumen accionable en el informe. Agrupar, por ejemplo, 205 mensajes RPN con sus detalles expandibles; conservar ubicación, canal, tick y valor.

## 4. Fases, dependencias y gates

Cada fase termina con un checkpoint revisable, tests pertinentes y actualización de evidencia antes de avanzar. P0–P6 forman la ruta prioritaria hasta usar la canción en Title. P7 cierra la compatibilidad PSX pendiente y la entrega completa.

| Fase | Dependencias | Entrega observable |
|---|---|---|
| P0 — Baseline y contrato | Inspección inicial | Caso real reproducible, perfil y criterios de calidad fijados |
| P1 — Interpretación MIDI | P0 | Los 280 eventos reciben una interpretación correcta y trazable |
| P2 — Biblioteca y SoundBank | P1, contrato estable | Asignación automática de instrumentos y percusión reales |
| P3 — Conversión y calidad | P2 | Presets, optimización explícita, cook determinista y costes |
| P4 — Reproducción y fidelidad PSX | P3 | Sonido convertido reproducible, controles correctos, recursos acotados |
| P5 — Flujo del editor y ayuda | P4 | Convertir, ajustar, comparar, aplicar y entender cada opción |
| P6 — Ironwood | P5 | `opening_02` suena en Title, con menú, loops y transiciones |
| P7 — Cierre PSX y validación final | P6; fidelidad sobre P2–P4 | Pendientes de E resueltos, regresiones y evidencia final |

### P0 — Baseline, referencias y contrato

1. Volver a inspeccionar estado y hashes; preservar los cambios existentes y capturar los archivos de Ironwood antes de la futura integración. Registrar SDK/compiler/emulador y perfil de build.
2. Conservar el informe del MIDI real, añadir trazas de control/nota y rangos por instrumento, y generar MIDI sintéticos redistribuibles que reproduzcan sus casos. El archivo externo sirve por ruta/hash; no se añade al repositorio sin verificar su redistribución.
3. Definir una matriz por evento/generator: preservado en fuente, interpretable en IR, representable exactamente en el perfil, adaptación explícita o error. Separar requisitos del MIDI de requisitos que introduzca la biblioteca.
4. Fijar biblioteca y referencia de escucha durante P2. Supuesto inicial: buena interpretación musical para PSX; igualar una grabación concreta requiere conocer esa referencia y los instrumentos empleados.
5. Fijar antes de P4 límites de memoria/servicio y tolerancias medibles de afinación/timing. Conservar baseline SFX/XA y no elevar un límite después solo para ocultar una regresión.

**Gate P0:** fallo actual reproducido; hashes y baseline registrados; fixtures sintéticas y criterios de comparación disponibles; requisitos pendientes identificados sin presentarlos como soporte.

### P1 — Semántica MIDI e IR neutral

1. Ampliar estado de canal para selección RPN/NRPN, Data Entry y sensibilidad de bend/afinación; aplicar RPN 0, 1 y 2, selección nula e incrementos/decrementos conforme a la especificación. Probar selección incompleta, mensajes intercalados y aislamiento entre canales.
2. Representar Bank Select MSB/LSB y el banco efectivo de Program Change. Evitar que un cambio de programa cambie el instrumento de notas ya activas. Versionar la identidad banco/programa/percusión y preservar mappings legacy de banco 0.
3. Representar explícitamente los cuatro controles de efectos en cero del archivo. No aceptar valores distintos fingiendo que carecen de efecto. Conservar eventos, orden y estado para reproducir y restaurar loops.
4. Completar en el perfil musical de uso general modulación, reset de controladores, All Notes Off/All Sound Off y tratamiento de sustain/sostenuto; aftertouch solo donde tenga un destino de modulación definido. Auditar resets de sistema y extensiones encontradas en el corpus, con contratos separados.
5. Mantener límites del parser y detección de overflow, SMF 0/1 y orden estable entre tracks. Conservar la fuente original y un ledger de operaciones no resueltas. SMPTE y operaciones fuera del perfil siguen devolviendo una incompatibilidad específica.

**Tests/gate P1:** fixtures RPN/Bank Select, bend de ±2 y ±12, ajustes en mitad de una nota, cambio de programa, loop con estado inicial distinto, sustain/reset, tracks simultáneos y entrada corrupta. `opening_02` conserva 1508 notas y 112 segundos; los 280 mensajes se resuelven sin `ignore_unsupported`; esto todavía no acredita playback sin banco.

### P2 — Biblioteca de instrumentos y banco automático

1. Elegir y registrar una biblioteca de calidad, su versión/hash, licencia y procedencia de mappings/samples. Comprobar los siete programas y siete sonidos de percusión del archivo, incluidos los rangos de nota y las modulaciones que usa cada instrumento. Resolver adquisición o muestras propias antes de cerrar el gate.
2. Implementar un importador SF2 acotado que preserve la fuente y extraiga presets, samples y definiciones al IR. Definir límites host de tamaño/chunks/zonas/moduladores antes del parser y comprobar índices/overflow. La [especificación original SoundFont 2.04](https://www.synthfont.com/sfspec24.pdf), publicada por E-mu y conservada por SynthFont, sirve de referencia; no se anuncia compatibilidad total por poder leer el contenedor.
3. Modelar bancos, regiones de nota/velocidad, capas, root/fine tune, envelopes, loops, grupos de exclusión de percusión y modulaciones necesarias. Preservar generators/modulators no ejecutables y diagnosticar su efecto. No elegir solo la primera zona coincidente cuando la fuente pide capas.
4. Ampliar el límite de catálogo de authoring independientemente del límite del banco cocinado. Una biblioteca grande no debe publicitar que cabe completa en los 128 mappings del perfil actual ni ensanchar todos los pools de consola a su tamaño.
5. Resolver automáticamente los instrumentos usados a una biblioteca predeterminada o elegida y crear/actualizar dependencias con identidad estable. Mostrar los fallos y permitir mapping explícito cuando exista una variante de banco o percusión distinta.
6. Generar una fixture de cada instrumento requerido para evaluar ataque, sustain, release y percusión; comparar contra una interpretación independiente de la misma biblioteca. No recurrir al sintetizador del sistema como fallback implícito.

**Tests/gate P2:** 14 asignaciones requeridas y todas sus notas/velocidades cubiertas; fuente/provenance preservadas; samples audibles adecuados a su mapping; SF2 corrupto, layers y exclusión de percusión probados; no hay fallback a programa 0 ni pérdida de un generator necesario sin resolver.

### P3 — Cooker, presets, optimización y migraciones

1. Llevar el IR a EPSQ/EPSB versionados; cocinar instrumentos desde la fuente de la biblioteca, con resampling filtrado, root/pitch correctos, downmix definido y SPU ADPCM. Mantener la receta legacy y sus bytes golden para SFX.
2. Recortar el derivado a instrumentos/regiones alcanzables con análisis conservador. Incluir Program/Bank Change, velocidades, capas, afinación, rango de `AudioSource.pitch` admitido y notas que atraviesan loops. Mantener un modo explícito de banco completo si se necesita reutilizarlo con secuencias adicionales.
3. Deduplicar únicamente datos compatibles en codificación, loops y estado de reproducción. Contabilizar alineación, bloques iniciales, metadatos y buffers en main RAM/SPU. No cobrar dos veces una reserva ni omitir copias del banco dentro del ejecutable.
4. Implementar presets y encoder effort; comprobar diferencias reales de coste/error/tiempo de cook. Desactivar opciones que no puedan cambiar el resultado del perfil en vez de generar controles decorativos.
5. Implementar `Optimize to budget` como búsqueda determinista y acotada sobre las modificaciones autorizadas en el panel. Proponer el resultado con el detalle de cambios y preview; fallar si no encuentra una conversión dentro de límites. Conservar notas y tempo.
6. Informar cambios de loop de sample por alineación ADPCM y medir la discontinuidad; ofrecer ajustes explícitos con escucha comparativa. No confundir loop del sample con loop de la canción.
7. Separar costes de calidad y de musicalidad: el codec SPU tiene bloques de 16 bytes por 28 muestras; encoder effort afecta el cook, mientras frecuencia/duración/layers pueden cambiar memoria o voces. Validar frente a la [documentación SPU](https://psx-spx.consoledev.net/soundprocessingunitspu/).

**Tests/gate P3:** mismos inputs/versiones producen mismos bytes e informe; cambios de biblioteca/preset/selección invalidan caché; podas conservadoras, budgets combinados con SFX, loops, fallo/cancelación y source preservation cubiertos. Cada preset produce un banco medido para el MIDI real; solo los que caben se marcan compatibles.

### P4 — Kernel compartido, síntesis y hardware PSX

1. Implementar los nuevos eventos/estados del perfil en el kernel compartido de host y PSX. Combinar bend configurado, tuning de canal, root/fine tune del sample y pitch del AudioSource con cálculo acotado. Detectar rangos que la SPU no puede reproducir; adaptar zonas al cocinar cuando sea posible.
2. Definir y probar las envelopes y modulaciones requeridas por la biblioteca elegida. Añadir vibrato/tremolo con estado acotado cuando formen parte del perfil. Preservar la evolución entre notas, controles, stop, reset y loops.
3. Implementar reverb PSX como recurso global con propietario, preset y reserva de memoria explícitos. El envío por voz no equivale automáticamente a un mezclador de efectos MIDI arbitrario. Declarar y probar cualquier adaptación de profundidad; los controles en cero deben dejar el efecto realmente desactivado.
4. Para chorus, phaser, filtros dinámicos u otros generators encontrados: fijar su tratamiento en la matriz. Ejecutarlos de forma acotada si se declaran parte del perfil, precalcular únicamente transformaciones válidas para ese uso con adaptación explícita, o devolver error. No retirar un bloqueo agregando un no-op. Los efectos necesarios para la biblioteca de Ironwood deben quedar resueltos antes de P6.
5. Contar voces físicas de capas, colas y efectos además de notas lógicas. Aplicar un techo agregado de música cuando haya varias secuencias y conservar prioridad/edad legacy de SFX. Definir una reserva de SFX solo si el allocator la garantiza.
6. Preservar reloj de audio, trabajo acotado de servicio/IRQ, ausencia de allocations no acotadas, pins, comandos tardíos, key-on seguro y retiro de bancos. No introducir lecturas continuas de CD para la secuencia residente.
7. Extender Target Preview con los mismos eventos y parámetros cocinados. Si todavía no reproduce interpolación SPU o reverb con fidelidad suficiente, identificarlo y usar captura de emulador como referencia del target; no etiquetarlo como reproducción física exacta.

**Tests/gate P4:** comparación con trazas esperadas independientes; pruebas de pitch medido y envelopes; cero notas colgadas; loops/stop/teardown, capas/exclusión, prioridad, agotamiento, múltiples secuencias, música↔XA y actividad de geometría/CD. Medir CPU máxima, gaps, drift, RAM y voces en carga normal y stress. El control musical correcto no se demuestra únicamente comparando dos backends del mismo kernel.

### P5 — Conversión integrada y tooltips

1. Integrar todas las opciones de la sección 3 con el importador, Inspector, Explorer, CLI y API de assets mediante el mismo servicio y validación. La UI no decide comportamientos que el cook de build desconozca.
2. Ofrecer presets, Advanced, estimación, optimización propuesta, A/B, Apply y restauración del borrador. Conservar source snapshot/UUID y advertir de los resultados desactualizados de forma visible.
3. Añadir el helper de ayuda con retardo de 2,0 segundos y un registro de texto por opción. Probar el retardo con reloj de prueba; no usar sleeps largos ni ejecutar el cook para abrir un tooltip.
4. Mantener una audition, cancelación y caché consistentes en crudo/importado. Los errores se publican una vez por intento en Console; cambiar de vista o esperar frames no vuelve a publicarlos.
5. En un MIDI sin banco, mostrar cómo seleccionar/instalar la biblioteca y continuar la conversión. En un banco incompleto, enumerar mappings faltantes y dirigir al ajuste pertinente.

**Tests/gate P5:** prueba real ImGui desde importación hasta Play con la biblioteca; guardar/reabrir/reimportar conserva settings; grid/lista/Inspector equivalentes; ninguna opción carece de ayuda, incluidas las deshabilitadas; 1,9 s no abre y 2,1 s abre la ayuda del control correcto; navegación/cancelación y workers tardíos cubiertos.

### P6 — Integración de Ironwood y primer resultado útil para este caso

1. Probar primero en una copia de trabajo del proyecto, con la fuente identificada por hash. Conservar los demás BGM y SFX.
2. Convertir `opening_02` con la biblioteca seleccionada, `ignore_unsupported=false`, Role Music y secuencia residente. Seleccionar el mejor preset que cumpla los presupuestos medidos y la comparación auditiva; registrar su receta exacta.
3. Escuchar los 112 segundos completos y al menos dos repeticiones consecutivas. Validar los bends, instrumentos, percusión, paneos, expresión, loop y las colas según el modo elegido.
4. Asignar el UUID existente al AudioSource `Title Music` manteniendo nombre, volumen, autoplay y acceso desde Options. Crear únicamente los assets/dependencias necesarios; el MIDI original y el UUID no cambian.
5. Incluir Title e iniciarlo en la ejecución de aceptación. Registrar las diferencias de configuración de Play antes de aplicarlas al proyecto real; no convertir accidentalmente la selección de dos escenas en una build final incompleta.
6. Probar Options/volumen, mover/aceptar/cancelar, empezar partida, salir/volver a Title y transiciones con los BGM XA existentes. Verificar MIDI residente con serial/host y en un paquete de disco independiente; cada modo usa sus capacidades reales.
7. Repetir la prueba con capturas/trazas de emulador y hardware disponible. Para música sola no deben existir notas denegadas ni robos inesperados. Con los SFX normales del menú debe conservarse el resultado musical acordado; el stress de agotamiento tiene expectativas separadas.

**Gate P6:** el MIDI real se convierte desde la UI y suena completo en Title; banco y preset publicados; integración y rutas de ejecución reproducibles; informe exacto de recursos; SFX y XA siguen funcionando. Hardware no ejecutado se mantiene pendiente y no se infiere de la captura de emulador.

### P7 — Fidelidad de adaptadores PSX y cierre completo

El checkpoint E actual preserva VAB/VH+VB y las variantes de SEQ/SEP, pero bloquea la reproducción de bancos Sony por fidelidad pendiente. Ese bloqueo es trabajo real del cierre A–E; no desaparece porque `opening_02` funcione.

1. Resolver con evidencia referencia de pitch/rate, envelopes nativas, gain/pan por etapa, capas, modulación/reverb y loops ADPCM predictivos de bancos Sony. Retener bloques y estado nativos cuando su reutilización PSX sea correcta; mantener también fuente y representación neutral suficiente. No inventar una frecuencia ni descartar capas para habilitar Play.
2. Mantener parsers y fixtures independientes para Sony SEQ, SEP y el perfil `converted-seq-le32-v1`. Preservar selección de canción, registros fuente y bloqueos de semánticas todavía no resueltas. Los nombres de código y UI siguen siendo genéricos.
3. Validar importación, selección, reimportación de ambas partes VH/VB y playback con fixtures propias. Usar el corpus externo únicamente en lectura para comprobar casos adicionales sin copiar sus assets al repositorio.
4. Cerrar las filas pendientes de fidelidad del perfil anunciado con evidencia de escucha/traza, memoria y timing. Una función que sigue bloqueada se reporta como pendiente y no permite declarar cerrado ese perfil.
5. Ejecutar suites relevantes completas, build de bins y export/build PSX limpio con las herramientas fijadas. Repetir regresiones legacy y la aceptación real después de los cambios de fidelidad.
6. Consolidar guía de usuario, tooltips, esquema/perfiles, migraciones, resultados y costes. Revisar diff para separar trabajo previo, no reformatear archivos ajenos ni incluir corpus privado. Commit/push solo conforme a la autorización vigente del usuario.

**Gate P7:** adaptadores anunciados reproducen el perfil documentado y sus gates están cerrados; entrega reproducible y sin regresiones nuevas; evidencia física separada y completa si se declara validación en hardware. Si falta ese equipo, entregar el hito software con la validación física pendiente, sin llamarlo validación total.

## 5. Migraciones y archivos a intervenir

### Reglas de compatibilidad

- Versionar el esquema de MusicSequence/SoundBank si cambia su significado. Los presets se identifican por ID y versión; guardar los valores efectivos, no solo un nombre que pueda cambiar de receta.
- Los bancos portables actuales migran al banco lógico 0 conservando programa, drum key, zonas y envelope legacy. No cambia el sonido de un asset antiguo solo por abrirlo y guardarlo.
- El estado por defecto para fuentes sin RPN sigue siendo el del perfil documentado. Los paquetes existentes que usaban `ignore_unsupported` conservan esa opción al cargar; las nuevas interpretaciones se señalan al recocinar y requieren un perfil/versionado explícito si cambian el resultado previo.
- Introducir EPSQ/EPSB v2 únicamente cuando el nuevo contrato lo requiera. Conservar lectura y fixtures v1 o una reconstrucción versionada desde el authoring; nunca interpretar bytes v1 como v2. Un paquete/export autónomo antiguo debe seguir funcionando con su runtime emparejado.
- Invalidar cachés por cambios semánticos, dependencias, receta y herramientas. Un fallo no publica un derivado parcial ni identifica un cook viejo como resultado de settings nuevos.
- Preservar UUID, snapshots, campos desconocidos y manifiestos de fuentes compuestas. No convertir un sample de una consola futura desde el ADPCM PSX como fuente maestra.

| Área | Archivos existentes principales; rutas nuevas propuestas |
|---|---|
| Parser y semántica | `src/midi.rs`, `src/sequence_ir.rs`, `src/sequence.rs`, `src/sequence_stream.rs` |
| Biblioteca/bancos | `src/sound_bank.rs`, `src/bank_compat.rs`, nuevo importador SF2 y manifiesto de biblioteca/provenance |
| Contrato/cook/cache | `src/audio_import.rs`, `src/audio_ir.rs`, `src/psx_sequence.rs`, `src/import_settings.rs`, `src/assets.rs`, `src/asset_manager.rs` |
| Preview | `src/sequence_preview.rs`, `src/preview_audio.rs`, `src/content_preview.rs` |
| UI | `src/asset_ui.rs`, `src/asset_inspector.rs`, `src/project_browser.rs`, helper de ayuda en `src/gui.rs` o módulo local equivalente |
| Runtime | `runtime/sequence_kernel.hpp`, `sequence_data.hpp`, `sequence_service.hpp`, `sequence_tables.hpp`, `sequence_clock.hpp`, `sequence_lock.hpp`, `runtime/audio.hpp`, `runtime/music.hpp`, integración de `runtime/main.cpp` |
| Adaptadores E | `src/vab_import.rs`, `src/sequence_compat.rs`, `src/bank_compat.rs` |
| Pruebas | Tests Rust de los módulos, `src/audio_contract_tests.rs`, `tests/runtime/sequence_kernel.cpp`, `tests/runtime/sequence_service.cpp`, `tests/integration/verify_psx_sequence.py`, `verify_psx_sequence_stress.py` y aceptación Ironwood por ruta externa |
| Usuario/evidencia | `docs/assets.md`, documentos de arquitectura/perfiles y `artifacts/` para logs/capturas no publicables |
| Integración de proyecto | Assets del banco, `opening_02.epokasset`, `assets/scenes/Title.epokmap`, `Ironwood.epokproject`; conservar la fuente MIDI |

Las rutas enumeran el alcance esperado, no una orden de modificar todos los archivos. Inspeccionar dependencias reales antes de cada cambio. No reorganizar globalmente módulos ni introducir un framework multiplataforma nuevo.

## 6. Validación, costes y cierre de cada fase

### Pruebas y ejecución

- Ejecutar tests enfocados después de cambios lógicos; suites Rust/nativas/GUI pertinentes antes de cerrar cada fase. Las pruebas de UI ignoradas por defecto se invocan explícitamente para ese gate.
- Usar fixtures pequeñas propias para semántica, bancos y errores; ejecutar la aceptación externa contra el hash del MIDI real. Un test que solo verifica que se suprimió el diagnóstico no prueba la conversión.
- Registrar comandos completos, herramientas, exit codes, test names, logs y resultado. Todas las invocaciones shell empiezan por `rtk`; no ejecutar builds y tests que bloqueen el mismo ejecutable simultáneamente en Windows.
- Mantener golden SFX SPU-ADPCM, staging, XA/disco, prioridades y teardown. Repetir pruebas amplias cuando haya un cambio o riesgo que lo justifique, sin ocultar los fallos baseline.
- Hacer comparación auditiva a nivel equivalente y comprobar afinación/timing con trazas/frecuencias. Definir tolerancias numéricas en P0 y ajustar el diseño a ellas, no declarar «suena bien» como única validación.

### Informe obligatorio del caso real

| Métrica | Evidencia requerida antes del cierre |
|---|---|
| Autoría y fuentes | UUID/hash del MIDI, biblioteca/provenance, profile/preset/settings y versiones |
| Conversión | Tiempo en frío/caliente, memoria host máxima de análisis/cook, cambios propuestos por el optimizador |
| Main RAM | Secuencia, metadatos, samples conservados en RAM, kernel/pools, buffers, linked total y heap/stack máximos |
| SPU RAM | Samples, alineación, SFX, captura y reverb; total del build, bytes libres y límite |
| Voces | Pico lógico; pico físico con capas/colas/efectos; robos, denegaciones y reservas realmente aplicadas |
| Timing | CPU máxima por servicio, gaps, drift durante canción/loops, deadlines incumplidos y carga usada |
| Calidad | Cobertura de programas/percusión, pitch, ataques/colas, discontinuidad de loops, clipping y comparación A/B |
| Media | Tamaño del paquete/EXE/disco, lecturas durante secuenciación y convivencia con geometría/XA |
| Plataforma | Captura e informe de emulador; modelo/configuración y resultados físicos separados |

La SPU tiene 512 KiB y 24 voces. El perfil actual deja 520 192 bytes tras reservar 4096 para captura, antes de una nueva reserva de reverb. El presupuesto de samples debe recalcularse cuando P4 añada efectos y siempre incluir SFX. Véase [hardware SPU](https://psx-spx.consoledev.net/soundprocessingunitspu/).

Los bancos actuales permanecen residentes para el conjunto incluido en el build. Si la receta aceptada no cabe tras la optimización explícita, medir primero la demanda y plantear como tarea acotada residencia por conjunto/escena con carga y retiro seguros. No introducirla preventivamente ni fingir streaming de bancos. No cerrar P6 con un exceso oculto.

En cada checkpoint entregar: comportamiento observable, archivos cambiados, migraciones, tests y fallos baseline, costes medidos/desconocidos, evidencia host/emulador/hardware, blockers y siguiente gate. `Unknown` y `pending` no se sustituyen por cero.

### Checklist final

- [ ] `opening_02` se importa/convierte desde la UI y reproduce sus 112 segundos, con al menos dos loops comprobados.
- [ ] Los 280 mensajes tienen semántica verificada; `ignore_unsupported=false`.
- [ ] Banco automático con los siete programas y siete percusiones, todos los rangos usados y provenance.
- [ ] Presets y opciones avanzadas producen cambios medidos; optimización explícita y reproducible.
- [ ] Cada opción tiene ayuda contextual a los dos segundos; errores visibles y Console sin duplicados por frame.
- [ ] Source/Target Preview, dependencias, cancelación, reimportación y cachés son consistentes.
- [ ] Title, Options, SFX, entrada/salida de escena y BGM XA pasan en Ironwood.
- [ ] Presupuestos completos de build y timing se cumplen; sin asignaciones ni lecturas continuas nuevas en playback.
- [ ] Compatibilidad legacy y adaptadores PSX anunciados cierran sus gates sin fidelidad pendiente escondida.
- [ ] Guía de uso y evidencias registradas; pruebas físicas realizadas para declarar hardware validado.

Este plan prioriza primero que el MIDI real sea utilizable en Title (P6) y después el cierre completo de las rutas PSX ya iniciadas (P7). No requiere decidir manualmente catorce mappings para cada importación ni conocer los nombres de los formatos internos.
