# Implementation Agent Prompt: Epok PSX-First Portable Audio

Copy the prompt below into the implementation task. It is intentionally scoped
to PSX. N64, PS2 and GameCube are architectural constraints only and must not be
implemented by this task.

```text
Trabaja en el repositorio:

D:\GitProjects\GameEngines\Epok\EpokEngine

Objetivo
========

Implementa incrementalmente el sistema de audio portable descrito en:

- docs/architecture/cross-platform-audio-implementation-plan.md
- docs/architecture/multi-platform-architecture.md

El primer resultado completo debe permitir que el usuario:

1. Copie o importe WAV, FLAC, OGG, MP3, MID o MIDI dentro de assets/.
2. Detecte y reproduzca fuentes compatibles desde el Explorer antes de importar.
3. Importe la fuente a un .epokasset autoritativo que preserve fuente, UUID,
   settings y dependencias.
4. Seleccione el asset y edite desde Inspector intención y residencia sin ver
   formatos internos como decisión principal:
   - Role: SFX, Music, Ambience o Dialogue.
   - Load Mode: Auto, Resident o Stream.
   - calidad/canales/trim/normalize/loop.
5. Vea la representación resuelta, compatibilidad y coste del target elegido.
6. Importe un MIDI como MusicSequence, le asigne un SoundBank, lo reproduzca en
   el editor y lo cocine para reproducción secuenciada en PSX.
7. Mantenga funcionando los SFX SPU-ADPCM y BGM/XA existentes sin regresiones.

Instrucciones obligatorias del repositorio
==========================================

1. Lee primero C:\Users\Adolfo\.codex\RTK.md y obedece sus instrucciones. Todos
   los comandos de shell deben comenzar con `rtk`.
2. Lee completos los dos documentos de arquitectura indicados antes de editar.
3. Busca y obedece cualquier AGENTS.md aplicable.
4. Inspecciona `git status --short` antes de trabajar. El working tree ya puede
   contener muchos cambios del usuario. No reviertas, formatees, muevas ni
   sobrescribas cambios ajenos. Los documentos de architecture/ pueden aparecer
   como untracked y siguen siendo fuentes autoritativas para esta tarea.
5. No uses `git reset --hard`, `git checkout --` ni limpiezas destructivas.
6. No hagas commits salvo que el usuario lo solicite explícitamente.
7. Usa apply_patch para ediciones manuales. Los formateadores solo pueden tocar
   archivos realmente modificados por esta tarea.

Decisiones de producto no negociables
=====================================

- La extensión fuente no decide residencia: un MP3 puede cocinarse residente y
  un WAV puede cocinarse streaming.
- WAV/FLAC/OGG/MP3 son fuentes de audio muestreado. MIDI es una fuente de
  secuencia y necesita un SoundBank; MIDI no contiene instrumentos.
- XA, SPU ADPCM, VAB, SEQ y SEP son detalles o adaptadores del target PSX, nunca
  el esquema universal de authoring.
- Conserva la fuente original y un IR neutral suficiente para que un futuro
  target pueda cocinarse independientemente. En esta tarea cocina únicamente
  PSX; no crees derivados N64, PS2 o GC.
- La detección puede hacer probe y preview, pero no publica un asset autoritativo
  ni cocina todos los targets sin que el usuario importe/confirme settings.
- El cook específico se ejecuta al solicitar target/profile y usa caché con
  target, settings, versiones, dependencias y herramientas en la identidad.
- `Auto` debe resolverse de forma determinista y visible. `Resident` o `Stream`
  explícitos nunca cambian silenciosamente por falta de presupuesto.
- Un asset puede ser authoring válido y no ser compatible con cierto target.
  Ese target debe mostrar y devolver un diagnóstico preciso; otros targets
  continúan siendo utilizables.
- Los eventos de secuencia permanecen residentes en el primer perfil. El banco
  posee una política de residencia separada.
- El timing MIDI usa el reloj/servicio de audio, no el frame renderizado.
- Runtime normal sin allocations no acotadas; callbacks/interrupts no invocan
  gameplay ni publican memoria antes de retirar DMA/consumidores.
- No implementes N64, PS2 o GameCube: tampoco placeholders, no-op backends,
  cookers, codecs, SDK dependencies, packaging, preview modes ni capabilities
  publicitadas. Esas plataformas requieren otra autorización futura.

Orden de implementación
=======================

Ejecuta las fases A-D del plan en orden. Cada fase debe ser un cambio revisable
con tests antes de avanzar. No mezcles toda la migración en un rewrite.

Fase A: contrato, baseline y migración
-------------------------------------

1. Captura fixtures/golden tests del estado actual:
   - paquetes de audio legacy;
   - bytes SPU-ADPCM generados;
   - staging/declaraciones del banco;
   - conversión/staging XA y layout requerido;
   - selección de voces, prioridad y teardown relevantes.
2. Introduce AudioRole y LoadMode en el esquema versionado.
3. Migra settings legacy exactamente:
   - Sfx => role=Sfx, load_mode=Resident.
   - Music => role=Music, load_mode=Stream.
4. Separa opciones portables de overrides namespaced por target.
5. Conserva unknown fields y round-trip de documentos antiguos.
6. Añade capability/result records mínimos para que Inspector pueda mostrar el
   modo resuelto sin acoplarse a XA.

Gate A:

- Los proyectos antiguos cargan y guardan correctamente.
- Los bytes y comportamiento PSX existentes protegidos por fixtures no cambian.
- El modelo expresa resident music y streamed ambience.

Fase B: IR neutral, detección y preview
--------------------------------------

1. Extrae DecodedAudioIr o una frontera equivalente para PCM, canales,
   trim/normalize y loop markers.
2. Haz que los cookers PSX actuales consuman el IR/fuente original sin alterar
   sus salidas golden.
3. Extiende scanner, file picker, drag/drop, source_kind, iconos, filtros y
   mensajes a `.mid` y `.midi`.
4. Mantén preview y waveform actuales para WAV/FLAC/OGG/MP3 crudos e importados.
5. Añade identidad `Source Preview` vs `Target Preview` al UI/cache, aunque el
   primer Target Preview pueda implementarse solo para perfiles PSX ya
   decodificables.
6. Preserva cancelación, límite de una audition y protección contra workers
   tardíos/stale.

Gate B:

- La extensión fuente y LoadMode son independientes.
- Detección/import/reimport/play de las fuentes actuales sigue funcionando.
- MIDI aparece como fuente soportada y puede ser inspeccionado sin publicarse.

Fase C: MusicSequence, SoundBank y preview host
-----------------------------------------------

1. Implementa parser SMF acotado para formato 0 y 1:
   - PPQN;
   - note on/off y note-on velocity 0;
   - program change;
   - CC volume, pan, expression y sustain;
   - pitch bend con política documentada;
   - tempo/time signature;
   - canal 10 mediante mappings explícitos;
   - running status y end-of-track;
   - markers `loop_start` y `loop_end`.
2. Fusiona tracks tipo 1 por tick absoluto con orden estable documentado para
   eventos simultáneos. Rechaza SMPTE inicialmente. Reporta SysEx, aftertouch,
   RPN/NRPN y metaeventos no soportados sin descartarlos silenciosamente.
3. Añade Kind/Settings/dependencias para MusicSequence y SoundBank con migración,
   UUID, duplicate/move/relink y invalidación de caché.
4. Implementa SoundBank portable: programas, drum keys, zones, key/velocity
   ranges, root key, fine tune, gain/pan, ADSR y sample loops.
5. Implementa análisis de duración, tempo map, programas usados, instrumentos
   faltantes y peak polyphony.
6. Implementa un SequenceKernel común, allocation-free en runtime, y un backend
   host para audition desde Explorer/Inspector.
7. Añade Project Default SoundBank. No uses el sintetizador del OS como fallback
   silencioso. Si no hay banco, Play debe explicar cómo asignarlo.
8. No habilites un banco bundled sin samples propios o licencia/provenance
   redistribuible registrada.

Gate C:

- Un MIDI puede importarse, asignarse a un banco, reproducirse en Explorer,
  moverse y reimportarse sin perder identidad.
- MIDI sin banco o con programas faltantes produce diagnósticos accionables.
- Parser, orden temporal, loops, sustain y cancelación tienen unit tests.

Fase D: cooker y runtime secuenciado PSX
---------------------------------------

1. Define payloads versionados Epok PSX para sequence y bank. No nombres todo
   `.SEP` ni dependas de libSnd.
2. Cocina samples del SoundBank desde PCM original a SPU ADPCM, deduplica cuando
   sea seguro y contabiliza main RAM/SPU RAM por separado.
3. Cocina SequenceIr a un event stream acotado con fixtures deterministas.
4. Integra el SequenceKernel con un reloj de audio medido, no audio_tick por
   frame si eso introduce jitter musical.
5. Generaliza solo lo necesario el allocator existente para owner SFX/sequence,
   conservando prioridad/edad y comportamiento legacy. El límite inicial de
   música puede ser 16, configurable y reportado, no una promesa universal.
6. Integra AudioSource/PlayableAudio, Blueprint/API, play/stop/status, gain,
   pitch aplicable, loops, autoplay, enable/disable y scene teardown.
7. Pin de sequence y SoundBank hasta retirar todas las notas/comandos físicos.
8. Muestra en Inspector/build report:
   - sequence RAM;
   - sample/SPU RAM;
   - voice limit y peak polyphony;
   - package/disc cost;
   - resolved mode y target errors.
9. Prueba música secuenciada con SFX simultáneos, voice exhaustion, cambio de
   escena, loops, stop, cambios XA<->sequence, geometry/CD activity y fallos de
   presupuesto.

Gate D / primer milestone usable:

- Un MusicSequence + SoundBank portable se reproduce en host, emulador PSX y
  hardware PSX cuando esté disponible.
- Durante playback secuenciado no hay lecturas continuas del CD.
- SFX/XA legacy permanece funcional y sus regresiones están cubiertas.
- Los costes de RAM, SPU y voces son visibles y los excesos fallan claramente.
- Evidencia de hardware no ejecutada permanece marcada pending.

Trabajo PSX posterior
=====================

Después de D, implementa E (importadores Sony VAB/VH+VB, SEQ/SEP y SEQ/SEP convertido (LE32))
solo sobre el IR ya estable. Detecta formatos por cabecera/estructura, no por
extensión. Mantén cada variante identificada y testeada por separado.

Después de completar y validar E, detente. No comiences T27, T28, T42 ni ningún
trabajo N64, PS2 o GameCube. La única obligación multiplataforma de esta tarea es
dejar fuente, authoring, IR, cache identity y contratos lógicos sin acoplamiento
innecesario a PSX. No añadas abstracciones especulativas que el trabajo PSX no
necesite.

Archivos que debes inspeccionar antes de diseñar cambios
========================================================

- src/audio_import.rs
- src/audio_decode.rs
- src/music.rs
- src/audio.rs
- src/assets.rs
- src/import_settings.rs
- src/asset_manager.rs
- src/asset_ui.rs
- src/asset_inspector.rs
- src/content_preview.rs
- src/preview_audio.rs
- src/project_browser.rs
- runtime/audio.hpp
- runtime/music.hpp
- runtime/main.cpp
- docs/assets.md
- docs/architecture/multi-platform-architecture.md
- docs/architecture/cross-platform-audio-implementation-plan.md

Consulta el corpus externo de referencia registrado en artifacts/audio-phase-e-preparation/corpus-report.json en modo read-only únicamente
si necesitas fixtures o evidencia de VAB/SEQ/SEP. No copies código cuya licencia
o propiedad no esté clara. Implementa parsers/encoders propios a partir de las
especificaciones y fixtures verificadas.

Validación y disciplina
=======================

- Ejecuta tests enfocados después de cada edición lógica y las suites relevantes
  antes de cerrar una fase.
- Ejecuta rustfmt/clippy solo con alcance controlado y no reformatees archivos
  ajenos accidentalmente.
- Conserva output, exit code y nombres de tests ejecutados.
- No declares hardware validado a partir de emulador.
- No escondas warnings, unsupported features, unknown memory o underruns como 0.
- Si un prerequisite del roadmap no existe, termina el trabajo que no dependa de
  él y documenta el bloqueo exacto; no simules la implementación faltante.
- Actualiza documentación de usuario y arquitectura cuando el comportamiento
  real cambie.

Entrega final esperada
======================

Al finalizar cada fase informa:

1. Outcome funcional observable por el usuario.
2. Archivos modificados y decisiones de schema/runtime.
3. Migraciones y compatibilidad legacy.
4. Tests ejecutados con resultados.
5. Costes medidos o todavía desconocidos.
6. Evidencia de emulador/hardware y qué sigue pendiente.
7. Riesgos o blockers restantes y la siguiente fase habilitada.

No te limites a describir o prototipar si existen pasos seguros dentro del scope:
implementa, verifica y deja el repositorio en un checkpoint revisable. No amplíes
el alcance a una reescritura general del engine.
```
