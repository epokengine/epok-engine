"""Exercise the real command/dependency/marker methods, host only."""
from pathlib import Path
from types import SimpleNamespace
import importlib.util
import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p / 'tools/epok_documents.py').is_file())
sys.path.insert(0,str(ROOT/'tools'))
import epok_documents as documents
spec=importlib.util.spec_from_file_location('input_fix',ROOT/'tools/gameplay_helpers_acceptance.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)


class GeneratedInputTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='luarc-host-',dir=HERE)
        self.root=Path(self.temp.name);self.r=m.Runner.__new__(m.Runner)
        self.r.root=self.root;self.r.out=self.root/'output';self.r.engine='a'*64;self.r.documents=documents
        self.r.generation_inputs={'fake-builder':'host-only'};self.r.ledger={'rows':{}};self.r.save=lambda:None;self.r.ensure_frozen=lambda:None
        self.row={'template':'basic','flavor':'cpp','mode':'native_cpp'};self.p=self.r.project(self.row)
        (self.p/'assets/scripts').mkdir(parents=True);(self.p/'ProjectSettings').mkdir()
        (self.p/'assets/scripts/Game.hpp').write_text('actual source baseline\n',encoding='utf-8')
        (self.p/'ProjectSettings/Game.json').write_text('{}',encoding='utf-8')
        (self.p/'Test.epokproject').write_text(documents.dumps({'lua_execution':'native_cpp','gameplay':'cpp','startup_scene':'assets/Main.epokmap'}),encoding='utf-8')
        self.r.contract=lambda row:{}
    def tearDown(self):self.temp.cleanup()
    def marker(self,include_luarc=False):
        if include_luarc:(self.p/'.luarc.json').write_text('{"old":"LuaLS"}',encoding='utf-8')
        values,_,_=m.project_inputs(self.p,documents)
        if include_luarc:values['.luarc.json']=m.digest(self.p/'.luarc.json')
        marker=self.p/'.p10-generation.json'
        m.write(marker,dict(engine_sha256=self.r.engine,generation_sha256=self.r.generation_key(self.row),inputs=values))
        return marker
    def prepare_without_editor(self):
        # Skip only the unrelated reflection process; execute actual marker,
        # generation-key and authored-source comparison code unchanged.
        self.r.valid=lambda row,name:name=='reflection';self.r.prepare(self.row)
    def host_write(self,path,text):
        return m.Runner.command(self.r,self.row,'host-input-regression',[sys.executable,'-c',
          'from pathlib import Path;Path('+repr(str(path))+').write_text('+repr(text)+',encoding="utf-8")'])
    def test_real_command_accepts_late_editor_luarc_without_dependency_change(self):
        self.marker();before=self.r.dependency(self.row)
        self.host_write(self.p/'.luarc.json','{"runtime.version":"Lua 5.2","workspace.library":[".epok/lua"]}')
        self.assertEqual(before,self.r.dependency(self.row))
    def test_real_command_still_rejects_actual_code_edit(self):
        self.marker()
        with self.assertRaisesRegex(RuntimeError,'Authored project inputs changed'):
            self.host_write(self.p/'assets/scripts/Game.hpp','changed actual C++ source')
    def test_legacy_marker_with_luarc_is_compatible_without_rewriting_marker(self):
        marker=self.marker(True);before=marker.read_bytes()
        (self.p/'.luarc.json').write_text('{"new":"editor only"}',encoding='utf-8')
        self.prepare_without_editor();self.assertEqual(marker.read_bytes(),before)
    def test_legacy_marker_without_luarc_accepts_generated_file(self):
        self.marker();(self.p/'.luarc.json').write_text('{}',encoding='utf-8');self.prepare_without_editor()
    def test_marker_still_rejects_actual_source_edit(self):
        self.marker(True);(self.p/'assets/scripts/Game.hpp').write_text('changed',encoding='utf-8')
        with self.assertRaisesRegex(RuntimeError,'Generated project changed'):self.prepare_without_editor()
    def test_marker_generation_key_is_not_bypassed(self):
        path=self.marker(True);value=m.read(path);value['generation_sha256']='b'*64;m.write(path,value)
        with self.assertRaisesRegex(RuntimeError,'Generated project changed'):self.prepare_without_editor()
    def test_only_exact_root_config_is_excluded(self):
        for path in ['assets/scripts/.luarc.json','ProjectSettings/.luarc.json','settings.json']:
            before=self.r.dependency(self.row);p=self.p/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('{}',encoding='utf-8')
            self.assertNotEqual(before,self.r.dependency(self.row),path)
    def test_code_and_settings_edits_stale_source_build_and_export(self):
        self.marker();dependency=self.r.dependency(self.row)
        for name in ['source','build','export']:self.r.row(self.row)['stages'][name]=m.stage('passed',dependency_sha256=dependency,files={})
        for name in ['source','build','export']:self.assertTrue(m.Runner.valid(self.r,self.row,name))
        (self.p/'ProjectSettings/Game.json').write_text('{"gravity":99}',encoding='utf-8')
        for name in ['source','build','export']:self.assertFalse(m.Runner.valid(self.r,self.row,name))
        with self.assertRaisesRegex(RuntimeError,'current archived build'):m.Runner.export(self.r,self.row)


class AdapterSourceTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix="adapter-closure-host-",dir=HERE)
        self.root=Path(self.temp.name);self.addCleanup(self.temp.cleanup)
        self.adapters={"basic":{"command":["python","{root}/adapters/driver.py"],
            "source_files":["adapters/driver.py","adapters/helper.py","adapters/observe.lua"]}}
        for name in self.adapters["basic"]["source_files"]:
            path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_text("baseline "+name,encoding="utf-8")
        (self.root/"editor.exe").write_bytes(b"host-only fake editor: never executed")
        (self.root/"source.rs").write_text("frozen engine input",encoding="utf-8")
        m.write(self.root/"snapshot.json",{"source.rs":m.digest(self.root/"source.rs")})
        templates=["basic","sample","third-person","platformer-2d","dungeon","dungeon-complete"]
        m.write(self.root/"queue.json",dict(final_matrix=[dict(template=t,flavor=f,mode="native_cpp")
            for t in templates for f in ["cpp","blueprint","lua"]],matrix_contract=dict(scenarios={}),fixed_gates={}))
        self.row=dict(template="basic",flavor="cpp",mode="native_cpp")
        self.args=m.parser().parse_args(["--root",str(self.root),"--editor",str(self.root/"editor.exe"),
            "--output",str(self.root/"output"),"--queue",str(self.root/"queue.json"),
            "--source-snapshot",str(self.root/"snapshot.json"),"--adapters",str(self.root/"adapters.json"),
            "--nugget",str(self.root/"nugget"),"--prepare"])
        # Ctor/identity tests only: no command, editor, compiler or console runs.
        self.environment=patch.dict(os.environ,{"EPOK_RUNTIME_OPT":"-O2","EPOK_SCRIPT_OPT":"-O2"})
        self.environment.start();self.addCleanup(self.environment.stop)
    def runner(self):
        m.write(self.root/"adapters.json",self.adapters)
        runner=m.Runner(self.args);project=runner.project(self.row)
        (project/"assets/scripts").mkdir(parents=True,exist_ok=True)
        (project/"assets/scripts/Game.hpp").write_text("same authored rules",encoding="utf-8")
        (project/"Game.epokproject").write_text(documents.dumps(dict(gameplay="cpp",lua_execution="native_cpp")),encoding="utf-8")
        return runner
    def test_entry_and_transitive_edits_change_inventory_engine_and_dependency(self):
        for name in self.adapters["basic"]["source_files"]:
            with self.subTest(source=name):
                before=self.runner();inventory=dict(before.adapter_sources);dependency=before.dependency(self.row)
                path=self.root/name;path.write_text(path.read_text(encoding="utf-8")+"\nchanged",encoding="utf-8")
                after=self.runner()
                self.assertNotEqual(inventory,after.adapter_sources)
                self.assertNotEqual(before.engine,after.engine)
                self.assertNotEqual(dependency,after.dependency(self.row))
                with self.assertRaisesRegex(RuntimeError,"Frozen scenario adapter source changed"):
                    before.ensure_frozen()
                after.ensure_frozen()
    def test_transitive_change_invalidates_previous_source_build_and_export(self):
        before=self.runner();dependency=before.dependency(self.row)
        for name in ["source","build","export"]:
            before.row(self.row)["stages"][name]=m.stage("passed",dependency_sha256=dependency,files={})
            self.assertTrue(before.valid(self.row,name))
        before.save()
        (self.root/"adapters/helper.py").write_text("changed helper",encoding="utf-8")
        after=self.runner()
        for name in ["source","build","export"]:self.assertFalse(after.valid(self.row,name))
        with self.assertRaisesRegex(RuntimeError,"current archived build"):after.export(self.row)
    def test_missing_command_source_closure_is_rejected(self):
        for missing in [None,[]]:
            with self.subTest(closure=missing):
                entry=dict(command=["python","driver.py"])
                if missing is not None:entry["source_files"]=missing
                with self.assertRaisesRegex(RuntimeError,"requires its reviewed source closure"):
                    m.adapter_source_inventory(self.root,{"basic":entry})
    def test_missing_transitive_file_is_rejected_before_runner_can_reuse(self):
        before=self.runner();(self.root/"adapters/helper.py").unlink()
        with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):
            before.ensure_frozen()
        with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):self.runner()
    def test_parent_escape_is_rejected_even_when_file_exists(self):
        with tempfile.TemporaryDirectory(prefix="outside-closure-",dir=HERE) as directory:
            outside=Path(directory)/"helper.py";outside.write_text("real file outside root",encoding="utf-8")
            relative=os.path.relpath(outside,self.root)
            self.adapters["basic"]["source_files"].append(relative)
            with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):
                m.adapter_source_inventory(self.root,self.adapters)
    def linked_inventory(self,directory=False):
        target=self.root/"adapters"/("nested" if directory else "helper.py")
        if directory:target.mkdir();(target/"helper.py").write_text("linked helper",encoding="utf-8")
        link=self.root/("linked-dir" if directory else "linked.py")
        try:link.symlink_to(target,target_is_directory=directory)
        except OSError:
            # Windows may deny unprivileged link creation. Exercise the exact
            # filesystem predicate with a real existing path in that case.
            linked=target if directory else target
            original=Path.is_symlink
            with patch.object(Path,"is_symlink",lambda path: path==linked or original(path)):
                name=(target/"helper.py" if directory else target).relative_to(self.root).as_posix()
                with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):
                    m.adapter_source_inventory(self.root,{"basic":dict(command=["unused"],source_files=[name])})
        else:
            try:
                name=(link/"helper.py" if directory else link).relative_to(self.root).as_posix()
                with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):
                    m.adapter_source_inventory(self.root,{"basic":dict(command=["unused"],source_files=[name])})
            finally:link.unlink()
    def test_direct_symlink_inside_root_is_rejected(self):self.linked_inventory()
    def test_symlink_directory_ancestor_inside_root_is_rejected(self):self.linked_inventory(True)
    def test_junction_directory_ancestor_is_rejected(self):
        # Portable simulation of the Windows junction filesystem predicate;
        # source remains a real file and contained() still returns true.
        target=self.root/"adapters";original=getattr(Path,"is_junction",lambda path:False)
        with patch.object(Path,"is_junction",lambda path:path==target or original(path),create=True):
            with self.assertRaisesRegex(RuntimeError,"Missing/escaping adapter source"):
                m.adapter_source_inventory(self.root,self.adapters)
    def test_shared_transitive_files_are_deduplicated_without_losing_hashes(self):
        self.adapters["sample"]=dict(self.adapters["basic"])
        inventory=m.adapter_source_inventory(self.root,self.adapters)
        self.assertEqual(set(inventory),set(self.adapters["basic"]["source_files"]))
        for name,value in inventory.items():self.assertEqual(value,m.digest(self.root/name))
    def test_real_six_template_closures_include_runtime_and_transitive_helpers(self):
        adapters=m.read(ROOT/"knowledge/initiatives/gameplay-helpers/acceptance-adapters.json")
        self.assertEqual(set(adapters),{"basic","sample","third-person","platformer-2d","dungeon","dungeon-complete"})
        common={"tests/integration/gameplay_helpers/adapter.py","tests/integration/gameplay_helpers/layout.py",
            "tests/integration/gameplay_helpers/observe.lua","tools/epok_documents.py",
            "tools/profile_runtime_memory.py","tools/runtime_memory_probe.lua"}
        for template,entry in adapters.items():
            with self.subTest(template=template):
                self.assertTrue(common<=set(entry["source_files"]))
                inventory=m.adapter_source_inventory(ROOT,{template:entry})
                self.assertEqual(set(inventory),set(entry["source_files"]))
        for template in ["basic","sample"]:
            self.assertTrue({"tests/integration/gameplay_helpers/authored_edits.py",
                "tests/integration/gameplay_helpers/angular_observer.lua"}<=set(adapters[template]["source_files"]))
        for template in ["third-person","platformer-2d"]:
            self.assertTrue({"tests/integration/gameplay_helpers/player_routes.py",
                "tests/integration/gameplay_helpers/player_observe.lua",
                "tests/integration/gameplay_helpers/physical_support.py",
                "tests/integration/gameplay_helpers/physical_routes.lua",
                "tests/integration/gameplay_helpers/capture_reuse.py"}<=set(adapters[template]["source_files"]))
        self.assertTrue({"tests/integration/dungeon_complete_capture.py","tools/package_dungeon_complete.py",
            "tests/integration/verify_dungeon_complete_economy.py",
            "tests/integration/gameplay_helpers/complete_observation.py",
            "tests/integration/gameplay_helpers/complete_observation.lua",
            "tools/profile_runtime.py"}<=set(adapters["dungeon-complete"]["source_files"]))


if __name__=="__main__":unittest.main(verbosity=2)
