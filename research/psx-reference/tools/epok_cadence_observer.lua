-- Host-only instrumentation, with ELF-verified hooks supplied by the launcher.
-- No emulated memory writes, monkeypatches, controller input or guest timers.
local ffi = require('ffi')
local client = luv.new_tcp()
client:nodelay(true)
local pending, closed, profiling = '', false, false
local memory = PCSX.getMemPtr()
local rows, presentations, overflow, callback_error = {}, {}, false, nil
local vsyncs, last_vsync, frame_begin, flip_begin = 0, nil, nil, nil
local function cycles() return tonumber(PCSX.getCPUCycles()) end
local function word(address)
    assert(address >= 0x80000000 and address <= 0x801ffffc and address % 4 == 0)
    return tonumber(ffi.cast('const uint32_t*', memory + address - 0x80000000)[0])
end
local function quoted(value)
    return '"'..value:gsub('[%z\1-\31\\"]', function(c) return string.format('\\u%04x', string.byte(c)) end)..'"'
end
local function guarded(callback)
    return function(...)
        if profiling then
            local ok, why = pcall(callback, ...)
            if not ok then callback_error=tostring(why); profiling=false end
        end
        return true
    end
end
local costs=FUNCTION_COSTS and dofile('epok_function_costs.lua')(FUNCTION_COSTS,guarded,cycles) or nil
local pcs=PC_PROFILE and dofile('epok_pc_profile.lua')(PC_PROFILE,guarded,cycles) or nil
EPOK_FRAME_BEGIN = PCSX.addBreakpoint(FRAME_ENTRY, 'Exec', 4, 'Epok frame begin', guarded(function()
    if pcs then pcs.frame(#rows) end
    frame_begin=cycles()
end))
EPOK_FLIP_BEGIN = PCSX.addBreakpoint(FLIP_ENTRY, 'Exec', 4, 'Epok completed CPU frame', guarded(function()
    flip_begin=cycles()
    if #rows >= 20000 then overflow=true;profiling=false;return end
    local row={flip_begin,vsyncs,frame_begin and (flip_begin-frame_begin) or -1}
    for _,block in ipairs(COUNTER_BLOCKS) do
        for index=0,block[2]-1 do row[#row+1]=word(block[1]+index*4) end
    end
    rows[#rows+1]=row;frame_begin=nil
end))
EPOK_PRESENT = PCSX.addBreakpoint(GP1_STORE, 'Exec', 4, 'Epok display origin GP1 write', guarded(function()
    if #presentations >= 20000 then overflow=true;profiling=false;return end
    local registers=PCSX.getRegisters().GPR.n
    local command=tonumber(registers.v1)
    assert(tonumber(registers.v0)+0x1814==0x1f801814, 'GP1 address differs from verified code')
    assert(command==0x05000000 or command==0x05040000, 'Unexpected GP1 display origin')
    local now=cycles()
    presentations[#presentations+1]={now,vsyncs,command,word(COUNTER_BLOCKS[1][1]),
        flip_begin and (now-flip_begin) or -1,last_vsync and (now-last_vsync) or -1}
end))
EPOK_VSYNC = PCSX.Events.createEventListener('GPU::Vsync', function()
    vsyncs=vsyncs+1;last_vsync=cycles()
end)
local function data(bytes)
    assert(#bytes<=32*1024*1024,'Oversized observer payload')
    client:write({'CSD1 '..#bytes..'\n',bytes})
end
local function summary()
    return '"overflow":'..tostring(overflow)..',"callback_error":'..(callback_error and quoted(callback_error) or 'null')
end
local function command(line)
    if line=='M' then data(ffi.string(memory,0x200000));return end
    if line=='P' then
        rows={};presentations={};overflow=false;callback_error=nil;frame_begin=nil;flip_begin=nil;profiling=true
        if costs then costs.reset() end
        if pcs then pcs.reset() end
        data('profiling');return
    end
    if line=='Q' then data('{"frames":'..#rows..',"presentations":'..#presentations..','..summary()..'}');return end
    if line=='R' then
        profiling=false
        local output={'{"version":1,"cpu_clock_hz":'..tonumber(PCSX.CONSTS.CPU.CLOCKSPEED)..','..summary()..',"rows":['}
        for i,row in ipairs(rows) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]='],"presentations":['
        for i,row in ipairs(presentations) do
            if i>1 then output[#output+1]=',' end
            output[#output+1]='['..table.concat(row,',')..']'
        end
        output[#output+1]=']'
        if costs then output[#output+1]=',"function_costs":'..costs.json() end
        if pcs then output[#output+1]=',"pc_profile":'..pcs.json() end
        output[#output+1]='}'
        data(table.concat(output));return
    end
    error('Unsupported observer command')
end
local function close()
    if closed then return end
    closed=true
    if not client:is_closing() then client:close() end
end
client:connect('127.0.0.1',OBSERVER_PORT,function(err)
    if err then close();return end
    client:write('EPOK '..OBSERVER_TOKEN..'\n')
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
EPOK_QUIT=PCSX.Events.createEventListener('Quitting',close)
