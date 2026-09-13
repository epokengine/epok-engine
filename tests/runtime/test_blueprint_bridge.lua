-- Real LuaJIT/FFI, with deterministic emulator/socket services for protocol QA.
local ffi=require('ffi')
local memory=ffi.new('uint8_t[2097152]')
local words=ffi.cast('uint32_t*',memory+4096)
words[0]=0x55425144;words[1]=1;words[2]=10;words[3]=20;words[4]=2;words[5]=3;words[6]=0
local output,breakpoints={},{}
local receive
local paused=false
local returned=0x80003000
PCSX={
    getMemPtr=function()return memory end,
    getRegisters=function()return {GPR={n={ra=returned}}}end,
    pauseEmulator=function()paused=true end,
    resumeEmulator=function()paused=false end,
    nextTick=function(callback)callback()end,
    Events={createEventListener=function(_,_)return {remove=function()end}end},
    SIO0={slots={{pads={{clearOverride=function()end,setOverride=function()end,getButton=function()return false end}}}}},
    CONSTS={PAD={BUTTON={}}},GPU={takeScreenShot=function()return {width=0,height=0}end},
}
PCSX.addBreakpoint=function(address,_,_,_,callback)
    local point={callback=callback,enabled=true,removed=false}
    function point:disable()self.enabled=false end
    function point:enable()self.enabled=true end
    function point:remove()self.removed=true end
    breakpoints[address]=point;return point
end
local client={}
function client:connect(_,_,callback)callback()end
function client:write(value)output[#output+1]=type(value)=='table' and table.concat(value) or value end
function client:read_start(callback)receive=callback end
function client:is_closing()return false end
function client:close()end
luv={new_tcp=function()return client end}
printError=function(message)error(message)end
EPOK_PORT=1234;EPOK_TOKEN='test';EPOK_DEBUG_CONFIG='testconfig'
dofile=function(path)assert(path=='testconfig');return {version=1,hook=0x80002000,snapshot=0x80001000,breakpoints={{class=10,node=20,owner=65535,generation=0}}}end
assert(loadfile(EPOK_TEST_BRIDGE))()
assert(output[1]=='EPOK test\n')
local function invoke(address)
    local point=assert(breakpoints[address]);assert(point.enabled and not point.removed)
    if point.callback()==false then point.removed=true end
end
local function command(line)
    receive(nil,line..'\n')
    return output[#output]
end
-- Saved breakpoints stop the very first matching Blueprint node.
invoke(0x80002000);assert(paused)
assert(command('F 0')=='EPKF0\n')
assert(output[#output-1]:find('EPKD1 1 1 1 0 1 476\n',1,true)==1)
-- Step exits this hook, rearms at its return PC, then stops the next node.
assert(command('N')=='EPKA1 1\n' and not paused)
assert(not breakpoints[0x80002000].enabled)
invoke(returned);assert(breakpoints[0x80002000].enabled)
words[3]=21;invoke(0x80002000);assert(paused)
assert(command('R')=='EPKA1 1\n' and not paused);invoke(returned)
words[3]=22;invoke(0x80002000);assert(not paused)
assert(command('C')=='EPKA1 1\n')
assert(command('B 10 22 2 999')=='EPKA1 1\n');invoke(0x80002000);assert(not paused)
assert(command('B 10 22 2 3')=='EPKA1 1\n');invoke(0x80002000);assert(paused)
assert(command('R')=='EPKA1 1\n');invoke(returned);assert(command('C')=='EPKA1 1\n')
for node=1,32 do assert(command('B 10 '..node..' 65535 0')=='EPKA1 1\n')end
assert(command('B 10 33 65535 0')=='EPKA1 0\n')
assert(command('B 10 1 65535 0')=='EPKA1 1\n') -- Duplicate does not use capacity.
assert(command('X 10 1 65535 0')=='EPKA1 1\n');assert(command('B 10 33 65535 0')=='EPKA1 1\n')
assert(command('C')=='EPKA1 1\n');words[3]=500
for _=1,200 do invoke(0x80002000)end
command('F 0');assert(output[#output-1]:match('^EPKD1 1 0 0 %d+ 64 476\n'))
assert(command('B 4294967296 1 65535 0')=='EPKA1 0\n')
print('Blueprint Lua bridge protocol, saved breakpoints, real pause/step commands, instance filtering and bounded trace transport passed (emulator services mocked).')
