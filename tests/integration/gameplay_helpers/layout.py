"""Inspect the exact linked MIPS program; no compiler invocation or fixed ABI offsets."""
from pathlib import Path
import re
import subprocess

def tool(root,name,*args):
    folder=Path(root)/'.tools/mips/bin'
    path=folder/('mipsel-none-elf-'+name+('.exe' if (folder/('mipsel-none-elf-'+name+'.exe')).exists() else ''))
    return subprocess.check_output([str(path),*map(str,args)],text=True,encoding='utf-8',errors='replace')

def symbols(root,elf):
    result={}
    for line in tool(root,'nm','-S','--defined-only','--demangle',elf).splitlines():
        match=re.match(r'^([0-9a-fA-F]+)\s+(?:([0-9a-fA-F]+)\s+)?[a-zA-Z]\s+(.+)$',line)
        if match:result[match[3]]={'address':int(match[1],16),'size':int(match[2],16) if match[2] else None}
    return result

def dwarf_structures(text):
    """Read constant member offsets; reject unknown/location-expression layouts."""
    entries=[];current=None
    for line in text.splitlines():
        match=re.match(r'^\s*<(\d+)><([0-9a-fA-F]+)>:.*\((DW_TAG_\w+)\)',line)
        if match:
            current={'depth':int(match[1]),'offset':int(match[2],16),'tag':match[3],'attributes':{}};entries.append(current)
            continue
        match=re.search(r'DW_AT_(\w+)\s*:\s*(.*)$',line)
        if current and match:current['attributes'][match[1]]=match[2].strip()
    def name(value):
        # readelf emits both direct and indirect string encodings.
        if '): ' in value:return value.rsplit('): ',1)[1].strip()
        return re.sub(r'^\([^)]*\)\s*','',value).strip()
    def integer(value):
        value=re.sub(r'^\((?:data\d|udata|sdata|implicit_const)\)\s*','',value)
        if re.fullmatch(r'(?:0x[0-9a-fA-F]+|\d+)',value):return int(value,0)
        raise ValueError('Nonconstant DWARF layout: '+value)
    result={}
    for i,entry in enumerate(entries):
        attrs=entry['attributes']
        if entry['tag'] not in ['DW_TAG_structure_type','DW_TAG_class_type'] or 'name' not in attrs or 'byte_size' not in attrs:continue
        members={}
        for child in entries[i+1:]:
            if child['depth']<=entry['depth']:break
            a=child['attributes']
            if child['depth']==entry['depth']+1 and child['tag']=='DW_TAG_member' and 'name' in a and 'data_member_location' in a:
                try:members[name(a['name'])]=integer(a['data_member_location'])
                except ValueError:members[name(a['name'])]=None
        if members:
            layout={'size':integer(attrs['byte_size']),'members':members}
            result.setdefault(name(attrs['name']),[]).append(layout)
    return result

def require_layout(structures,name,members):
    candidates=[v for v in structures.get(name,[]) if set(members)<=set(v['members']) and all(type(v['members'][m]) is int for m in members)]
    unique={repr((v['size'],tuple(sorted(v['members'].items())))):v for v in candidates}
    if len(unique)!=1:raise ValueError('Missing/ambiguous compiled layout: '+name)
    value=next(iter(unique.values()))
    if any(not 0<=value['members'][m]<value['size'] for m in members):raise ValueError('Member outside compiled object')
    return value

def inspect(root,elf,template='sample'):
    if template not in ['basic','sample']:raise ValueError('Unreviewed actual-template observation boundary: '+template)
    syms=symbols(root,elf)
    def address(name):
        if name not in syms:raise ValueError('Missing exact linked symbol: '+name)
        return syms[name]['address']
    frame=[v['address'] for k,v in syms.items() if k.endswith('GameScene::frame()') and 'lambda' not in k]
    tick=[v['address'] for k,v in syms.items() if k.startswith('epok::Level::tick_actor(')]
    if len(set(frame))!=1:raise ValueError('Missing/ambiguous actual GameScene frame boundary')
    if template=='sample' and len(set(tick))!=1:raise ValueError('Missing/ambiguous selected Spinner simulation boundary')
    structures=dwarf_structures(tool(root,'readelf','--debug-dump=info','--wide',elf))
    required={'ActorData':['transform','name','owner','alive'], 'Transform':['position','rotation','scale'],
              'Actor':['m_component_count'], 'Time':['ticks','dropped_steps','frame_microseconds'],
              'ResourceUsage':['active_texture_bytes','resident_texture_bytes','dropped_primitives'],
              'HudStats':['dropped'], 'AudioSource':['enabled'], 'PerformanceStats':['frame','steps']}
    required['ActorData'].append('audio')
    layouts={name:require_layout(structures,name,members) for name,members in required.items()}
    assert layouts['Transform']['size']==36,'Unsupported Fixed/Transform target ABI'
    actor_bytes=syms['epok::objects']['size']
    assert actor_bytes and actor_bytes%layouts['ActorData']['size']==0,'Object array/ActorData ABI mismatch'
    tick_address=address('epok::time')+layouts['Time']['members']['ticks']
    return {'symbols':{name:address(name) for name in ['epok::objects','epok::object_count','epok::time','epok::performance_stats','epok::actor_stats','epok::resource_usage','epok::hud_stats']},
            'frame_entry':frame[0],'tick_entry':tick[0] if template=='sample' else None,'slot_capacity':actor_bytes//layouts['ActorData']['size'],
            'observation_boundary':'time-write-and-frame-return' if template=='basic' else 'selected-actor-tick-return',
            'tick_counter_address':tick_address,
            'layouts':layouts,'elf_symbol_sizes':{k:v['size'] for k,v in syms.items() if k in ['epok::objects','epok::performance_stats','epok::actor_stats','epok::hud_stats']}}
