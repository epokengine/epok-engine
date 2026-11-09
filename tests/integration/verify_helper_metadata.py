"""Execute inherited helper callbacks/defaults with a metadata-only ancestor."""
from __future__ import annotations
import argparse
import copy
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write, NAMESPACE
from gameplay_probe import run_modes, export_probe


def identity(name):
    return str(uuid.uuid5(NAMESPACE, 'helper-metadata:' + name))


def verify(out, modes, export):
    out.mkdir(parents=True, exist_ok=True)
    project = out / 'project'
    assert not project.exists(), 'Use fresh evidence output'
    sequence = 0

    def run(*args, label=None):
        nonlocal sequence
        sequence += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding='utf-8', timeout=1200)
        (out / (label or f'command-{sequence}.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    run('--create-project', project, '--template', 'basic')
    magic = 0x4D000000 | (uuid.uuid4().int & 0xffffff)
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t helper_metadata_probe[24]={};}
inline bool helper_metadata_requested=false;
inline uint32_t helper_metadata_requested_at=0;
class EPOK_CLASS(Abstract,Blueprintable,Id="BASE") MetadataCameraProbe:public epok::ThirdPersonCamera {
 uint32_t updates_=0;
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintEvent) virtual bool accept_probe(){return false;}
 epok::CameraRigSample calculate_view(epok::Fixed)override{
  ++updates_;epok::CameraRigSample result;result.status=epok::CameraRigStatus::Applied;
  result.position={2.0,3.0,4.0};result.yaw=20.0;result.pitch=30.0;result.distance=4.0;return result;
 }
 void on_frame(uint32_t)override{
  if(probe_slot>=3||updates_<30)return;auto* p=helper_metadata_probe+probe_slot*8;if(p[0])return;
  auto* registry=epok::active_object_registry;const auto* body=data();uint32_t errors=0;
  if(!registry||!body)return;
  // Spawns requested in callbacks are deferred. Observe the actual actor after
  // the next simulation step rather than expecting an immediate return handle.
  constexpr auto native_id=epok::detail::compact_class_id("1c06764a-ac44-5a70-a9cf-ed827a4cf9ba");
  auto* level=registry->resolve<epok::Level>(level_id());
  if(!helper_metadata_requested){
   if(probe_slot==0){
    auto* selected=epok::find_object_class(native_id);
    if(selected&&level)level->spawn_actor(*selected,nullptr);
    helper_metadata_requested=true;helper_metadata_requested_at=updates_;
   }
   return;
  }
  if(updates_<=helper_metadata_requested_at)return;
  bool spawned=false;
  if(level)for(size_t i=0;i<level->actor_count();++i){
   auto* actor=registry->resolve<epok::Actor3D>(level->actor_at(i));
   if(actor&&actor->class_id()==native_id&&actor->component_count()==2)spawned=true;
  }
  if(!spawned)errors|=64;
  if(registry->resolve<epok::Actor3D>(id())!=this)errors|=1;
  if(body->transform.position[0]!=2.0||body->transform.position[1]!=3.0||body->transform.position[2]!=4.0
      ||body->transform.rotation[0]!=30.0||body->transform.rotation[1]!=20.0)errors|=2;
  if(component_count()!=3||!root.id().valid()||!camera.id().valid()||!boom.id().valid()
      ||registry->get(root.id())!=&root||registry->get(camera.id())!=&camera||registry->get(boom.id())!=&boom)errors|=4;
  // Walk class metadata at run time, without naming the native ancestor and
  // thereby explicitly asking the cook for its creation surface.
  auto* type=epok::find_object_class(class_id());uint32_t ancestors=0,metadata=0;
  while(type&&type->parent&&ancestors<12){
   type=epok::find_object_class(type->parent);if(!type){errors|=8;break;}++ancestors;
   if((type->flags&epok::ObjectClassSpawnable)&&!(type->flags&epok::ObjectClassAbstract)&&!type->acquire)++metadata;
  }
  if(!metadata)errors|=16;
  if(!accept_probe())errors|=32;
  p[1]=updates_;p[2]=errors;p[3]=component_count();p[4]=1;p[5]=1;p[6]=metadata;p[7]=1;
  p[0]=MAGIC+EPOK_LUA_MODE;
 }
};
class EPOK_CLASS(Blueprintable,Id="CPP") CppMetadataCameraProbe:public MetadataCameraProbe {
public:bool accept_probe()override{return true;}
};
'''.replace('BASE', identity('base')).replace('CPP', identity('cpp')).replace('MAGIC', str(magic) + 'u')
    write(project / 'assets/scripts/MetadataCameraProbe.hpp', source)
    metadata = json.loads(run('--project', project, '--reflect'))
    base = next(c for c in metadata['classes'] if c['cpp_name'] == 'MetadataCameraProbe')
    event = next(f for f in base['functions'] if f['name'] == 'accept_probe')
    run('--project', project, '--new-blueprint', 'BP_MetadataCameraProbe', '--parent', identity('base'))
    bp_path = project / 'assets/Blueprints/BP_MetadataCameraProbe.epokbp'
    asset = documents.loads(bp_path.read_text(encoding='utf-8'))
    entry, done, function = (str(uuid.uuid4()) for _ in range(3))
    asset['functions'] = [{'id': function, 'name': 'accept_probe', 'override_id': event['id'],
        'parameters': event['parameters'], 'returns': event['returns'], 'entry': entry,
        'nodes': [{'id': entry, 'kind': {'kind': 'entry'}, 'inputs': {}, 'outputs': {'next': [done]}},
                  {'id': done, 'kind': {'kind': 'return'},
                   'inputs': {'value': {'kind': 'literal', 'value_type': {'kind': 'bool'}, 'value': True}}, 'outputs': {}}]}]
    write(bp_path, asset)
    write(project / 'assets/scripts/LuaMetadataCameraProbe.lua', '''---@class LuaMetadataCameraProbe : MetadataCameraProbe
local LuaMetadataCameraProbe = MetadataCameraProbe:extend()
---@override
function LuaMetadataCameraProbe:accept_probe()
 return true
end
return LuaMetadataCameraProbe
''')
    write(project / 'ProjectSettings/LuaClasses.epoksettings', {'classes': {
        'assets/scripts/LuaMetadataCameraProbe.lua': identity('lua')}})
    manifest = documents.loads(next(project.glob('*.epokproject')).read_text(encoding='utf-8'))
    path = project / manifest['startup_scene']
    scene = documents.loads(path.read_text(encoding='utf-8'))
    root = copy.deepcopy(next(c for c in scene['actors'][0]['components'] if c.get('root')))
    scene['actors'] = []
    for slot, (name, class_id) in enumerate((('CppMetadataCameraProbe', identity('cpp')),
                                           ('BP_MetadataCameraProbe', asset['id']),
                                           ('LuaMetadataCameraProbe', identity('lua')))):
        transform = copy.deepcopy(root)
        transform['id'] = str(uuid.uuid4())
        transform['properties'].update(position=[0, 0, 0], rotation=[0, 0, 0])
        scene['actors'].append({'id': str(uuid.uuid4()), 'name': name,
            'class': {'name': name, 'class_id': class_id}, 'properties': {'probe_slot': slot},
            'overrides': ['probe_slot'], 'components': [transform]})
    write(path, scene)
    run('--project', project, '--compile-blueprints')

    def check(values, expected):
        assert values[0] == expected and values[1] >= 30 and values[2:6] == [0, 3, 1, 1] and values[6] >= 1 and values[7] == 1, values

    run_modes(out, project, 'inherited-metadata', modes, 'helper_metadata_probe', magic, check)
    generated = (project / '.epok/build/scene.hh').read_text(encoding='utf-8')
    assert 'SharedObjectPool<epok::CameraRig3D' not in generated
    assert 'SharedObjectPool<epok::ThirdPersonCamera' in generated
    if export:
        export_probe(out, project, run, ('camera_rig.hpp', 'object_model.hpp'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--mode', action='append', choices=('native_cpp', 'vm_bytecode', 'vm_source'))
    parser.add_argument('--export', action='store_true')
    args = parser.parse_args()
    verify(args.out.resolve(), args.mode, args.export)
