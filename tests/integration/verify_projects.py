"""Real CLI lifecycle and MIPS build in an external project, including spaces in paths."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / 'target/debug/epok-editor.exe'
FLAGS = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0


def run(cwd, *args, success=True):
    result = subprocess.run([str(EXE), *map(str, args)], cwd=cwd, capture_output=True,
                            text=True, timeout=120, creationflags=FLAGS)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result


def main():
    with tempfile.TemporaryDirectory(prefix='epok-projects-') as temporary:
        parent = Path(temporary).resolve()
        game = parent / 'My Sample Game'
        run(parent, '--build-psx', success=False)
        run(parent, '--project', ROOT, '--build-psx', success=False)
        run(parent, '--create-project', game, '--template', 'sample')
        manifest_path = project_manifest(game)
        manifest = manifest_path.read_bytes()
        run(parent, '--create-project', game, success=False)
        assert manifest_path.read_bytes() == manifest
        assert not any((game / p).exists() for p in ['src', 'runtime', 'third_party', '.tools', 'resources'])
        run(parent, '--project', game, '--build-psx')
        run(parent, '--project', manifest_path, '--build-psx')
        run(parent, manifest_path, '--build-psx')
        build = game / '.epok/build'
        assert (build / 'epok.ps-exe').read_bytes().startswith(b'PS-X EXE')
        timestamp = (build / 'scripts/Spinner.o').stat().st_mtime_ns
        run(parent, '--project', game, '--build-psx')
        assert (build / 'scripts/Spinner.o').stat().st_mtime_ns == timestamp
        # Only delete our generated cache within this test's resolved temporary root.
        cache = (game / '.epok').resolve()
        assert cache.is_relative_to(parent)
        shutil.rmtree(cache)
        moved = parent / 'Relocated Game'
        assert game.resolve().is_relative_to(parent) and moved.parent.resolve() == parent
        game.rename(moved)
        run(parent, '--project', moved, '--build-psx')
        assert (moved / '.epok/build/epok.ps-exe').is_file()
        # A different startup scene is read from the manifest, never a sample-path fallback.
        settings = documents.loads(manifest)
        old = moved / settings['startup_scene']
        settings['startup_scene'] = 'assets/scenes/Level1.epokmap'
        old.rename(moved / settings['startup_scene'])
        documents.write_text(project_manifest(moved), json.dumps(settings))
        run(parent, '--project', moved, '--bake-lighting')
        assert documents.loads((moved / settings['startup_scene']).read_text())['bake']
        settings['format_version'] = 999
        documents.write_text(project_manifest(moved), json.dumps(settings))
        run(parent, '--project', moved, '--build-psx', success=False)
        basic = parent / 'Basic'
        run(parent, '--create-project', basic)
        run(parent, '--project', basic, '--build-psx')
        assert not list((basic / 'assets/scripts').glob('*.cpp'))
        # Reading an old manifest does not rewrite it. Migration and recovery
        # preserve settings, then use the same staging/export services.
        descriptor=project_manifest(basic)
        contents=descriptor.read_bytes()
        legacy=basic/'ProjectSettings/project.json'
        descriptor.rename(legacy)
        run(parent,'--project',basic,'--build-psx')
        assert legacy.read_bytes()==contents and not descriptor.exists()
        run(parent,'--migrate-project',basic)
        assert project_manifest(basic).read_bytes()==contents and not legacy.exists()
        legacy.write_bytes(contents)
        run(parent,'--project',basic,'--build-psx',success=False)
        documents.write_text(descriptor, '{interrupted')
        run(parent,'--recover-project',basic,'--prefer','descriptor',success=False)
        run(parent,'--recover-project',basic,'--prefer','legacy')
        assert legacy.read_bytes()==contents and not descriptor.exists()
        run(parent,'--migrate-project',basic)
        run(parent,'--project',basic,'--build-psx')
        export=Path(run(parent,'--project',basic,'--export-psx').stdout.strip())
        assert (export/'Makefile').is_file() and (export/'scene.hh').is_file()
        duplicate=basic/'Duplicate.EPOKPROJECT'
        duplicate.write_bytes(contents)
        run(parent,'--project',basic,'--build-psx',success=False)
    print('PASS external creation, no-project gate, overwrite protection, build, incremental cache, relocation, scene settings and basic template')


if __name__ == '__main__':
    main()
