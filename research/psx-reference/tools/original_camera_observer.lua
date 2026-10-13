-- Optional read-only trace of SCUS_942.44 camera target/rail blending.
-- Addresses and post-delay-slot return sites are from OriginalCameraFollowAudit.
return function(word, half, guarded, enabled)
    local rows, pending, operation, property, overflow = {}, nil, nil, nil, false
    local handles = {}
    local function triple(address) return {word(address),word(address+4),word(address+8)} end
    local function hook(address, name, callback)
        handles[#handles+1] = PCSX.addBreakpoint(address,'Exec',4,name,guarded(function()
            if enabled() then callback(PCSX.getRegisters()) end
        end))
    end
    local returns = {
        [0x800252cc]=1,[0x800252e0]=1,[0x800252f4]=1,[0x80025354]=1,
        [0x800253c0]=1,[0x800253d4]=1,[0x800253e8]=1,[0x80025cd0]=1,
        [0x8002530c]=2,[0x8002531c]=2,[0x8002532c]=2,
        [0x80025434]=3,[0x8002544c]=3,[0x80025464]=3,
    }
    local property_returns = {[0x8002527c]=0x183,[0x800252ac]=0x183,
                              [0x800254a0]=0x119,[0x80025c94]=0x16e}
    hook(0x80025188,'Camera follow begin',function(r)
        assert(not pending,'Overlapping camera follow')
        if #rows >= 2048 then overflow=true;return end
        pending={cycles=tonumber(PCSX.getCPUCycles()),caller=tonumber(r.GPR.n.ra),
            target=triple(tonumber(r.GPR.n.a0)),camera=triple(0x80065cf0),
            base=triple(0x80065d20),previous=triple(0x80065da8),
            flags={half(0x8005c63a),half(0x8005c63e),word(0x80065d80),word(0x80065d58),
                word(0x8005c69c),word(0x8005c690),word(0x8005c66c),word(0x8005c670),
                word(0x8005c674),word(0x8003ae8c),word(0x8005c6e0)},
            transition=triple(0x8005c678),route={word(0x80065d64),word(0x80065d68),
                word(0x80065d6c),word(0x80065d70)},operations={},properties={}}
    end)
    hook(0x800251f4,'Camera look direction ready',function(r)
        if pending then pending.look=triple(tonumber(r.GPR.n.sp)+0x28) end
    end)
    for address,kind in pairs({[0x80031e10]=1,[0x80031f9c]=2,[0x80031e34]=3}) do
        hook(address,'Camera angle helper begin',function(r)
            local ra=tonumber(r.GPR.n.ra)
            if pending and returns[ra]==kind then
                assert(not operation,'Overlapping angle helper')
                operation={kind,ra,tonumber(r.GPR.n.a0),tonumber(r.GPR.n.a1),
                           tonumber(r.GPR.n.a2),tonumber(r.GPR.n.a3)}
            end
        end)
    end
    for address in pairs(returns) do
        hook(address,'Camera angle helper result',function(r)
            if pending then
                assert(operation and operation[2]==address,'Missing angle helper entry')
                operation[7]=tonumber(r.GPR.n.v0)
                pending.operations[#pending.operations+1]=operation;operation=nil
            end
        end)
    end
    hook(0x80032710,'Camera property begin',function(r)
        local ra=tonumber(r.GPR.n.ra)
        if pending and property_returns[ra] then
            assert(not property,'Overlapping camera property')
            property={property_returns[ra],tonumber(r.GPR.n.a0),tonumber(r.GPR.n.a2),ra}
        end
    end)
    for address,id in pairs(property_returns) do
        hook(address,'Camera property result',function(r)
            if pending then
                assert(property and property[4]==address,'Missing camera property entry')
                local pointer=tonumber(r.GPR.n.v0)
                property[5]=pointer
                if pointer~=0 then
                    local count=id==0x119 and 4 or id==0x183 and 3 or 2
                    for i=0,count-1 do property[#property+1]=half(pointer+2*i) end
                end
                pending.properties[#pending.properties+1]=property;property=nil
            end
        end)
    end
    hook(0x8002559c,'Camera follow result',function()
        if pending then
            assert(not operation and not property,'Incomplete camera helper')
            pending.angles=triple(0x80065cfc)
            rows[#rows+1]=pending;pending=nil
        end
    end)
    local function json(value)
        if type(value)~='table' then return tostring(value) end
        local parts={}
        if #value>0 or next(value)==nil then
            for _,v in ipairs(value) do parts[#parts+1]=json(v) end
            return '['..table.concat(parts,',')..']'
        end
        local keys={};for key in pairs(value) do keys[#keys+1]=key end;table.sort(keys)
        for _,key in ipairs(keys) do parts[#parts+1]='"'..key..'":'..json(value[key]) end
        return '{'..table.concat(parts,',')..'}'
    end
    return {
        reset=function() rows={};pending=nil;operation=nil;property=nil;overflow=false end,
        json=function() return json({version=1,overflow=overflow,incomplete=pending~=nil,rows=rows}) end,
        handles=handles,
    }
end
