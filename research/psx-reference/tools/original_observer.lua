-- Read-only PSX observer; controller overrides and save-state restore are explicit.
-- All transport is an outgoing authenticated connection to the local controller.
local ffi = require('ffi')
local client = luv.new_tcp()
client:nodelay(true)
local pending, sequence, vsyncs, closed = '', 0, 0, false
local buttons = {'SELECT','L3','R3','START','UP','RIGHT','DOWN','LEFT',
                 'L2','R2','L1','R1','TRIANGLE','CIRCLE','CROSS','SQUARE'}
local pad = PCSX.SIO0.slots[1].pads[1]
local function set_buttons(mask)
    for i, name in ipairs(buttons) do
        local button = PCSX.CONSTS.PAD.BUTTON[name]
        if button then
            if bit.band(mask, bit.lshift(1, i-1)) ~= 0 then pad.setOverride(button)
            else pad.clearOverride(button) end
        end
    end
end
local profiling, rows, overflow, previous = false, {}, false, nil
local presentations, presentation_phases, last_vsync = {}, {}, nil
local world_samples, pending_sample = {}, nil
local world_start, world_cycles, world_calls = nil, 0, 0
local world_tokens, packet_bytes, packet_begin, world_buffer = 0, 0, nil, nil
local last_camera, last_callback, last_mode, last_argument = {0,0,0}, 0, 0, 0
local last_gte = {}
local callback_error
-- The bundled Redux predates this optional API. -1 means unavailable, never 0 FPS.
local function guest_fps()
    return type(PCSX.GPU.getGuestFPS)=='function' and tonumber(PCSX.GPU.getGuestFPS()) or -1
end
local function json_string(value)
    return '"'..value:gsub('[%z\1-\31\\"]', function(c)
        return string.format('\\u%04x', string.byte(c))
    end)..'"'
end
local memory = PCSX.getMemPtr()
local function word(address)
    assert(address >= 0x80000000 and address <= 0x801ffffc and address % 4 == 0)
    return tonumber(ffi.cast('const uint32_t*', memory + address - 0x80000000)[0])
end
local function half(address)
    assert(address >= 0x80000000 and address <= 0x801ffffe and address % 2 == 0)
    return tonumber(ffi.cast('const uint16_t*', memory + address - 0x80000000)[0])
end
local function guarded(callback)
    return function()
        local ok, why = pcall(callback)
        if not ok then callback_error=tostring(why); profiling=false end
        return true
    end
end
local camera_trace
if ORIGINAL_CAMERA_SCRIPT then
    camera_trace=assert(loadfile(ORIGINAL_CAMERA_SCRIPT))()(word,half,guarded,function() return profiling end)
end
-- Entry and common epilogue verified in /SCUS_942.44 in Ghidra.
ORIGINAL_WORLD_BEGIN = PCSX.addBreakpoint(0x80040ed8, 'Exec', 4, 'Original world begin', guarded(function()
    if not profiling then return end
    assert(not world_start, 'Overlapping world invocation')
    world_start = tonumber(PCSX.getCPUCycles())
    local registers = PCSX.getRegisters()
    local count = half(tonumber(registers.GPR.n.a0))
    assert(count <= 32767, 'Invalid signed SLST count')
    -- Header count, not count-1: tokens begin at +4 and the reverse cursor
    -- starts at +2+count*2 (80040f34..50 / 80041058). Do not infer emitted
    -- packet count from this header; clipping and the special pass still apply.
    world_tokens = world_tokens + count
    world_buffer = tonumber(registers.GPR.n.a1)
    packet_begin = word(world_buffer+8)
    if count > 1 then
        local config = tonumber(registers.GPR.n.a2)
        local camera = word(config)
        for i=1,3 do last_camera[i]=word(camera+(i-1)*4) end
        last_callback=word(config+0x14);last_mode=word(config+0x28)
        last_argument=tonumber(registers.GPR.n.a3)
        if #world_samples < 4 then
            assert(count <= 4096, 'Study snapshot SLST bound exceeded')
            local address=tonumber(registers.GPR.n.a0)
            local sample={world_start,address,count,half(address+2),last_camera[1],last_camera[2],last_camera[3],
                          last_callback,last_mode,last_argument}
            local tokens={}
            for i=1,count do tokens[i]=half(address+2+i*2) end
            pending_sample={header=sample,tokens=tokens}
        end
    end
end))
-- Capture after TR=0, ZSF3/4 and the optional setup callback. The observed
-- callbacks 8003f518/80041e1c do not load R; it is supplied by the caller.
ORIGINAL_WORLD_SETUP = PCSX.addBreakpoint(0x80041020, 'Exec', 4, 'Original world setup complete', guarded(function()
    if not profiling then return end
    local registers = PCSX.getRegisters()
    for i=0,31 do last_gte[i+1]=tonumber(registers.CP2C.r[i]) end
    if pending_sample then
        local sample=pending_sample.header
        for i=1,32 do sample[#sample+1]=last_gte[i] end
        for _,token in ipairs(pending_sample.tokens) do sample[#sample+1]=token end
        world_samples[#world_samples+1]=sample;pending_sample=nil
    end
end))
-- 80041514 also covers the empty-list branch (80040f6c), unlike 8004150c.
ORIGINAL_WORLD_END = PCSX.addBreakpoint(0x80041514, 'Exec', 4, 'Original world common epilogue', guarded(function()
    if not profiling then return end
    if world_start then
        world_cycles=world_cycles+tonumber(PCSX.getCPUCycles())-world_start
        world_calls=world_calls+1
        packet_bytes=packet_bytes+word(world_buffer+8)-packet_begin
        world_start=nil
    end
end))
ORIGINAL_FRAME = PCSX.addBreakpoint(0x80016634, 'Exec', 4, 'Original frame finalization', guarded(function()
    if not profiling then return end
    local now=tonumber(PCSX.getCPUCycles())
    if previous then
        if #rows < 12000 then
            local row={now,now-previous,world_cycles,world_calls,guest_fps(),
                       word(0x8003ae8c),word(0x80068fa4),vsyncs}
            for i=1,32 do row[#row+1]=last_gte[i] or 0 end
            row[#row+1]=world_tokens
            for i=1,3 do row[#row+1]=last_camera[i] end
            row[#row+1]=last_callback;row[#row+1]=last_mode;row[#row+1]=last_argument
            row[#row+1]=packet_bytes
            rows[#rows+1]=row
        else overflow=true end
    end
    previous=now;world_cycles=0;world_calls=0;world_tokens=0;packet_bytes=0
end))
-- SetDispEnv, after the optional debug print and before sending GP1(05h).
-- Record actual requested display origins; no dependency on newer FPS APIs.
ORIGINAL_PRESENT = PCSX.addBreakpoint(0x8004ce84, 'Exec', 4, 'Original display origin', guarded(function()
    if not profiling then return end
    if #presentations >= 12000 then overflow=true;return end
    local display=tonumber(PCSX.getRegisters().GPR.n.s0)
    local now=tonumber(PCSX.getCPUCycles())
    presentations[#presentations+1]={now,vsyncs,word(display)}
    -- Optional side channel keeps the established v3 presentation row intact.
    -- This is phase since Redux's event, not a hardware scanout/tearing oracle.
    presentation_phases[#presentation_phases+1]={now,last_vsync and (now-last_vsync) or -1}
end))
ORIGINAL_VSYNC = PCSX.Events.createEventListener('GPU::Vsync', function()
    vsyncs=vsyncs+1;last_vsync=tonumber(PCSX.getCPUCycles())
end)
local function data(bytes)
    assert(#bytes <= 32*1024*1024, 'Oversized study payload')
    client:write({'CSD1 '..#bytes..'\n',bytes})
end
local function frame(mask)
    set_buttons(mask)
    local ss=PCSX.GPU.takeScreenShot()
    if ss.width==0 or ss.height==0 then client:write('EPKF0\n');return end
    local depth=tonumber(ss.bpp)==0 and 2 or 3
    sequence=sequence+1
    local header=string.format('CSF1 %d %d %d %d %d %.0f %d %.3f %d\n',
        ss.width,ss.height,depth,sequence,mask,tonumber(PCSX.getCPUCycles()),vsyncs,
        guest_fps(),tonumber(PCSX.getRegisters().pc))
    client:write({header,tostring(ss.data)})
end
local function command(line)
    if line=='M' then data(ffi.string(memory,0x200000));return end
    if line=='S' then data(tostring(PCSX.createSaveState()));return end
    if line=='T' then data(PCSX.getSaveStateProtoSchema());return end
    if line=='H' then PCSX.pauseEmulator();data('paused');return end
    if line=='C' then PCSX.resumeEmulator();data('running');return end
    if line=='P' then
        profiling=true;rows={};presentations={};presentation_phases={};overflow=false;previous=nil;world_start=nil;world_cycles=0;world_calls=0;callback_error=nil
        world_samples={};pending_sample=nil
        if camera_trace then camera_trace.reset() end
        world_tokens=0;packet_bytes=0;last_gte={};last_camera={0,0,0};last_callback=0;last_mode=0;last_argument=0
        data('profiling');return
    end
    if line=='R' then
        profiling=false
        local output={'{"version":3,"cpu_clock_hz":'..tonumber(PCSX.CONSTS.CPU.CLOCKSPEED)..',"overflow":'..tostring(overflow)..',"callback_error":'..(callback_error and json_string(callback_error) or 'null')..',"rows":['}
        for i,row in ipairs(rows) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]='],"presentations":['
        for i,row in ipairs(presentations) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]='],"presentation_phases":['
        for i,row in ipairs(presentation_phases) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]='],"world_samples":['
        for i,row in ipairs(world_samples) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]=']'
        if camera_trace then output[#output+1]=',"camera_trace":'..camera_trace.json() end
        output[#output+1]='}'
        data(table.concat(output));return
    end
    local state=line:match('^L ([%w_.-]+)$')
    if state then
        assert(not state:find('..',1,true))
        local file=Support.File.open(ORIGINAL_FOLDER..'/'..state)
        PCSX.loadSaveState(file);file:close();data('restored');return
    end
    local mask=tonumber(line:match('^F (%d+)$'))
    assert(mask and mask<=65535,'Invalid study command')
    frame(mask)
end
local function close()
    if closed then return end
    closed=true;pcall(set_buttons,0)
    if not client:is_closing() then client:close() end
end
client:connect('127.0.0.1',ORIGINAL_PORT,function(err)
    if err then close();return end
    client:write('EPOK '..ORIGINAL_TOKEN..'\n')
    client:read_start(function(err,chunk)
        if err or not chunk then close();return end
        pending=pending..chunk
        if #pending>256 then close();return end
        local line,rest=pending:match('^([^\n]*)\n(.*)$')
        if line then
            pending=rest
            PCSX.nextTick(function()
                if closed then return end
                local ok,why=pcall(command,line)
                if not ok then data('ERROR: '..tostring(why)) end
            end)
        end
    end)
end)
ORIGINAL_QUIT=PCSX.Events.createEventListener('Quitting',close)
