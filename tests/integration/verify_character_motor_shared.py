"""Exercise the shared motor API through real C++, Blueprint and Lua calls."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
import uuid

from verify_character_motor import ROOT, EDITOR, identity, write, documents
from gameplay_probe import run_modes, export_probe


def verify(out, modes, prepare_only, export):
    out.mkdir(parents=True, exist_ok=True)
    project = out / 'project'
    magic = 0x45000000 | (uuid.uuid4().int & 0xffffff)
    counter = 0

    def run(*command, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, command)], cwd=ROOT,
                                capture_output=True, text=True, encoding='utf-8', timeout=900)
        (out / (label or f'command-{counter}.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if not project.exists():
        run('--create-project', project, '--template', 'basic', '--name', 'Shared Motor Providers')
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t shared_motor_probe[24]={};}
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="BASE") SharedMotorProbe:public epok::CharacterMotor3DComponent {
 uint32_t frame_=0,failures_=0,jumps_=0,grounded_=0,zero_=0,invalid_=0,hash_=2166136261u;
 void mix(uint32_t value){hash_=(hash_^value)*16777619u;}
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpatialQueryContextComponent") epok::ObjectId query_context{};
 EPOK_FUNCTION(BlueprintEvent) virtual epok::CharacterMotionSample3D run_command(epok::CharacterCommand3D command,epok::Fixed dt,epok::ObjectId context,uint32_t revision){(void)command;(void)dt;(void)context;(void)revision;return {};}
 EPOK_FUNCTION(BlueprintEvent) virtual bool run_facing(epok::Fixed degrees){(void)degrees;return false;}
 EPOK_FUNCTION(BlueprintEvent) virtual void run_clear(){}
 void begin_play()override{epok::CharacterMotor3DComponent::begin_play();automatic_tick=false;}
 void on_frame(uint32_t)override{
  if(frame_>=120||probe_slot>=3)return;auto* data=get_owner()?get_owner()->data():nullptr;if(!data)return;
  if(!run_facing(frame_<95?epok::Fixed(90.0):epok::Fixed(45.0)))failures_|=1;
  if(frame_==30&&run_facing(epok::Fixed(400000)))failures_|=2;
  if(frame_==90||frame_==110)run_clear();
  if(frame_==105)if(auto* context=epok::active_object_registry->resolve<epok::SpatialQueryContextComponent>(query_context))context->reset_queries();
  epok::CharacterCommand3D command;command.velocity.x=frame_<75?epok::Fixed(6.0):epok::Fixed(0.0);command.jump=frame_==15||frame_==100;
  const auto before=data->transform;const auto expected=frame_==95?epok::CharacterMotionStatus::NoStep:frame_==100?epok::CharacterMotionStatus::InvalidContext:epok::CharacterMotionStatus::Applied;
  const auto sample=run_command(command,frame_==95?epok::Fixed(0.0):epok::Fixed(1.0/60.0),frame_==100?epok::ObjectId{}:query_context,frame_>=115?0:1);
  if(sample.status!=expected)failures_|=4;
  if(sample.speed.raw())failures_|=8;
  const epok::Fixed delta[]={sample.displacement.x,sample.displacement.y,sample.displacement.z};
  for(unsigned i=0;i<3;++i)if(delta[i]!=data->transform.position[i]-before.position[i])failures_|=16;
  if(expected!=epok::CharacterMotionStatus::Applied&&data->transform.rotation[1]!=before.rotation[1])failures_|=32;
  if(frame_==101&&!sample.jumped)failures_|=64;
  jumps_+=sample.jumped;grounded_+=sample.grounded;zero_+=sample.status==epok::CharacterMotionStatus::NoStep;invalid_+=sample.status==epok::CharacterMotionStatus::InvalidContext;
  const uint32_t words[]={uint32_t(sample.status),uint32_t(sample.velocity.x.raw()),uint32_t(sample.velocity.y.raw()),uint32_t(sample.velocity.z.raw()),uint32_t(sample.displacement.x.raw()),uint32_t(sample.displacement.y.raw()),uint32_t(sample.displacement.z.raw()),uint32_t(sample.facing.raw()),sample.grounded,sample.jumped,sample.blocked,sample.unresolved_overlap};
  for(auto value:words)mix(value);++frame_;
  if(frame_==120){auto* p=shared_motor_probe+probe_slot*8;p[1]=frame_;p[2]=failures_;p[3]=jumps_;p[4]=grounded_;p[5]=hash_;p[6]=zero_;p[7]=invalid_;p[0]=MAGIC+EPOK_LUA_MODE;}
 }
};
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="CPP") CppSharedMotorProbe:public SharedMotorProbe {
public:
 epok::CharacterMotionSample3D run_command(epok::CharacterCommand3D command,epok::Fixed dt,epok::ObjectId context,uint32_t revision)override{submit(command);return simulate_shared(dt,context,revision);}
 bool run_facing(epok::Fixed degrees)override{return set_facing(degrees);}
 void run_clear()override{clear_velocity();}
};
'''.replace('"BASE"', '"' + identity('shared-base') + '"').replace('"CPP"', '"' + identity('shared-cpp') + '"').replace('MAGIC+', str(magic) + 'u+')
    write(project / 'assets/scripts/SharedMotorProbe.hpp', source)
    classes = {c['cpp_name']: c for c in json.loads(run('--project', project, '--reflect'))['classes']}
    base = classes['SharedMotorProbe']
    methods = {f['name']: f for f in classes['epok::CharacterMotor3DComponent']['functions']}
    events = {f['name']: f for f in base['functions']}
    path = project / 'assets/Blueprints/BPSharedMotorProbe.epokbp'
    if not path.exists():
        run('--project', project, '--new-blueprint', 'BPSharedMotorProbe', '--parent', base['id'])
    asset = documents.loads(path.read_text(encoding='utf-8'))
    functions = []
    for event_name, calls, returns in (
        ('run_command', [('submit', {'command': 'command'}), ('simulate_shared', {'delta_seconds': 'dt', 'query_context': 'context', 'support_revision': 'revision'})], True),
        ('run_facing', [('set_facing', {'degrees': 'degrees'})], True),
        ('run_clear', [('clear_velocity', {})], False),
    ):
        event = events[event_name]
        entry = {'id': str(uuid.uuid4()), 'kind': {'kind': 'entry'}, 'inputs': {}, 'outputs': {}}
        nodes = [entry]
        previous = entry
        for name, parameters in calls:
            call = {'id': str(uuid.uuid4()), 'kind': {'kind': 'call', 'function': methods[name]['id']},
                    'inputs': {key: {'kind': 'parameter', 'name': value} for key, value in parameters.items()}, 'outputs': {}}
            previous['outputs']['next'] = [call['id']]
            nodes.append(call);previous = call
        finish = {'id': str(uuid.uuid4()), 'kind': {'kind': 'return'}, 'inputs': {}, 'outputs': {}}
        if returns:
            finish['inputs']['value'] = {'kind': 'link', 'node': previous['id'], 'pin': 'value'}
        previous['outputs']['next'] = [finish['id']]
        nodes.append(finish)
        functions.append({'id': str(uuid.uuid4()), 'name': event_name, 'override_id': event['id'],
                          'parameters': event['parameters'], 'returns': event['returns'], 'entry': entry['id'], 'nodes': nodes})
    asset['functions'] = functions
    write(path, asset)
    write(project / 'assets/scripts/LuaSharedMotorProbe.lua', '''---@class LuaSharedMotorProbe : SharedMotorProbe
local LuaSharedMotorProbe = SharedMotorProbe:extend()
---@override
function LuaSharedMotorProbe:run_command(command, dt, context, revision)
 self:submit(command)
 return self:simulate_shared(dt, context, revision)
end
---@override
function LuaSharedMotorProbe:run_facing(degrees)
 return self:set_facing(degrees)
end
---@override
function LuaSharedMotorProbe:run_clear()
 self:clear_velocity()
end
return LuaSharedMotorProbe
''')
    write(project / 'ProjectSettings/LuaClasses.epoksettings', {'classes': {'assets/scripts/LuaSharedMotorProbe.lua': identity('shared-lua')}})
    manifest = documents.loads(next(project.glob('*.epokproject')).read_text(encoding='utf-8'))
    scene_path = project / manifest['startup_scene']
    scene = documents.loads(scene_path.read_text(encoding='utf-8'))
    camera = copy.deepcopy(scene['actors'][0])
    scene['actors'] = [camera]

    def component(name, props):
        return {'id': str(uuid.uuid4()), 'name': name, 'class': {'name': name, 'class_id': classes[name]['id']},
                'properties': props, 'overrides': list(props)}

    def body(name, position, half):
        result = copy.deepcopy(camera)
        result.update(id=str(uuid.uuid4()), name=name, properties={}, overrides=[])
        result['components'] = [c for c in result['components'] if c.get('root')]
        result['components'][0]['id'] = str(uuid.uuid4())
        result['components'][0]['properties'].update(position=position, rotation=[0, 0, 0])
        result['components'].append(component('epok::Collider3DComponent', {'collider': {
            'enabled': True, 'trigger': False, 'center': [0, 0, 0], 'half_extents': half,
            'layer': 1, 'mask': 4294967295}}))
        return result

    floor = body('Floor', [0, -.25, 0], [100, .25, 100])
    context = component('epok::SpatialQueryContextComponent', {'automatic_reset': False})
    floor['components'].append(context)
    scene['actors'].append(floor)
    for slot, (name, class_id) in enumerate((('CppSharedMotorProbe', identity('shared-cpp')),
                                           ('BPSharedMotorProbe', asset['id']), ('LuaSharedMotorProbe', identity('shared-lua')))):
        actor = body('Same display name', [slot * 10, 1, 0], [.4, 1, .4])
        actor['components'].append({'id': str(uuid.uuid4()), 'name': name,
            'class': {'name': name, 'class_id': class_id},
            'properties': {'probe_slot': slot, 'query_context': context['id'], 'measure_speed': False,
                           'automatic_tick': False, 'step_height': 0},
            'overrides': ['probe_slot', 'query_context', 'measure_speed', 'automatic_tick', 'step_height']})
        scene['actors'].append(actor)
    write(scene_path, scene)
    run('--project', project, '--compile-blueprints')
    if prepare_only:
        print('PASS shared motor calls prepared for all three providers', flush=True)
        return

    def check(values, expected):
        assert values[:4] == [expected, 120, 0, 2], values
        assert values[4] > 10 and values[6:] == [1, 1], values

    run_modes(out, project, 'shared-motor', modes, 'shared_motor_probe', magic, check)
    if export:
        export_probe(out, project, run, ('character3d.hpp', 'gameplay_api.hpp'))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, default=ROOT / 'artifacts/gameplay-helpers/p8/motor-shared-providers')
    parser.add_argument('--mode', action='append', choices=('native_cpp', 'vm_bytecode', 'vm_source'))
    parser.add_argument('--prepare-only', action='store_true')
    parser.add_argument('--export', action='store_true')
    args = parser.parse_args()
    verify(args.out.resolve(), args.mode, args.prepare_only, args.export)
