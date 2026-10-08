-- Optional host-only per-instruction cycle profile of one ELF-verified function.
-- Every instruction address of the function gets an Exec breakpoint; the
-- emulated cycles between consecutive hits are charged to the earlier
-- instruction, so a call's callee time lands on its delay slot. Verified
-- return sites close the open interval. Breakpoints stay disabled outside the
-- requested frame window. No guest instruction patches, timers, input or
-- memory writes.
return function(config,guarded,cycles)
    local first,last=config.first,config.first+config.frames-1
    local hits,charged,handles={},{},{}
    local open_pc,open_cycles,enabled,frames,discarded=nil,nil,false,0,0
    local function close(now)
        if open_pc then charged[open_pc]=(charged[open_pc] or 0)+(now-open_cycles) end
        open_pc=nil
    end
    local step=guarded(function(address)
        local now=cycles()
        close(now)
        hits[address]=(hits[address] or 0)+1
        open_pc,open_cycles=address,now
    end)
    for address=config.entry,config.entry+config.size-4,4 do
        local handle=PCSX.addBreakpoint(address,'Exec',4,'Epok pc profile',step)
        handle:disable()
        handles[#handles+1]=handle
    end
    for _,address in ipairs(config.returns) do
        local handle=PCSX.addBreakpoint(address,'Exec',4,'Epok pc profile return',guarded(function() close(cycles()) end))
        handle:disable()
        handles[#handles+1]=handle
    end
    local function set(state)
        if state==enabled then return end
        for _,handle in ipairs(handles) do if state then handle:enable() else handle:disable() end end
        enabled=state
    end
    return {
        -- Called at each frame entry with the index of the frame about to run.
        frame=function(index)
            -- A return site always closes a pass, so an open interval here
            -- means an unverified exit; it is counted, never charged.
            if open_pc then discarded=discarded+1;open_pc=nil end
            if index>=first and index<=last then frames=frames+1;set(true) else set(false) end
        end,
        reset=function() set(false);hits={};charged={};open_pc=nil;frames=0;discarded=0 end,
        json=function()
            local parts={'{"version":1,"entry":'..config.entry..',"size":'..config.size..',"first":'..first
                ..',"frames":'..frames..',"discarded":'..discarded..',"open":'..tostring(open_pc~=nil)..',"pcs":['}
            local n=0
            for address,count in pairs(hits) do
                if n>0 then parts[#parts+1]=',' end
                parts[#parts+1]='['..address..','..count..','..(charged[address] or 0)..']';n=n+1
            end
            parts[#parts+1]=']}'
            return table.concat(parts)
        end,
        handles=handles,
    }
end
