-- Optional host-only function timers. ELF/JAL-derived entry and return sites;
-- no guest instruction patches, timers, input or memory writes.
return function(config,guarded,cycles)
    local calls,pending,handles={},{},{}
    local function hook(address,label,callback)
        handles[#handles+1]=PCSX.addBreakpoint(address,'Exec',4,label,guarded(callback))
    end
    for index,definition in ipairs(config) do
        local allowed={}
        for _,address in ipairs(definition[2]) do allowed[address]=true end
        hook(definition[1],'Epok cost entry '..index,function()
            local r=PCSX.getRegisters().GPR.n
            local ra,sp=tonumber(r.ra),tonumber(r.sp)
            assert(allowed[ra],'Unverified indirect/tail caller of cost hook '..index)
            assert(not pending[index],'Recursive/overlapping cost hook '..index)
            pending[index]={cycles(),ra,sp}
        end)
        for _,address in ipairs(definition[2]) do
            hook(address,'Epok cost return '..index,function()
                local call=pending[index]
                if not call then return end -- profiling may begin inside a call
                assert(call[2]==address,'Wrong cost return site')
                assert(call[3]==tonumber(PCSX.getRegisters().GPR.n.sp),'Cost return stack mismatch')
                assert(#calls<100000,'Function cost trace overflow')
                calls[#calls+1]={index,call[1],cycles(),address,call[3]}
                pending[index]=nil
            end)
        end
    end
    return {
        reset=function() calls={};pending={} end,
        json=function()
            local parts={'{"version":1,"calls":['}
            for i,call in ipairs(calls) do
                if i>1 then parts[#parts+1]=',' end
                parts[#parts+1]='['..table.concat(call,',')..']'
            end
            parts[#parts+1]='],"incomplete":['
            local first=true
            for index in pairs(pending) do
                if not first then parts[#parts+1]=',' end
                parts[#parts+1]=tostring(index);first=false
            end
            parts[#parts+1]=']}'
            return table.concat(parts)
        end,
        handles=handles,
    }
end
