import json
from pathlib import Path
import struct
import tempfile
import unittest
import epok_documents as documents
from migrate_project import migrate


class MigrationTests(unittest.TestCase):
    def test_yaml_strings_do_not_become_legacy_booleans_or_dates(self):
        self.assertEqual(documents.loads('name: on\nlabel: yes\ndate: 2026-09-08\nenabled: true'),
                         {'name': 'on', 'label': 'yes', 'date': '2026-09-08', 'enabled': True})

    def test_migration_preserves_source_ids_payloads_and_updates_references(self):
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / 'old'
            (source / 'ProjectSettings').mkdir(parents=True)
            (source / 'assets/scenes').mkdir(parents=True)
            (source / 'assets/scripts').mkdir(parents=True)
            manifest = {'name': 'Test Game', 'startup_scene': 'assets/scenes/Main.uniqo.json'}
            (source / 'ProjectSettings/project.json').write_text(json.dumps(manifest))
            (source / 'assets/scenes/Main.uniqo.json').write_text(json.dumps({
                'entities': [{'id': 'keep-id', 'asset': 'assets/Stone.uniqoasset'}]}))
            (source / 'assets/scripts/Enemy.hpp').write_text(
                '#include "uniqo.hpp"\nclass UQ_CLASS() Enemy: public uniqo::Behaviour {};')
            payload = b'unchanged source bytes'
            meta = b'{"id":"asset-id"}'
            package = b'UNIQOAS1' + struct.pack('<II', len(meta), len(payload)) + meta + payload
            (source / 'assets/Stone.uniqoasset').write_bytes(package)
            before = {p.relative_to(source): p.read_bytes() for p in source.rglob('*') if p.is_file()}
            destination = Path(temp) / 'new'
            descriptor = migrate(source, destination)
            self.assertEqual(documents.loads(descriptor.read_text())['startup_scene'], 'assets/scenes/Main.epokmap')
            text = (destination / 'assets/scenes/Main.epokmap').read_text()
            self.assertFalse(text.startswith('{'))
            self.assertEqual(documents.loads(text)['entities'][0], {'id': 'keep-id', 'asset': 'assets/Stone.epokasset'})
            self.assertEqual((destination / 'assets/Stone.epokasset').read_bytes(), b'EPOKAS01' + package[8:])
            self.assertIn('EPOK_CLASS()', (destination / 'assets/scripts/Enemy.hpp').read_text())
            self.assertEqual(before, {p.relative_to(source): p.read_bytes() for p in source.rglob('*') if p.is_file()})
            with self.assertRaises(ValueError):
                migrate(source, destination)

    def test_invalid_startup_does_not_create_destination(self):
        with tempfile.TemporaryDirectory() as temp:
            source = Path(temp) / 'old'
            source.mkdir()
            (source / 'Game.uniqoproject').write_text(json.dumps({'name': 'Game', 'startup_scene': '../escape.uniqo.json'}))
            destination = Path(temp) / 'new'
            with self.assertRaises(ValueError):
                migrate(source, destination)
            self.assertFalse(destination.exists())


if __name__ == '__main__':
    unittest.main()
