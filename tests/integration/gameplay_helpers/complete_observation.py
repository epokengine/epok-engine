"""Exact-program Complete observation. No target instrumentation or policy writes.

Host contract tests exercise rejection paths; they never constitute PSX proof.
"""
from pathlib import Path
import hashlib
import json
import re
import yaml
import layout

LEAVES = ('run_flow', 'player_vitals', 'player_combat', 'encounter_policy',
          'economy_policy', 'game_hud', 'terminal_widget', 'shop_widget',
          'inventory_widget', 'economy_station')
TELEMETRY = {
    'fps': ('dungeon_fps_telemetry', 'DungeonFpsTelemetry',
            ('frames','slow','max_us','explore_max','combat_max','interface_max','startup_max')),
    'flow': ('dungeon_flow_telemetry', 'DungeonFlowTelemetry',
             ('phase','elapsed','fade','music_volume','music_playing','inputs_locked','starts','restarts')),
    'game': ('dungeon_telemetry', 'DungeonTelemetry',
             ('ticks','state','health','key','kills','boss_health','enemy_shots','damage','heals','doors_opened','restarts')),
    'combat': ('combat_telemetry', 'CombatTelemetry',
               ('ticks','spawned','active','shots','hits','immune','walls','recovered','muzzle_errors','muzzle','states')),
    'audio': ('dungeon_audio_telemetry', 'DungeonAudioTelemetry', ('events','missing')),
    'economy': ('economy_telemetry', 'EconomyTelemetry', ('menu','selection','slot','ore','credits','medkits','tanks','buster','transfers','installed','error','saves','loaded','operation')),
}

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def one(values, reason):
    if len(values) != 1:
        raise ValueError('Missing/ambiguous ' + reason)
    return values[0]

def startup_scene_scope(header):
    """Follow the compiled startup loader, excluding reusable actor prototypes.

    The generated header reuses actor_records/actor_components_N in each modal
    prototype namespace. A label alone cannot identify a resident scene slot.
    Mask literals/comments before balancing C++ braces so authored text such as
    a '}' in a label cannot end a namespace or loader body prematurely.
    """
    tokens=re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|//[^\n]*|/\*[\s\S]*?\*/')
    masked=tokens.sub(lambda match:' '*len(match[0]),header)
    def body(match):
        start=match.end()-1;depth=1;end=start+1
        while end<len(masked) and depth:
            if masked[end]=='{':depth+=1
            elif masked[end]=='}':depth-=1
            end+=1
        if depth:raise ValueError('Unbalanced compiled scene scope')
        return header[start+1:end-1]
    initializers=[body(match) for match in re.finditer(r'\binline\s+void\s+initialize_scripts\s*\(\s*\)\s*\{',masked)]
    # Prototype initializers only initialize_components(); the resident startup
    # initializer is the one that actually dispatches a compiled load_bank_N.
    initializer=one([value for value in initializers if re.search(r'\bload_bank_\d+\s*\(',value)], 'compiled startup initializer')
    bank=one(re.findall(r'\bload_bank_(\d+)\s*\(\s*\)\s*;',initializer), 'compiled startup bank')
    loader=one(list(re.finditer(r'\binline\s+void\s+load_bank_'+bank+r'\s*\(\s*\)\s*\{',masked)), 'compiled startup loader')
    scene=one(re.findall(r'\bload_actor_bank\s*\(\s*scene_(\d+)::actor_table\s*,',body(loader)), 'compiled startup actor table')
    scopes=[body(match) for match in re.finditer(r'\bnamespace\s+epok::scene_'+scene+r'\s*\{',masked)]
    return one([scope for scope in scopes if re.search(r'\bActorRecord\s+actor_records\[\]\s*=',scope)], 'compiled startup actor scope')

def header_slot(header, name):
    # Resolve UUID-bound source labels only within the actual startup scene.
    scope=startup_scene_scope(header)
    literal = json.dumps(name, ensure_ascii=False)
    table=one(re.findall(r'\bActorRecord\s+actor_records\[\]\s*=\s*\{(.*?)\};',scope,re.S), 'compiled startup actor records')
    records = re.findall(re.escape(literal) + r',(?:true|false),-?\d+,-?\d+,-?\d+,actor_components_(\d+),', table)
    record = one(records, 'compiled actor label ' + name)
    components=re.findall(r'\bActorComponentRecord\s+actor_components_'+record+r'\[\]\s*=\s*\{\{UINT64_C\(\d+\),"(?:[^"\\]|\\.)*",(?:true|false),-?\d+,(-?\d+),',scope)
    component=one(components,'compiled resident root component '+name)
    if int(component) < 0:
        raise ValueError('Bound actor has no resident ActorData: ' + name)
    return int(component)

def scene_contract(project, header):
    project = Path(project).resolve(strict=True)
    installation = yaml.safe_load((project/'ProjectSettings/Template.epoksettings').read_text(encoding='utf-8'))
    manifests = list(project.glob('*.epokproject'))
    manifest = yaml.safe_load(one(manifests, 'actual project manifest').read_text(encoding='utf-8'))
    if installation['template'] != 'dungeon-complete' or manifest['lua_execution'] != 'native_cpp':
        raise ValueError('Only the actual Complete native/AOT installation is supported')
    scene_file = (project/manifest['startup_scene']).resolve(strict=True)
    if not scene_file.is_relative_to(project):
        raise ValueError('Startup scene escapes actual project')
    scene = yaml.safe_load(scene_file.read_text(encoding='utf-8'))
    actors = scene['actors']; roles = installation['roles']
    components = {c['id']:(a,c) for a in actors for c in a.get('components', [])}
    if len(components) != sum(len(a.get('components', [])) for a in actors):
        raise ValueError('Duplicate component UUID')
    def selected(role):
        identity = roles[role]['class_id']
        values = [(a,None) for a in actors if a['class'].get('class_id') == identity]
        values += [(a,c) for a in actors for c in a.get('components', []) if c['class'].get('class_id') == identity]
        # Modal widget classes are deliberately unplaced and spawned by the
        # selected presenter; requiring resident instances changes the template.
        if not values and role in ('terminal_widget','shop_widget','inventory_widget'):
            return (None,None)
        # Shop and terminal intentionally have separate instances of the same
        # selected station class. Their behavior is checked by the economy route;
        # this preflight verifies the selected definition, not one chosen station.
        if role=='economy_station' and values:return values[0]
        return one(values, 'selected role ' + role)
    selected_roles = {role:selected(role) for role in LEAVES}
    definitions={}
    extension={'cpp':'.hpp','blueprint':'.epokbp','lua':'.lua'}[installation['gameplay']]
    for role in LEAVES:
        source=one(list((project/'assets').rglob(roles[role]['name']+extension)), 'chosen-language role source '+role)
        relative=source.relative_to(project).as_posix()
        if source.is_symlink():raise ValueError('Linked selected rule source')
        if installation['gameplay']=='lua':
            registry=yaml.safe_load((project/'ProjectSettings/LuaClasses.epoksettings').read_text(encoding='utf-8'))
            if registry.get('classes',{}).get(relative)!=roles[role]['class_id']:
                raise ValueError('Selected Lua class identity mismatch: '+role)
        elif roles[role]['class_id'] not in source.read_text(encoding='utf-8'):
            raise ValueError('Selected authored class identity mismatch: '+role)
        definitions[role]={'path':relative,'sha256':sha(source),'placed':selected_roles[role][0] is not None}
    player = selected('player')[0]
    game_owner, game = selected('game_root')
    controller_owner, controller = selected('controller')
    if game is None or controller is None or player['id'] != controller_owner['id']:
        raise ValueError('Current player/controller ownership mismatch')
    gp = game.get('properties', {}); cp = controller.get('properties', {})
    # New composition leaves two free player component slots. Old installations
    # inferred the player from the Game owner only when no explicit link existed.
    game_player = gp['player_actor'] if 'player_actor' in gp else game_owner['id']
    if game_player != player['id']:
        raise ValueError('Game is not bound to the selected current player')
    if cp.get('game_component') != game['id'] or cp.get('combat_policy') != selected_roles['player_combat'][1]['id']:
        raise ValueError('Controller is not bound to selected game/combat rules')
    for property_name, role in [('run_flow','run_flow'),('player_vitals','player_vitals'),
                                ('combat_policy','player_combat'),('encounter_policy','encounter_policy'),('economy_policy','economy_policy')]:
        rule_component = selected_roles[role][1]
        if rule_component is None or gp.get(property_name) != rule_component['id']:
            raise ValueError('Game is not bound to selected ' + role)
    refs = components.get(gp.get('encounter_bindings'))
    if refs is None:
        raise ValueError('Current encounter binding component absent')
    actor_by_id = {a['id']:a for a in actors}
    if len(actor_by_id) != len(actors):
        raise ValueError('Duplicate actor UUID')
    bound = {}
    for role in ['guard0','guard1','guard2','boss','key','pack0','pack1','pack2']:
        identity = refs[1].get('properties', {}).get(role)
        if identity not in actor_by_id:
            raise ValueError('Missing actual encounter actor ' + role)
        bound[role] = actor_by_id[identity]
    bound.update(player=player, hud=selected_roles['game_hud'][0])
    if cp.get('lock_marker') not in actor_by_id:
        raise ValueError('Current target marker binding missing')
    bound['marker'] = actor_by_id[cp['lock_marker']]
    def position(actor):
        transform = one([c for c in actor['components'] if c.get('root') and 'position' in c.get('properties', {})], 'bound actor root transform')
        return [round(v*4096) for v in transform['properties']['position']]
    # Parent affine transforms require another reviewed route, never guessed world coordinates.
    if any(a.get('parent') for a in bound.values()):
        raise ValueError('Parented route actors require reviewed world-space setup')
    if any(sum(a['name'] == actor['name'] for a in actors) != 1 for actor in bound.values()):
        raise ValueError('Current compiled observation label is ambiguous')
    return {'scene_sha256':sha(scene_file), 'installation_sha256':sha(project/'ProjectSettings/Template.epoksettings'),
            'selected_roles':{role:roles[role] for role in LEAVES},
            'selected_rule_definitions':definitions,
            'slots':{role:header_slot(header,a['name']) for role,a in bound.items()},
            'positions_q12':{role:position(a) for role,a in bound.items() if role not in ['hud','marker']},
            'reactor_entry_q12':round(selected_roles['run_flow'][1].get('properties', {}).get('reactor_entry_z',53)*4096)}

def compiled_contract(root, elf, scene, syms=None, structures=None):
    syms = syms if syms is not None else layout.symbols(root, elf)
    structures = structures if structures is not None else layout.dwarf_structures(layout.tool(root,'readelf','--debug-dump=info','--wide',elf))
    def method(prefix):
        return one([v['address'] for k,v in syms.items() if k.startswith(prefix) and '[clone' not in k], 'linked ' + prefix) | 0x80000000
    names = {}; offsets = {}
    for key,(symbol,type_name,fields) in TELEMETRY.items():
        value = layout.require_layout(structures,type_name,fields)
        linked = syms.get(symbol)
        if linked is None or linked['size'] != value['size'] or value['size'] % 4:
            raise ValueError('Linked telemetry/layout mismatch: ' + symbol)
        offsets[key] = {field:value['members'][field]//4 for field in fields}
        if any(value['members'][field]%4 for field in fields):
            raise ValueError('Unsupported telemetry scalar alignment')
        names[key] = (symbol,value['size']//4)
    # Existing route logic uses the reviewed common prefix. Extra diagnostics
    # may grow; a reordered prefix needs a reviewed driver update, not guessing.
    for key in ['fps','flow','game']:
        if list(offsets[key].values()) != list(range(len(offsets[key]))):
            raise ValueError('Changed common telemetry prefix: ' + key)
    if [offsets['combat'][name] for name in TELEMETRY['combat'][2][:9]]!=list(range(9)):
        raise ValueError('Changed common combat telemetry prefix')
    specs = {'ActorData':['transform','image','active','alive','generation'], 'Transform':['position'],
             'Image':['enabled','color','borders'], 'DungeonPlayerController':['camera_mode','target_handle'],
             'DataHandle':['index','generation']}
    layouts = {name:layout.require_layout(structures,name,fields) for name,fields in specs.items()}
    stride = layouts['ActorData']['size']; linked = syms.get('epok::objects', {})
    if not linked.get('size') or linked['size']%stride:
        raise ValueError('Compiled ActorData array size mismatch')
    if any(not 0 <= slot < linked['size']//stride for slot in scene['slots'].values()):
        raise ValueError('Source/header slot outside actual compiled array')
    base = linked['address'] & 0x1fffff
    return {'names':names, 'word_offsets':offsets, 'layouts':layouts,
            'frame_entry':method('DungeonGame::on_frame('),
            'controller_entry':method('DungeonPlayerController::update('),
            'poll_entry':method('epok::ControllerPad::poll('),
            'object_address':base, 'slot_capacity':linked['size']//stride,
            'player_position_address':base+scene['slots']['player']*stride+layouts['ActorData']['members']['transform']+layouts['Transform']['members']['position']}

def observer_lua(contract, scene, symbols, closure=False):
    # The only writes in the observer are player-pose setup and ordinary virtual
    # pad overrides. All policy state, HUD, target handles and counters are read.
    import adapter
    # ABI names use Python tuples. The shared literal writer handles lists but
    # otherwise falls back to JSON, whose ["symbol", count] is invalid Lua.
    # Normalize every nested tuple before handing this data to that writer.
    config = json.loads(json.dumps({'abi':contract,'scene':scene,'symbols':symbols,'closure':closure}))
    source = Path(__file__).with_name('complete_observation.lua').read_text(encoding='utf-8')
    return '\nlocal COMPLETE_CONFIG=' + adapter.lua_literal(config) + '\n' + source

def validate_observation(rows, *, require_closure=False):
    """Only observed transitions may satisfy functional gates; no source-based Green."""
    if not rows or any(b['frame'] <= a['frame'] for a,b in zip(rows,rows[1:])):
        raise ValueError('Missing/nonmonotonic completed-frame observation')
    seen = {name:False for name in ['intro/unlock','lock/freeaim/targetcycle','attack/feedback/heal','death/retry/victory']}
    if any(r['audio_missing'] or r['muzzle_errors'] for r in rows):
        raise ValueError('Missing audio or invalid projectile muzzle')
    phases = {r['phase'] for r in rows}
    seen['intro/unlock'] = set(range(5)) <= phases and any(r['phase']==4 and not r['locked'] and r['music_playing'] for r in rows)
    cycle = any(a['target_valid'] and b['target_valid'] and b['previous_target_valid'] and a['target_index']!=b['target_index'] and a['restarts']==b['restarts'] and b['pad_mask']&2048 and b['camera_mode']==1 for a,b in zip(rows,rows[1:]))
    seen['lock/freeaim/targetcycle'] = cycle and any(r['camera_mode']==2 and r['pad_mask']&1024 and not r['target_valid'] and not r['marker_active'] for r in rows) and any(r['lock_audio']>0 for r in rows)
    damage = any(b['damage']>a['damage'] and b['health']<a['health'] and b['damage_audio']>a['damage_audio'] for a,b in zip(rows,rows[1:]) if b['restarts']==a['restarts'])
    feedback = any(r['damage']>0 and r['image_enabled'] and min(r['borders'])>0 and r['image_color'][0]>r['image_color'][1] for r in rows)
    heal = any(b['heals']>a['heals'] and b['health']>a['health'] and b['heal_audio']>a['heal_audio'] and b['pack_inactive'] for a,b in zip(rows,rows[1:]) if b['restarts']==a['restarts'])
    attack = any(b['hits']>a['hits'] and b['shots']>=a['shots'] and b['shots']>0 and b['hit_audio']>a['hit_audio'] for a,b in zip(rows,rows[1:]) if b['restarts']==a['restarts'])
    seen['attack/feedback/heal'] = damage and feedback and heal and attack
    death = next((r for r in rows if r['state']==3 and r['health']==0 and r['locked']),None)
    after_death = [r for r in rows if death and r['frame']>=death['frame']]
    restarted = next((r for r in after_death if r['restarts']>death['restarts'] and r['phase']==4 and r['health']>0 and r['key']==0 and r['kills']==0),None)
    victory = next((r for r in rows if restarted and r['frame']>restarted['frame'] and r['state']==2 and r['boss_health']==0 and r['boss_destroyed_audio']>0),None)
    victory_retry = any(victory and r['frame']>victory['frame'] and r['restarts']>victory['restarts'] and r['phase']==4 and r['state']==0 and not r['key'] and not r['kills'] and r['health']>0 for r in rows)
    faded = any(r['phase']==6 and r['fade']>0 for r in after_death)
    stopped = any(r['phase']==7 and r['fade']==255 and r['music_volume']==0 and not r['music_playing'] for r in after_death)
    seen['death/retry/victory'] = bool(death and {5,6,7}<={r['phase'] for r in after_death} and faded and stopped and restarted and victory and victory_retry)
    if require_closure and not all(seen.values()):
        raise ValueError('Mandatory actual Complete transitions missing: ' + ', '.join(k for k,v in seen.items() if not v))
    return seen

def validate_rule_edit(control, variant, role, baseline, edited):
    """Bind externally executed edited-policy proof to the same current project.

    Inputs are source inventories, exact built image hashes and console rows;
    merely listing an editable source never satisfies this gate.
    """
    if role not in ('encounter_policy','shop_widget','run_flow'):
        raise ValueError('Unreviewed rule edit role')
    for value in [control,variant]:
        if value.get('mode')!='native_cpp' or value.get('flavor') not in ('cpp','blueprint','lua'):
            raise ValueError('Edited-rule proof requires chosen native/AOT authoring')
        if not re.fullmatch('[0-9a-f]{64}',value.get('image_sha256','')):
            raise ValueError('Edited-rule proof lacks exact executed image')
        if value.get('execution_accepted') is not True or value.get('source_at_execution')!=value.get('source_inventory'):
            raise ValueError('Edited-rule proof is not bound to executed source')
    if control['flavor']!=variant['flavor'] or control['image_sha256']==variant['image_sha256']:
        raise ValueError('Rule edit did not change the selected compiled image')
    path = control['selected_rule_files'][role]
    if variant['selected_rule_files']!=control['selected_rule_files']:
        raise ValueError('Selected rule identity changed')
    changes = {p for p in control['source_inventory'].keys()|variant['source_inventory'].keys() if control['source_inventory'].get(p)!=variant['source_inventory'].get(p)}
    if changes!={path} or path.startswith('.epok/') or not path.endswith({'cpp':'.hpp','blueprint':'.epokbp','lua':'.lua'}[control['flavor']]):
        raise ValueError('Edit must affect only the chosen-language rule source')
    if not baseline or not edited or [r['step'] for r in baseline]!=[r['step'] for r in edited]:
        raise ValueError('Rule edit requires matched actual simulation steps')
    if control.get('route_settings_sha256')!=variant.get('route_settings_sha256') or not re.fullmatch('[0-9a-f]{64}',control.get('route_settings_sha256','')):
        raise ValueError('Rule edit requires the same captured route/input settings')
    field = {'encounter_policy':'health','shop_widget':'selected','run_flow':'phase'}[role]
    if [r[field] for r in baseline]==[r[field] for r in edited]:
        raise ValueError('Chosen rule edit has no observed semantic effect')
    return {'passed':True,'role':role,'source':path,'control_image':control['image_sha256'],'edited_image':variant['image_sha256'],'observed_field':field}

def load_rule_edit_receipt(path, binding):
    """Consume an actual edited-project run, never an isolated policy fixture.

    Each side points at the existing actual-project driver's result, typed ABI
    receipt and completed-frame capture, all hashed. No selected source change
    is sufficient without its changed-image execution and measured effect.
    """
    path=Path(path).resolve(strict=True)
    receipt=json.loads(path.read_text(encoding='utf-8'));checked={str(path):sha(path)}
    control=receipt['control'];variant=receipt['variant'];role=receipt['role']
    if control['source_inventory']!=binding['source_inventory'] or control['image_sha256']!=sha(binding['executable']):
        raise ValueError('Edited-rule control is not the current final18 source/image')
    def capture(side):
        values={}
        for key in ['result','observation','program_contract']:
            reference=side['captures'][key];source=(path.parent/reference['path']).resolve(strict=True)
            if sha(source)!=reference['sha256']:raise ValueError('Edited-rule capture hash mismatch')
            checked[str(source)]=sha(source);values[key]=json.loads(source.read_text(encoding='utf-8'))
        result=values['result'];contract=values['program_contract']
        if result.get('current_contract_sha256')!=side['captures']['program_contract']['sha256'] or result.get('complete_observation_sha256')!=side['captures']['observation']['sha256']:
            raise ValueError('Edited-rule result does not bind its exact capture/ABI files')
        if result.get('accepted')is not True or result.get('executable_sha256')!=side['image_sha256'] or contract.get('executable_sha256')!=side['image_sha256']:
            raise ValueError('Edited-rule capture used a foreign/unaccepted image')
        if result.get('slow_frames')!=0 or result.get('max_frame_us',25001)>25000:
            raise ValueError('Edited-rule capture fails unchanged25ms gate')
        if any(result.get(k,-1)!=0 for k in ['max_dropped_triangles','max_stream_skipped_chunks','max_stream_failed_chunks','missing_sound_assets']):
            raise ValueError('Edited-rule capture lacks strict resource checks')
        for selected,definition in contract['scene']['selected_rule_definitions'].items():
            if side['selected_rule_files'].get(selected)!=definition['path'] or side['source_inventory'].get(definition['path'])!=definition['sha256']:
                raise ValueError('Edited-rule capture selected a different implementation')
        return values['observation']['rows']
    baseline=capture(control);edited=capture(variant)
    return validate_rule_edit(control,variant,role,baseline,edited),checked
