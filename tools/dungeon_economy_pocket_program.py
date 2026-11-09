"""Pocket protocol authoring for the ordinary editable economy Blueprint class.

The small authoring notation only runs while generating files. It introduces no
interpreter in the game. C++ and Lua keep their ordinary editable source leaves;
every transaction/state decision is implemented in each chosen language.
"""
from dataclasses import dataclass

# Per-instance uint32 state; Base accessors only bound/copy these words.
NAMES='mode phase index needed sector completed expected_serial total old_block block address upgrading starting complete error detail stop_polls start_polls legacy_hash current_hash expected_hash running serial received cursor checksum found matches current_running current_serial temp aux'.split()
SLOTS={name:i for i,name in enumerate(NAMES)}
REPLY,DIR,SAVE,BLOCKS,PACKET,VALUES=0,256,2304,2432,2448,2600
SCRATCH=2664
PUBLIC={'protocol_start','protocol_tick','protocol_cancel','protocol_percent','protocol_observation','pocket_execute','pocket_update'}

@dataclass(eq=False)
class X:
    kind:str
    args:tuple
    def __add__(self,b):return op('add',self,b)
    def __radd__(self,b):return op('add',b,self)
    def __sub__(self,b):return op('subtract',self,b)
    def __mul__(self,b):return op('multiply',self,b)
    def __truediv__(self,b):return op('divide',self,b)
    def __mod__(self,b):return op('modulo',self,b)
    def __lt__(self,b):return op('less',self,b)
    def __le__(self,b):return op('less_equal',self,b)
    def __gt__(self,b):return op('greater',self,b)
    def __ge__(self,b):return op('greater_equal',self,b)
def val(a):return a if isinstance(a,X)else X('lit',(a,))
def op(n,a,b):return X('op',(n,val(a),val(b)))
def eq(a,b):return op('equal',a,b)
def ne(a,b):return op('not_equal',a,b)
def AND(*a):
    out=val(a[0])
    for x in a[1:]:out=op('and',out,x)
    return out
def OR(*a):
    out=val(a[0])
    for x in a[1:]:out=op('or',out,x)
    return out
def NOT(a):return X('not',(val(a),))
def XOR(a,b):return X('xor',(val(a),val(b)))
def WRAP(a,b):return X('wrapping_multiply',(val(a),val(b)))
def P(n):return X('var',(n,))
def R(n):return native('wire_word',SLOTS[n])
def W(n,x):return E(native('wire_set_word',SLOTS[n],x))
def BYTE(at):return native('wire_byte',at)
def SET(at,x):return E(native('wire_set_byte',at,x))
def WORD(at):return native('wire_u32',at)
def PUT(at,x):return E(native('wire_set_u32',at,x))
def APP(n,*a):return native('wire_app_'+n,*a)
def native(n,*a):return X('native',(n,tuple(map(val,a))))
def rule(n,*a):return X('rule',(n,tuple(map(val,a))))
def io(n,*a):return X('io',(n,tuple(map(val,a))))
def field(x,n):return X('field',(x,n))
def PROP(n,field):return X('property',(n,field))
def E(x):return ('eval',x)
def LET(n,x):return ('let',n,x)
def IF(c,y,n=()):return ('if',val(c),list(y),list(n))
def RET(x=None):return ('return',None if x is None else val(x))
def LOOP(count,body,cursor='cursor'):
    return [W(cursor,0),('loop',count,list(body)+[W(cursor,R(cursor)+1)])]
def INC(n):return W(n,R(n)+1)
def CALL(n,*a):return E(rule(n,*a))
def FAIL(error,detail=0):return [CALL('wire_fail',error,detail),RET()]
def DBASE(b):return val(DIR)+val(b)*128
def PK(at,value):return SET(val(PACKET)+val(at),value)
def RECEIVE(at):return BYTE(val(REPLY)+val(at))
def OUT(notice=0):return native('decision',0,notice,0,0,0,PROP('tx','serial'),PROP('tx','total'),False)

# (return kind, parameter list, statements); primitive helpers have explicit Lua
# annotations, public record overrides inherit the native typed signature.
FUNCTIONS={}
def FN(name,ret,params,steps):FUNCTIONS[name]=(ret,params,list(steps))

def build():
    FUNCTIONS.clear()
    FN('wire_fail','void',[('error','uint32'),('detail','uint32')],[W('error',P('error')),W('detail',P('detail')),W('complete',1),W('phase',0),RET()])
    FN('wire_finish','void',[],[CALL('wire_fail',0,0),RET()])
    FN('wire_packet','void',[('command','uint32')],[*LOOP(140,[PK(R('cursor'),0)]),PK(0,0x81),PK(1,P('command')),RET()])
    FN('wire_send','void',[('next','uint32'),('length','uint32'),('length_at','int32')],[
        IF(NOT(native('wire_channel_exists')),FAIL(11)),
        IF(NOT(io('clear_request')),FAIL(12)),
        *LOOP(140,[IF(R('cursor')<P('length'),[IF(NOT(io('set_request_byte',R('cursor'),BYTE(val(PACKET)+R('cursor')))),FAIL(11))])]),
        W('phase',P('next')),E(native('wire_assign_token',io('begin',P('length'),P('length_at')))),
        IF(eq(field(native('wire_token'),'request'),0),[CALL('wire_fail',native('wire_error_code',io('last_rejection')),0)]),RET()])
    FN('wire_read','void',[('next','uint32'),('address','uint32')],[W('address',P('address')),CALL('wire_packet',0x52),PK(4,P('address')/256),PK(5,P('address')%256),CALL('wire_send',P('next'),140,-1),RET()])
    FN('wire_status','void',[('next','uint32')],[CALL('wire_packet',0x5a),CALL('wire_send',P('next'),21,-1),RET()])
    FN('wire_values','void',[],[CALL('wire_packet',0x5b),PK(2,0x82),CALL('wire_send',4,70,4),RET()])
    FN('wire_launch','void',[('next','uint32')],[CALL('wire_packet',0x59),PK(4,R('block')),CALL('wire_send',P('next'),9,-1),RET()])
    FN('wire_wait_start','void',[],[INC('start_polls'),IF(R('start_polls')>120,FAIL(5,40)),CALL('wire_status',18),RET()])
    FN('wire_valid_sector','bool',[],[
        IF(OR(ne(R('received'),140),ne(RECEIVE(139),0x47),ne(RECEIVE(8),R('address')/256),ne(RECEIVE(9),R('address')%256)),[CALL('wire_fail',5,0),RET(False)]),
        W('checksum',XOR(R('address')/256,R('address')%256)),
        *LOOP(128,[W('checksum',XOR(R('checksum'),RECEIVE(R('cursor')+10)))]),
        IF(ne(R('checksum'),RECEIVE(138)),[CALL('wire_fail',5,0),RET(False)]),RET(True)])
    FN('wire_parse_status','bool',[],[
        IF(OR(ne(R('received'),21),ne(RECEIVE(2),0x12)),[CALL('wire_fail',2,0),RET(False)]),
        W('current_running',RECEIVE(3)*256+RECEIVE(4)),W('current_serial',WORD(REPLY+9)),RET(True)])
    FN('wire_capture_status','void',[],[W('running',R('current_running')),W('serial',R('current_serial')),RET()])
    FN('wire_source','uint32',[('at','uint32')],[IF(AND(ne(R('upgrading'),0),eq(R('index'),0),eq(R('sector'),APP('save_offset')/128)),[RET(BYTE(val(SAVE)+P('at')))]),RET(APP('image_byte',R('index')*8192+R('sector')*128+P('at')))])
    FN('wire_write','void',[('next','uint32'),('address','uint32'),('source','uint32')],[
        CALL('wire_packet',0x57),PK(4,P('address')/256),PK(5,P('address')%256),W('checksum',XOR(P('address')/256,P('address')%256)),
        *LOOP(128,[IF(eq(P('source'),0),[W('temp',rule('wire_source',R('cursor')))], [W('temp',BYTE(P('source')+R('cursor')))]),PK(R('cursor')+6,R('temp')),W('checksum',XOR(R('checksum'),R('temp')))]),
        PK(134,R('checksum')),CALL('wire_send',P('next'),138,-1),RET()])
    FN('wire_write_data','void',[],[CALL('wire_write',6,BYTE(val(BLOCKS)+R('index'))*64+R('sector'),0),RET()])
    FN('wire_named','bool',[('block','uint32')],[
        *LOOP(20,[IF(ne(BYTE(DBASE(P('block'))+10+R('cursor')),APP('name_byte',R('cursor'))),[RET(False)]),IF(eq(APP('name_byte',R('cursor')),0),[RET(True)])]),RET(eq(APP('name_byte',20),0))])
    FN('wire_allocate','void',[],[
        W('needed',APP('size')/8192),W('found',0),
        *LOOP(15,[W('aux',R('cursor')+1),IF(AND(R('found')<R('needed'),eq(BYTE(DBASE(R('aux')))/16,10)),[SET(val(BLOCKS)+R('found'),R('aux')),INC('found')])]),
        IF(ne(R('found'),R('needed')),[IF(ne(R('old_block'),0),FAIL(8,41),FAIL(8))]),W('block',BYTE(BLOCKS)),W('index',0),W('sector',0),CALL('wire_write_data'),RET()])
    FN('wire_existing','void',[],[W('sector',0),W('legacy_hash',2166136261),W('current_hash',2166136261),W('expected_hash',2166136261),CALL('wire_read',13,R('old_block')*64),RET()])
    FN('wire_directory_ready','void',[],[
        W('matches',0),W('block',0),W('found',0),
        *LOOP(15,[W('aux',R('index')+1),IF(eq(BYTE(DBASE(R('aux'))),0x51),[IF(rule('wire_named',R('aux')),[INC('matches'),W('found',R('aux'))])])],cursor='index'),
        IF(R('matches')>1,FAIL(5,42)),
        W('block',R('found')),
        IF(ne(R('block'),0),[
            IF(ne(R('mode'),0),[CALL('wire_status',3),RET()]),
            IF(eq(APP('save_offset'),0),[CALL('wire_launch',8),RET()]),
            IF(OR(ne(APP('size'),8192),ne(WORD(DBASE(R('block'))+4),8192),ne(BYTE(DBASE(R('block'))+8),255),ne(BYTE(DBASE(R('block'))+9),255)),FAIL(5,43)),
            W('old_block',R('block')),CALL('wire_status',10),RET()]),
        IF(ne(R('mode'),0),FAIL(7)),CALL('wire_allocate'),RET()])
    # Each frame is a separate helper: constant128 work bound; parent loop is15.
    FN('wire_commit_frame','void',[('item','uint32')],[
        W('aux',DBASE(BYTE(val(BLOCKS)+P('item')))),*LOOP(128,[SET(R('aux')+R('cursor'),0)]),
        IF(eq(P('item'),0),[SET(R('aux'),0x51),PUT(R('aux')+4,APP('size')),
            *LOOP(20,[SET(R('aux')+10+R('cursor'),APP('name_byte',R('cursor')))])],
            [IF(eq(P('item')+1,R('needed')),[SET(R('aux'),0x53)],[SET(R('aux'),0x52)])]),
        IF(P('item')+1<R('needed'),[SET(R('aux')+8,BYTE(val(BLOCKS)+P('item')+1)-1),SET(R('aux')+9,0)], [SET(R('aux')+8,255),SET(R('aux')+9,255)]),
        W('checksum',0),*LOOP(127,[W('checksum',XOR(R('checksum'),BYTE(R('aux')+R('cursor'))))]),SET(R('aux')+127,R('checksum')),RET()])
    FN('wire_commit','void',[],[
        *LOOP(15,[IF(R('index')<R('needed'),[CALL('wire_commit_frame',R('index'))])],cursor='index'),
        W('index',R('needed')-1),CALL('wire_write',7,BYTE(val(BLOCKS)+R('index')),DBASE(BYTE(val(BLOCKS)+R('index')))),RET()])
    FN('wire_retire','void',[],[W('aux',DBASE(R('old_block'))),SET(R('aux'),0xa1),W('checksum',0),
        *LOOP(127,[W('checksum',XOR(R('checksum'),BYTE(R('aux')+R('cursor'))))]),SET(R('aux')+127,R('checksum')),
        CALL('wire_write',16,R('old_block'),R('aux')),RET()])

    # Public entry points. A selected leaf owns both transaction lifecycle and
    # packet interpretation. All packet copying uses direct typed SDK calls.
    FN('protocol_start','bool',[('mode','uint32'),('amount','uint32'),('expected_serial','uint32')],[
        IF(AND(eq(R('complete'),0),ne(R('phase'),0)),[RET(False)]),
        IF(NOT(native('wire_channel_exists')),[CALL('wire_fail',11,0),RET(False)]),
        *[W(n,0)for n in NAMES],W('mode',P('mode')),W('total',P('amount')),W('expected_serial',P('expected_serial')),
        IF(OR(P('mode')>3,NOT(APP('available')),eq(APP('size'),0),ne(APP('size')%8192,0),APP('size')>15*8192,
            AND(ne(APP('save_offset'),0),OR(ne(APP('save_offset')%128,0),APP('save_offset')>APP('size')-128))),[CALL('wire_fail',11,0),RET(False)]),
        IF(P('mode')>=2,[CALL('wire_status',3)],[CALL('wire_packet',0x58),CALL('wire_send',1,5,-1)]),RET(eq(R('complete'),0))])
    FN('protocol_cancel','void',[],[IF(native('wire_channel_exists'),[E(io('cancel',native('wire_token')))]),CALL('wire_fail',11,44),RET()])
    FN('protocol_percent','uint32',[],[IF(AND(ne(R('complete'),0),eq(R('error'),0)),[RET(100)]),IF(eq(R('needed'),0),[RET(0)]),RET(R('completed')*99/(R('needed')*64))])
    FN('protocol_observation','observation',[],[RET(native('wire_make_observation',ne(R('complete'),0),native('wire_epoch'),R('error'),R('running'),R('block'),R('serial'),WORD(VALUES+8),WORD(VALUES+16),WORD(VALUES),WORD(VALUES+12),rule('protocol_percent')))])

    FN('wire_phase_probe','void',[],[IF(OR(ne(R('received'),5),ne(RECEIVE(2),2)),FAIL(2)),W('index',0),CALL('wire_read',2,0),RET()])
    FN('wire_phase_directory','void',[],[IF(NOT(rule('wire_valid_sector')),[RET()]),
        W('checksum',0),*LOOP(128,[SET(DBASE(R('index'))+R('cursor'),RECEIVE(R('cursor')+10)),IF(R('cursor')<127,[W('checksum',XOR(R('checksum'),RECEIVE(R('cursor')+10)))])]),
        IF(ne(R('checksum'),BYTE(DBASE(R('index'))+127)),FAIL(5)),
        IF(AND(eq(R('index'),0),OR(ne(BYTE(DIR),77),ne(BYTE(DIR+1),67))),FAIL(6)),INC('index'),
        IF(R('index')<16,[CALL('wire_read',2,R('index'))],[CALL('wire_directory_ready')]),RET()])
    FN('wire_phase_upgrade_status','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),CALL('wire_capture_status'),
        IF(eq(R('running'),R('old_block')),[CALL('wire_packet',0x59),PK(5,0x30+R('old_block')),CALL('wire_send',11,9,-1)],[CALL('wire_existing')]),RET()])
    FN('wire_phase_stop','void',[],[IF(OR(ne(R('received'),9),ne(RECEIVE(2),6)),FAIL(5)),CALL('wire_status',12),RET()])
    FN('wire_phase_stopped','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),IF(ne(R('current_serial'),R('serial')),FAIL(5,45)),
        IF(eq(R('current_running'),R('old_block')),[INC('stop_polls'),IF(R('stop_polls')>=120,FAIL(9)),CALL('wire_status',12)],[CALL('wire_existing')]),RET()])
    # FNV explicitly wraps modulo2^32; ordinary script UInt32 arithmetic keeps
    # its standard saturating behavior everywhere else.
    FN('wire_phase_existing','void',[],[IF(NOT(rule('wire_valid_sector')),[RET()]),
        *LOOP(128,[IF(ne(R('sector'),42),[W('legacy_hash',WRAP(XOR(R('legacy_hash'),RECEIVE(R('cursor')+10)),16777619))],[SET(val(SAVE)+R('cursor'),RECEIVE(R('cursor')+10))]),
            IF(ne(R('sector'),APP('save_offset')/128),[W('current_hash',WRAP(XOR(R('current_hash'),RECEIVE(R('cursor')+10)),16777619)),W('expected_hash',WRAP(XOR(R('expected_hash'),APP('image_byte',R('sector')*128+R('cursor'))),16777619))])]),
        INC('sector'),IF(R('sector')<64,[CALL('wire_read',13,R('old_block')*64+R('sector')),RET()]),
        IF(eq(R('current_hash'),R('expected_hash')),[CALL('wire_launch',8),RET()]),IF(ne(R('legacy_hash'),0x99022a33),FAIL(5,46)),
        IF(OR(WORD(SAVE)>1000000,WORD(SAVE+4)>1000000,WORD(SAVE+8)>WORD(SAVE+4)),FAIL(5,47)),W('upgrading',1),CALL('wire_status',14),RET()])
    FN('wire_phase_identity','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),IF(ne(R('current_serial'),R('serial')),FAIL(5,45)),IF(eq(R('current_running'),R('old_block')),FAIL(9)),CALL('wire_allocate'),RET()])
    FN('wire_phase_verify','void',[],[IF(NOT(rule('wire_valid_sector')),[RET()]),
        *LOOP(128,[IF(ne(RECEIVE(R('cursor')+10),rule('wire_source',R('cursor'))),FAIL(5,48))]),
        INC('sector'),IF(eq(R('sector'),64),[W('sector',0),INC('index')]),IF(R('index')<R('needed'),[CALL('wire_read',15,BYTE(val(BLOCKS)+R('index'))*64+R('sector'))],[CALL('wire_commit')]),RET()])
    FN('wire_phase_retire','void',[],[IF(OR(ne(R('received'),138),ne(RECEIVE(137),0x47)),FAIL(4)),CALL('wire_launch',8),RET()])
    FN('wire_phase_status','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),CALL('wire_capture_status'),
        IF(AND(ne(R('expected_serial'),0),ne(R('serial'),R('expected_serial'))),FAIL(5)),
        IF(AND(eq(R('mode'),1),ne(R('running'),R('block'))),[IF(ne(R('running'),0),FAIL(5,49)),W('starting',1),CALL('wire_launch',17),RET()]),
        IF(eq(R('mode'),2),[CALL('wire_packet',0x5c),PK(2,0x80),PK(4,5),PK(6,0xd1),PUT(PACKET+7,R('total')),CALL('wire_send',9,12,-1)],[CALL('wire_values')]),RET()])
    FN('wire_phase_start','void',[],[IF(OR(ne(R('received'),9),ne(RECEIVE(2),6)),FAIL(5)),CALL('wire_wait_start'),RET()])
    FN('wire_phase_await','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),IF(ne(R('current_serial'),R('serial')),FAIL(5,50)),
        IF(eq(R('current_running'),R('block')),[CALL('wire_capture_status'),CALL('wire_values')],
            [IF(eq(R('current_running'),0),[CALL('wire_wait_start')],FAIL(5,51))]),RET()])
    FN('wire_phase_shared','void',[],[
        IF(OR(ne(R('received'),70),ne(RECEIVE(2),255),ne(RECEIVE(3),0),ne(RECEIVE(4),64)),[IF(ne(R('starting'),0),[CALL('wire_wait_start')],FAIL(5)),RET()]),
        *LOOP(64,[SET(val(VALUES)+R('cursor'),RECEIVE(R('cursor')+5))]),
        IF(AND(ne(R('starting'),0),OR(ne(WORD(VALUES+8),0x41535431),ne(WORD(VALUES+16),1))),[CALL('wire_wait_start')],[CALL('wire_status',5)]),RET()])
    FN('wire_phase_confirm','void',[],[IF(NOT(rule('wire_parse_status')),[RET()]),IF(OR(ne(R('current_serial'),R('serial')),ne(R('current_running'),R('running'))),FAIL(5)),CALL('wire_finish'),RET()])
    FN('wire_phase_data','void',[],[IF(OR(ne(R('received'),138),ne(RECEIVE(137),0x47)),FAIL(4)),
        INC('completed'),INC('sector'),IF(eq(R('sector'),64),[W('sector',0),INC('index')]),
        IF(R('index')<R('needed'),[CALL('wire_write_data')],[W('index',0),W('sector',0),CALL('wire_read',15,BYTE(BLOCKS)*64)]),RET()])
    FN('wire_phase_commit','void',[],[IF(OR(ne(R('received'),138),ne(RECEIVE(137),0x47)),FAIL(4)),
        IF(ne(R('index'),0),[W('index',R('index')-1),CALL('wire_write',7,BYTE(val(BLOCKS)+R('index')),DBASE(BYTE(val(BLOCKS)+R('index')))),RET()]),
        IF(ne(R('upgrading'),0),[CALL('wire_retire')],[CALL('wire_launch',8)]),RET()])
    FN('wire_phase_launch','void',[],[IF(OR(ne(R('received'),9),ne(RECEIVE(2),6)),FAIL(5)),CALL('wire_finish'),RET()])
    FN('wire_phase_send','void',[],[IF(OR(ne(R('received'),12),ne(RECEIVE(2),255),ne(RECEIVE(3),1),ne(RECEIVE(5),5)),FAIL(5)),CALL('wire_finish'),RET()])
    phases=['probe','directory','status','shared','confirm','data','commit','launch','send','upgrade_status','stop','stopped','existing','identity','verify','retire','start','await']
    FN('protocol_tick','bool',[('microseconds','uint32')],[
        IF(OR(ne(R('complete'),0),eq(R('phase'),0)),[RET(ne(R('complete'),0))]),
        IF(NOT(native('wire_channel_exists')),[CALL('wire_fail',11,0),RET(True)]),
        E(io('poll',native('wire_token'),P('microseconds'))),LET('sample',io('snapshot',native('wire_token'))),
        IF(NOT(field(P('sample'),'valid')),[CALL('wire_fail',11,0),RET(True)]),IF(NOT(field(P('sample'),'ready')),[RET(False)]),
        W('error',native('wire_error_code',field(P('sample'),'error'))),IF(ne(R('error'),0),[CALL('wire_fail',R('error'),0),RET(True)]),
        W('received',field(P('sample'),'received')),*LOOP(140,[SET(R('cursor'),io('reply_byte',native('wire_token'),R('cursor')))]),
        *[IF(eq(R('phase'),i+1),[CALL('wire_phase_'+name),RET(ne(R('complete'),0))])for i,name in enumerate(phases)],
        CALL('wire_fail',11,0),RET(True)])
    # End-to-end orchestration belongs to the same selected implementation.
    # A shell only supplies the menu action/time and renders the decision.
    FN('pocket_execute','decision',[('action','uint32'),('port','uint32')],[
        LET('gate',rule('pocket_begin',P('action'),APP('available'),P('port'))),
        IF(AND(ne(field(P('gate'),'action'),7),ne(field(P('gate'),'action'),8),ne(field(P('gate'),'action'),9)),[RET(P('gate'))]),
        E(native('wire_configure_channel',P('port'),False)),
        IF(eq(P('action'),2),[E(rule('protocol_start',0,0,0))]),
        IF(eq(P('action'),3),[E(rule('protocol_start',1,0,0))]),
        IF(eq(P('action'),5),[E(rule('protocol_start',2,PROP('tx','total'),PROP('tx','serial')))]),
        IF(ne(R('complete'),0),[RET(rule('pocket_request_failed',PROP('pocket','epoch'),R('error')))]),RET(P('gate'))])
    FN('pocket_update','decision',[('dt','fixed'),('microseconds','uint32')],[
        IF(PROP('pocket','operation')>=4,[
            IF(eq(PROP('pocket','operation'),7),[E(rule('advance_ack',P('dt')))]),
            E(rule('protocol_tick',P('microseconds'))),
            IF(AND(ne(R('complete'),0),ne(R('error'),0),OR(eq(PROP('pocket','operation'),4),eq(PROP('pocket','operation'),5))),[
                LET('failed',rule('pocket_request_failed',PROP('pocket','epoch'),R('error'))),IF(ne(R('detail'),0),[RET(OUT(R('detail')))]),RET(P('failed'))]),
            RET(rule('pocket_observe',rule('protocol_observation')))]),
        IF(eq(PROP('pocket','operation'),3),[
            LET('next',rule('pocket_advance',P('dt'))),IF(eq(field(P('next'),'action'),10),[
                E(native('wire_configure_channel',PROP('pocket','port'),False)),
                IF(NOT(rule('protocol_start',3,0,PROP('tx','serial'))),[RET(rule('pocket_request_failed',PROP('pocket','epoch'),R('error')))]),RET(OUT())]),RET(P('next'))]),
        RET(OUT())])
    return FUNCTIONS
