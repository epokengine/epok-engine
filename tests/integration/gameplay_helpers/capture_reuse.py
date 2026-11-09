"""Reconcile unaffected P7 capture mechanisms without rerunning accepted tests.

This does not claim a capture binding exists in a default starter. It reuses the
actual selected-provider P7 token/gravity scenario only if its relevant SDK and
authored-controller closure is exactly unchanged. Any difference stays missing.
"""
from pathlib import Path
import hashlib
import json
import re

sha=lambda data:hashlib.sha256(data).hexdigest()


def block(source,marker):
    # Preserve offsets while masking comments and string/character literals.
    expression=r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\])*"|\x27(?:\\.|[^\x27\\])*\x27'
    masked=re.sub(expression,lambda m:''.join('\n' if c=='\n' else ' ' for c in m[0]),source,flags=re.S)
    start=source.index(marker);at=masked.index('{',start);depth=1;end=at+1
    while depth:
        if end>=len(masked):raise ValueError('Unbalanced dependency definition')
        depth+=(masked[end]=='{')-(masked[end]=='}');end+=1
    return source[start:end]


def class_block(source,name):
    match=re.search(r'^class EPOK_CLASS\([^\n]*\) '+re.escape(name)+r'\s*(?::[^\n{]+)?\{',source,re.M)
    if not match:raise ValueError('Missing relevant SDK class '+name)
    return block(source,match[0])


def reconcile(root,binding):
    folder='player-routes' if binding['template']=='third-person' else 'platformer-routes'
    reference=root/'artifacts/gameplay-helpers/p7'/folder/'foundation'/binding['flavor']
    result=reference/'result.json';build=reference/'project/.epok/build'
    current_project=Path(binding['project']);current_build=current_project/'.epok/build'
    proof={'status':'missing','scope':'Unaffected input-capture/token-lifetime mechanism; not current starter capture binding, CPU baseline or whole-scene acceptance',
           'reference_result':str(result),'current_dependency_source':str(current_build),'source_identity':{},'evidence_files':{},'missing':[],'current_template_capture_executed':False}
    if not result.is_file():proof['missing'].append('Accepted selected-provider P7 capture receipt unavailable');return proof
    rows=json.loads(result.read_text(encoding='utf-8'));native=[r for r in rows if r.get('mode')=='native_cpp']
    if len(native)!=1 or len(native[0].get('sample',[]))!=8 or native[0]['sample'][1:3]!=[240,0] or native[0]['sample'][6]<=0:
        proof['missing'].append('Accepted P7 capture/fall witness missing or failed');return proof
    proof.update(reference_result_sha256=sha(result.read_bytes()),reference_editor_sha256=native[0]['editor_sha256'],
                 reference_executable_sha256=native[0]['exe_sha256'],captured_fall_steps=native[0]['sample'][6])
    def evidence(path):proof['evidence_files'][str(path.resolve())]=sha(path.read_bytes())
    evidence(result)
    def compare(name,old,current):
        same=old==current;proof['source_identity'][name]={'previous':sha(old),'current':sha(current),'identical':same}
    try:
        for name in ['input.hpp','gameplay_control.hpp','character_input.hpp']:
            compare(name,(build/name).read_bytes(),(current_build/name).read_bytes())
            evidence(build/name);evidence(current_build/name)
        evidence(build/'object_model.hpp');evidence(current_build/'object_model.hpp')
        old=(build/'object_model.hpp').read_text(encoding='utf-8');now=(current_build/'object_model.hpp').read_text(encoding='utf-8')
        for name in ['Object','ActorComponent']:
            compare('object_model::'+name,class_block(old,name).encode(),class_block(now,name).encode())
        for marker in ['struct ObjectId {','struct ObjectSlot {','struct ClassDescriptor {','struct ObjectRegistry {',
                       'inline const ClassDescriptor* find_object_class(','inline bool object_class_is_a(','bool actor_active(','bool pending_destruction() const',
                       'ObjectId level_id() const','bool active() const','inline bool is_active(Object* value)']:
            compare('object_model::'+marker,block(old,marker).encode(),block(now,marker).encode())
        current=current_project;controller='ThirdPersonController' if binding['template']=='third-person' else 'PlatformerController'
        if binding['flavor']=='blueprint':names=['assets/Blueprints/'+controller+'.epokbp']
        elif binding['flavor']=='lua':names=['assets/scripts/'+controller+'.lua']
        else:names=['assets/scripts/'+controller+'.hpp','assets/scripts/'+controller+'.cpp']
        for name in names:
            compare('selected-controller::'+name,(reference/'project'/name).read_bytes(),(current/name).read_bytes())
            evidence(reference/'project'/name);evidence(current/name)
        proof['current_executable_sha256']=sha((current_build/'epok.ps-exe').read_bytes())
    except (OSError,ValueError) as error:proof['missing'].append(str(error));return proof
    changed=[name for name,value in proof['source_identity'].items() if not value['identical']]
    if changed:proof['missing'].append('Relevant capture dependency changed: '+', '.join(changed))
    else:proof['status']='reused-unaffected-capture-mechanism'
    # Whole object_model.hpp/CharacterMotor3D may include unrelated newer work;
    # only this explicit closure is reused. Current actual physical routes still
    # execute their motor/camera/level behavior with the current linked image.
    return proof
