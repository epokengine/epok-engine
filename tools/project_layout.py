"""Descriptor-aware fixture paths; mirror the editor's unambiguous discovery rule."""
from pathlib import Path


def project_manifest(project):
    path = Path(project).resolve(strict=True)
    root = path.parent if path.is_file() else path
    descriptors = sorted(p for p in root.iterdir() if p.suffix.lower() == '.epokproject')
    legacy = root / 'ProjectSettings/project.json'
    if len(descriptors) > 1 or (descriptors and legacy.exists()):
        raise ValueError('Ambiguous Epok project descriptors')
    selected = descriptors[0] if descriptors else legacy
    if not selected.is_file() or (path.is_file() and selected != path):
        raise ValueError('Not an Epok project folder or active descriptor')
    return selected
