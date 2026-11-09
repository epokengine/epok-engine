"""Explicit position-only setup schedules and strict P7 surface contracts."""
from pathlib import Path
import hashlib
import math


def plan(binding):
    # A full neutral second after tick240 keeps its final complete render frame
    # identical to the default route even with the maximum8 catch-up steps.
    stages=[];start=301
    def add(name,setup,measure,pose=None,buttons=0,**extra):
        nonlocal start
        stage=dict(name=name,start=start,setup=setup,measure=measure,buttons=buttons,**extra)
        if pose is not None:stage['pose']=pose
        stages.append(stage)
        start+=setup+measure
    if binding['template']=='third-person':
        add('ramp',75,70,[-12,0,2.5],1<<5)
        add('wall',75,70,[-14.5,0,-10],1<<7)
        add('boom-blocked',75,70,[-7,0,-14.5])
        add('boom-clear',75,90,[-7,0,-6])
        actors=binding['scene']['actors'];roles=binding['installation']['roles']
        camera=next(a for a in actors if a['class']['class_id']==roles['camera']['class_id'])
        input=next(c for c in camera['components'] if c['class']['class_id']==roles['camera_input']['class_id'])
        look_rate=(input.get('properties') or {}).get('yaw_rate',140)
        if not 0<look_rate<=4096:raise ValueError('Unsupported observed camera-input tuning')
        return dict(prefix_steps=240,prefix_padding=60,steps=start-1,stages=stages,look_rate=look_rate)
    # Use a real jump during setup to seed upward velocity. Only position is
    # then changed, before measured traversal. No motor/body state is written.
    add('one-way',22,120,[-184,84],jump_setup_step=21,second_pose_step=22,second_pose=[-184,50])
    add('plank-jump',0,80,jump_measure_step=1)
    add('lift-carry',75,160,lift_pose=True)
    actors={a['id']:a for a in binding['scene']['actors']}
    lift=next(a for a in actors.values() if any(c['class']['name']=='epok::PathFollow2DComponent' for c in a['components']))
    offset=[0,0];parent=lift.get('attach',{}).get('actor');visited=set()
    while parent:
        if parent in visited:raise ValueError('Cyclic lift parent setup')
        visited.add(parent);actor=actors[parent];root=next(c for c in actor['components'] if c.get('root'));p=root.get('properties') or {}
        if p.get('rotation',0)!=0 or p.get('scale',[1,1])!=[1,1]:raise ValueError('Surface setup requires source-reviewed identity lift parent scale/rotation')
        position=p.get('position',[0,0]);offset=[offset[i]+round(position[i]*4096) for i in range(2)]
        parent=actor.get('attach',{}).get('actor')
    return dict(prefix_steps=240,prefix_padding=60,steps=start-1,stages=stages,lift_parent_offset=offset)


def validate(value,binding):
    route=value.get('physical_route');expected=plan(binding)
    if route!=expected:raise ValueError('Physical setup/input schedule is foreign or incomplete')
    trace=value.get('physical_trace',[]);setups=value.get('pose_setups',[])
    actual_setups=[(s.get('phase'),s.get('tick')) for s in setups]
    expected_setups=[]
    for stage in route['stages']:
        if stage.get('pose') is not None or stage.get('lift_pose'):expected_setups.append((stage['name'],stage['start']))
        if stage.get('second_pose_step'):expected_setups.append((stage['name'],stage['start']+stage['second_pose_step']-1))
    if actual_setups!=expected_setups:raise ValueError('Missing, extra or ongoing position setup write')
    for setup in setups:
        stage=next(s for s in route['stages'] if s['name']==setup['phase'])
        if not stage['start']<=setup['tick']<stage['start']+stage['setup'] or setup.get('kind')!='position-only':raise ValueError('Position write escaped setup into a measured phase')
        expected_pose=stage.get('second_pose') if setup['tick']!=stage['start'] else stage.get('pose')
        if expected_pose is not None and setup.get('position')!=[round(v*4096) for v in expected_pose]:raise ValueError('Actual initial pose differs from reviewed setup timeline')
        if stage.get('lift_pose'):
            support=setup.get('sampled_support_world')
            if not support or setup.get('position')!=[support[0],support[1]-15*4096]:raise ValueError('Lift setup did not use the actual sampled moving platform')
    counts={}
    for stage in route['stages']:
        rows=[r for r in trace if r.get('phase')==stage['name']]
        if [r.get('step') for r in rows]!=list(range(1,stage['measure']+1)):raise ValueError('Missing actual measured physical phase steps')
        if any(r.get('global_tick')!=stage['start']+stage['setup']+r['step']-1 for r in rows):raise ValueError('Measured phase includes setup or a skipped Time step')
        if any(r['sample'].get('motion_status')!=0 or r['sample'].get('animation_status')!=0 or r['sample'].get('camera_valid') is not True for r in rows):raise ValueError('Actual measured motor/animation/camera status failed')
        counts[stage['name']]=rows
    if binding['template']=='third-person':
        ramp=counts['ramp'];up=[r for r in ramp if r['sample']['grounded'] and r['sample']['position'][1]>.2*4096]
        down=[r for r in ramp if r['sample']['position'][0]>-8.5*4096 and r['sample']['grounded'] and abs(r['sample']['position'][1])<.1*4096]
        if not up or not down or up[0]['step']>=down[-1]['step']:raise ValueError('Real ramp ascent/descent missing')
        wall=counts['wall']
        if sum(r['sample']['blocked'] for r in wall)<30 or min(r['sample']['position'][0] for r in wall)<-15.3*4096:raise ValueError('Real wall response fails unchanged P7 contract')
        blocked=counts['boom-blocked'];clear=counts['boom-clear']
        if sum(r['sample']['camera_blocked'] for r in blocked)<60:raise ValueError('Real boom occlusion/retraction missing')
        if any(r['sample']['camera_blocked'] and not -15.5*4096<r['sample']['camera_position'][2]<r['sample']['position'][2] for r in blocked):raise ValueError('Retracted camera crossed source wall/player bounds')
        if sum(not r['sample']['camera_blocked'] for r in clear[10:])<70:raise ValueError('Real boom recovery missing')
        for r in clear[10:]:
            s=r['sample'];reach=r['camera_distance']*math.cos(math.radians(s['camera_pitch']/4096))
            if abs(s['camera_position'][2]-(s['position'][2]-reach))>.02*4096:raise ValueError('Real clear boom reach fails original P7 tolerance')
    else:
        seeds=value.get('physical_setup_observations',[])
        one_stage=next(s for s in route['stages'] if s['name']=='one-way')
        if len(seeds)!=1 or seeds[0].get('phase')!='one-way' or seeds[0].get('tick')!=one_stage['start']+one_stage['jump_setup_step']-1 or seeds[0].get('kind')!='actual-jump-seed' or seeds[0].get('jumped') is not True or seeds[0].get('velocity',[0,0])[1]>=0:raise ValueError('One-way setup lacks an actual input-driven jump; no velocity may be forced')
        one=counts['one-way'];plank=lambda r:r['sample']['grounded'] and r['sample']['hit_tile'] and r['sample']['tile'][1]==2 and -13<=r['sample']['tile'][0]<=-11
        if not any(r['sample']['velocity'][1]<0 and r['sample']['position'][1]+12*4096<32*4096 for r in one):raise ValueError('Real upward one-way traversal missing')
        if any(r['on_ceiling'] for r in one):raise ValueError('One-way platform incorrectly blocked upward movement')
        landed=[r for r in one if plank(r)]
        if len(landed)<30 or any(abs(r['sample']['position'][1]-20*4096)>.1*4096 for r in landed):raise ValueError('Real one-way landing fails original P7 contract')
        if sum(r['sample']['jumped'] for r in counts['plank-jump'])!=1:raise ValueError('Real jump from landed one-way platform missing or repeated')
        carry=counts['lift-carry'];carried=[r for r in carry if r['sample']['on_lift']]
        moving=0
        if len(carried)<120:raise ValueError('Real moving-platform contact/carry below original P7 threshold')
        for index,row in enumerate(carry):
            if not row['sample']['on_lift']:continue
            p=row['sample']['position'];at=row['lift_world_position']
            if abs(p[0]-at[0])>.1*4096 or abs(p[1]-(at[1]-15*4096))>.1*4096:raise ValueError('Actual player is not supported by the live lift')
            if index>1 and carry[index-1]['sample']['on_lift']:
                prior=carry[index-1];delta=[at[i]-prior['lift_world_position'][i] for i in range(2)]
                if any(delta):
                    moving+=1
                    if any(abs((p[i]-prior['sample']['position'][i])-delta[i])>.1*4096 for i in range(2)):raise ValueError('Player displacement does not match the actual moving lift')
        if moving<100:raise ValueError('Actual moving carry below original P7 threshold')
    return {'physical_scenarios':{name:True for name in counts},'setup_excluded_from_measured_phase_comparison':True,
            'position_writes_confined_to_explicit_setup':True}


def normalized(value):
    # Startup/no-world-write prefix stays byte-for-byte comparable. Setup rows
    # and absolute addresses are intentionally not phase-comparison samples.
    phases=[]
    for row in value['physical_trace']:
        item={k:v for k,v in row.items() if k not in ['global_tick','sample','input']}
        item['sample']={k:v for k,v in row['sample'].items() if k!='step'}
        item['input']={k:v for k,v in row['input'].items() if k!='step'}
        phases.append(item)
    return {'default_prefix':value['trace'][:240],'measured_phases':phases}
