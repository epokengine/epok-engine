"""Capture the running PSX framebuffer and exported gameplay telemetry."""
from pathlib import Path
import json,re,struct,sys,urllib.request
from PIL import Image
P=Path.cwd() # The caller sets the created project before capturing.
OUT=P/'.epok/validation/dungeon-complete'
def request(path,post=False):
    return urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8077/api/v1/'+path,data=b'' if post else None),timeout=3).read()
def symbols():
    text=(P/'.epok/build/epok.map').read_text()
    return {n:int(re.findall(r'0x([0-9a-fA-F]+)\s+[^\r\n]*'+n+r'[^\r\n]*',text)[-1],16)&0x1fffff
            for n in ['economy_telemetry','dungeon_telemetry','hud_stats','performance_stats']}
def snapshot(name='snapshot',quiet=False):
    ram=request('cpu/ram/raw');sym=symbols();result={}
    for n,count in [('economy_telemetry',14),('dungeon_telemetry',11),('hud_stats',6),('performance_stats',20)]:
        result[n]=struct.unpack_from('<'+str(count)+'I',ram,sym[n])
    mapping=(P/'.epok/build/epok.map').read_text()
    for symbol,count in [('dungeon_profile',13),('dungeon_fps_telemetry',7),('epok::time',8),('dungeon_audio_telemetry',18)]:
        matches=re.findall(r'0x([0-9a-fA-F]+)\s+'+re.escape(symbol)+r'\s*$',mapping,re.M)
        if matches:result[symbol]=struct.unpack_from('<'+str(count)+'I',ram,int(matches[-1],16)&0x1fffff)
    (OUT/(name+'.json')).write_text(json.dumps(result,indent=2))
    raw=request('gpu/vram/raw');pixels=[]
    for y in range(240):
        for x in range(320):
            p=struct.unpack_from('<H',raw,(y*1024+x)*2)[0]
            pixels.append(((p&31)*255//31,((p>>5)&31)*255//31,((p>>10)&31)*255//31))
    im=Image.new('RGB',(320,240));im.putdata(pixels);im.save(OUT/(name+'.png'))
    if not quiet:print(json.dumps(result))
    return result
if __name__=='__main__':snapshot(sys.argv[1] if len(sys.argv)>1 else 'snapshot')
