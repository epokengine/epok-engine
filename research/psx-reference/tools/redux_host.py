"""Host discovery shared by the owned-emulator helpers.

Resolves the engine checkout, the PCSX-Redux executable its machine-local
configuration names, and the OpenBIOS image that emulator ships, the same way
the editor's Play does. Nothing here launches or modifies anything.
"""
from pathlib import Path
import shutil
import sys


def default_engine():
    """The checkout containing these tools: research/psx-reference/tools."""
    return Path(__file__).resolve().parents[3]


def configured_emulator(engine):
    """The emulator named by Local.epokconfig, else Editor.epokconfig.

    A value with a path separator is relative to the engine; a bare name is
    looked up on PATH. The Windows distribution's pcsx-redux.exe is an updater
    that spawns pcsx-redux.main, so the latter is preferred when present.
    """
    engine = Path(engine)
    sys.path.insert(0, str(engine / "tools"))
    import epok_documents
    for name in ("Local.epokconfig", "Editor.epokconfig"):
        config = engine / name
        if not config.is_file():
            continue
        value = str(epok_documents.loads(config.read_text(encoding="utf-8")).get("emulator") or "")
        if not value:
            continue
        if "/" in value or "\\" in value:
            emulator = Path(value) if Path(value).is_absolute() else engine / value
        else:
            found = shutil.which(value)
            if not found:
                raise FileNotFoundError(f"{name} names emulator {value!r}, which is not on PATH")
            emulator = Path(found)
        if emulator.name == "pcsx-redux.exe" and emulator.with_name("pcsx-redux.main").is_file():
            emulator = emulator.with_name("pcsx-redux.main")
        if not emulator.is_file():
            raise FileNotFoundError(f"{name} names emulator {emulator}, which does not exist")
        return emulator.resolve()
    raise FileNotFoundError("No emulator configured; run the repository setup tool first")


def openbios(executable_dir):
    """OpenBIOS beside the executable or in the macOS bundle, fast-boot first."""
    executable_dir = Path(executable_dir)
    resources = executable_dir / "../Resources/share/pcsx-redux/resources"
    for folder in (executable_dir, resources):
        for name in ("openbios-fastboot.bin", "openbios.bin"):
            if (folder / name).is_file():
                return (folder / name).resolve()
    raise FileNotFoundError(f"No OpenBIOS image beside {executable_dir} or in its application bundle")


def emulator_and_bios(engine):
    emulator = configured_emulator(engine)
    return emulator, openbios(emulator.parent)
