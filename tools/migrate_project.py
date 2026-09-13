"""Copy a UniQo game into a new Epok project, preserving the original directory.

Usage: python tools/migrate_project.py OLD_GAME NEW_GAME
Only authoring files and local view/import preferences are copied. Build caches,
exports, reports, Git metadata, and machine-specific tool overrides are omitted.
"""
from pathlib import Path
import argparse
import json
import struct
import epok_documents as documents

RENAMES = (
    ('.uniqo.json', '.epokmap'), ('.blueprint.json', '.epokbp'),
    ('.script.json', '.epokscript'), ('.uniqoproject', '.epokproject'),
    ('.uniqoasset', '.epokasset'), ('scenes.json', 'Maps.epoksettings'),
    ('scene-view.json', 'SceneView.epokprefs'), ('import-state.json', 'ImportState.epokprefs'),
    ('blueprint-breakpoints.json', 'Breakpoints.epokprefs'),
)


def renamed(text):
    for before, after in RENAMES:
        text = text.replace(before, after)
    return text


def source_code(text):
    return (text.replace('uniqo', 'epok').replace('UniQo', 'Epok')
            .replace('UNIQO', 'EPOK').replace('UQ_', 'EPOK_').replace('uq_', 'epok_'))


def values(value):
    if isinstance(value, str):
        return source_code(renamed(value))
    if isinstance(value, list):
        return [values(item) for item in value]
    if isinstance(value, dict):
        return {key: values(item) for key, item in value.items()}
    return value


def migrate(source, destination):
    source = Path(source).resolve(strict=True)
    destination = Path(destination).resolve()
    if not source.is_dir() or destination.exists() or destination.is_relative_to(source):
        raise ValueError('Choose an existing source game and a new destination outside it.')
    descriptors = list(source.glob('*.uniqoproject'))
    legacy = source / 'ProjectSettings/project.json'
    if legacy.is_file():
        descriptors.append(legacy)
    if len(descriptors) != 1:
        raise ValueError('Expected exactly one UniQo project descriptor or legacy manifest.')
    manifest = values(json.loads(descriptors[0].read_text(encoding='utf-8-sig')))
    name = manifest.get('name', '')
    if not name or name in {'.', '..'} or any(c in name for c in '<>:"/\\|?*'):
        raise ValueError('The project name is not a portable descriptor filename.')
    scene = Path(manifest['startup_scene'])
    if scene.is_absolute() or '..' in scene.parts or not scene.parts[:2] == ('assets', 'scenes'):
        raise ValueError('Startup map must remain inside assets/scenes.')
    outputs = {Path(f'{name}.epokproject'): documents.dumps(manifest).encode('utf-8')}
    for directory in ['assets', 'ProjectSettings', 'UserSettings']:
        folder = source / directory
        if not folder.exists():
            continue
        if folder.is_symlink():
            raise ValueError(f'Linked authoring directory: {folder}')
        for file in sorted(folder.rglob('*')):
            if file.is_symlink():
                raise ValueError(f'Linked authoring path: {file}')
            if not file.is_file() or file == descriptors[0]:
                continue
            relative = Path(renamed(file.relative_to(source).as_posix()))
            data = file.read_bytes()
            if relative.suffix in documents.EXTENSIONS:
                data = documents.dumps(values(json.loads(data))).encode('utf-8')
            elif relative.suffix == '.epokasset':
                if len(data) < 16 or data[:8] != b'UNIQOAS1':
                    raise ValueError(f'Invalid legacy asset package: {file}')
                meta_size, source_size = struct.unpack_from('<II', data, 8)
                if 16 + meta_size + source_size != len(data):
                    raise ValueError(f'Invalid legacy asset lengths: {file}')
                data = b'EPOKAS01' + data[8:]
            elif relative.suffix in {'.hpp', '.cpp', '.h', '.hh', '.cc', '.inl'}:
                data = source_code(data.decode('utf-8-sig')).encode('utf-8')
            if relative in outputs:
                raise ValueError(f'Duplicate destination path: {relative}')
            outputs[relative] = data
    if scene not in outputs:
        raise ValueError(f'Missing startup map: {scene}')
    ignore = source / '.gitignore'
    ignores = renamed(ignore.read_text('utf-8')) if ignore.is_file() else ''
    ignores = ignores.replace('.uniqo', '.epok').replace('uniqo.local.json', 'Local.epokconfig')
    outputs[Path('.gitignore')] = (ignores + '\n/.epok/\n/UserSettings/\n/exports/\n/artifacts/\n/Local.epokconfig\n').encode()
    # Validate and prepare every output before creating the destination.
    destination.mkdir(parents=True, exist_ok=False)
    for relative, data in outputs.items():
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    return destination / f'{name}.epokproject'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    args = parser.parse_args()
    print(migrate(args.source, args.destination))
