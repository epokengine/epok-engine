"""Pocket protocol graph authoring for the ordinary economy policy class."""
from pathlib import Path
import importlib.util
import sys
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import dungeon_economy_pocket_program as p
spec=importlib.util.spec_from_file_location('economy_inventory_graph',ROOT/'tools/generate-dungeon-economy.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
U,B,F,V,D=m.U,m.B,m.F,m.V,m.DECISION
QFIELDS=[('operation',U),('port',U),('epoch',U),('installed',U),('error',U),('progress',U)]
OFIELDS=[('ready',B)]+[(n,U)for n in('epoch','error','running_block','expected_block','serial','signature','version','total','claimed','percent')]
FLOW=m.record('DungeonPocketFlow',QFIELDS)
OBS=m.record('DungeonPocketObservation',OFIELDS)
METHODS=[('pocket_begin',D,[('action',U),('app_available',B),('port',U)]),('pocket_progress',D,[('epoch',U),('percent',U)]),
         ('pocket_request_failed',D,[('epoch',U),('error',U)]),('pocket_observe',D,[('sample',OBS)]),('pocket_advance',D,[('dt',F)])]
m.EVENTS.update({name:m.identity('fn/'+name)for name,_,_ in METHODS})
m.COPIES.update(('assign_pocket','make_pocket'))


class Graph(m.Graph):
    def q(self,name):return self.r('pocket',name)
    def put(self,**changes):
        return self.call('assign_pocket',{'value':m.link(self.call('make_pocket',{n:changes.get(n,self.q(n))for n,_ in QFIELDS}))})


def coordinator(asset):
    graphs=[];split=[]
    def add(g,steps):graphs.append(g.finish(steps));split.extend(g.split)
    def event(name):
        _,returns,params=next(v for v in METHODS if v[0]==name)
        return Graph(name,returns,params)
    u,b,link=m.u,m.b,m.link
    g=Graph('pocket_next_request',V,[('operation',U),('port',U),('progress',U)])
    add(g,[g.put(operation=g.p('operation'),port=g.p('port'),progress=g.p('progress'),epoch=g.op('add',g.q('epoch'),u(1)),error=u(0)),
           g.condition(g.eq(g.q('epoch'),0),[g.put(epoch=u(1))]),g.ret()])
    g=event('pocket_begin');action=g.p('action')
    ack=g.call('ack_started',{'accepted':b(True)})
    branches=[]
    for selected,operation,command,notice in[(2,4,7,37),(3,5,8,38),(5,6,9,39)]:
        branches.append(g.condition(g.eq(action,selected),[g.call('pocket_next_request',{'operation':u(operation),'port':g.p('port'),'progress':u(101)if selected==2 else g.q('progress')}),g.ret(g.out(command,notice))]))
    add(g,[g.condition(g.op('not_equal',g.q('operation'),u(0)),[g.ret(g.out(0,26))]),g.condition(g.op('greater',g.p('port'),u(1)),[g.ret(g.out(0,35))]),
        g.condition(g.all(*[g.op('not_equal',action,u(v))for v in(2,3,5)]),[g.ret(g.out(0,27))]),
        g.condition(g.all(g.any(g.eq(action,2),g.eq(action,3)),g.op('not_equal',g.x('phase'),u(0))),[g.ret(g.out(0,26))]),
        g.condition(g.all(g.eq(action,3),g.neg(g.r('loaded'))),[g.ret(g.out(0,1))]),
        g.condition(g.neg(g.p('app_available')),[g.condition(g.eq(action,5),[g.call('ack_started',{'accepted':b(False)}),g.ret(g.out(0,23))]),g.ret(g.out(0,31))]),
        g.condition(g.eq(action,5),[ack,g.condition(g.op('not_equal',g.x('phase'),u(4)),[g.ret(link(ack))])]),*branches,g.ret(g.out())])
    g=event('pocket_progress')
    add(g,[g.condition(g.any(g.op('not_equal',g.p('epoch'),g.q('epoch')),g.op('not_equal',g.q('operation'),u(4))),[g.ret(g.out(0,25))]),
        g.condition(g.any(g.op('greater',g.p('percent'),u(100)),g.op('equal',g.p('percent'),g.q('progress'))),[g.ret(g.out())]),g.put(progress=g.p('percent')),g.ret(g.out(0,32,amount=g.p('percent')))])
    g=Graph('pocket_failure',D,[('previous',U),('error',U)])
    add(g,[g.put(operation=u(0),error=g.p('error')),g.condition(g.eq(g.p('previous'),6),[g.call('ack_sent',{'succeeded':b(False)}),g.ret(g.out(0,23))]),
        g.condition(g.eq(g.p('previous'),7),[g.call('receipt_checked',{'succeeded':b(False),'serial':g.x('serial'),'signature':u(0),'claimed':u(0)}),g.ret(g.out(0,23))]),g.ret(g.out(0,36))])
    g=event('pocket_request_failed');failed=g.call('pocket_failure',{'previous':g.q('operation'),'error':g.p('error')})
    add(g,[g.condition(g.any(g.op('not_equal',g.p('epoch'),g.q('epoch')),g.op('less',g.q('operation'),u(4)),g.op('greater',g.q('operation'),u(7))),[g.ret(g.out(0,25))]),failed,g.ret(link(failed))])
    g=Graph('pocket_success',D,[('previous',U),('sample',OBS)])
    observed=g.call('receive',{'running':g.op('equal',g.p('sample','running_block'),g.p('sample','expected_block')),**{n:g.p('sample',n)for n in('signature','version','serial','total','claimed')}})
    sent=g.call('ack_sent',{'succeeded':b(True)})
    checked=g.call('receipt_checked',{'succeeded':b(True),**{n:g.p('sample',n)for n in('serial','signature','claimed')}})
    add(g,[g.put(operation=u(0),error=u(0)),g.condition(g.eq(g.p('previous'),4),[g.put(installed=g.op('add',g.q('installed'),u(1))),g.ret(g.out(0,33,changed=True))]),
        g.condition(g.eq(g.p('previous'),5),[observed,g.ret(link(observed))]),g.condition(g.eq(g.p('previous'),6),[sent,g.condition(g.eq(g.x('phase'),5),[g.put(operation=u(3))]),g.ret(link(sent))]),
        checked,g.condition(g.eq(g.x('phase'),5),[g.put(operation=u(3))]),g.ret(link(checked))])
    g=event('pocket_observe')
    progress=g.call('pocket_progress',{'epoch':g.p('sample','epoch'),'percent':g.p('sample','percent')})
    failed=g.call('pocket_request_failed',{'epoch':g.p('sample','epoch'),'error':g.p('sample','error')})
    success=g.call('pocket_success',{'previous':g.q('operation'),'sample':g.p('sample')})
    add(g,[g.condition(g.any(g.op('not_equal',g.p('sample','epoch'),g.q('epoch')),g.op('less',g.q('operation'),u(4)),g.op('greater',g.q('operation'),u(7))),[g.ret(g.out(0,25))]),
        g.condition(g.neg(g.p('sample','ready')),[g.condition(g.eq(g.q('operation'),4),[progress,g.ret(link(progress))]),g.ret(g.out())]),
        g.condition(g.op('not_equal',g.p('sample','error'),u(0)),[failed,g.ret(link(failed))]),success,g.ret(link(success))])
    g=event('pocket_advance');advanced=g.call('advance_ack',{'dt':g.p('dt')})
    add(g,[g.condition(g.all(g.op('not_equal',g.q('operation'),u(3)),g.op('not_equal',g.q('operation'),u(7))),[g.ret(g.out())]),advanced,
        g.condition(g.op('not_equal',g.field(advanced,'action'),u(6)),[g.ret(link(advanced))]),
        g.call('pocket_next_request',{'operation':u(7),'port':g.q('port'),'progress':g.q('progress')}),g.ret(g.out(10))])
    asset['functions'].extend(graphs)
    asset['layout']['split_pins']=sorted(set(asset['layout']['split_pins']+split))
    asset['layout']['positions'].update({n['id']:[float(i%8*280),float(i//8*170)]for graph in graphs for i,n in enumerate(graph['nodes'])})
    return asset


n=SimpleNamespace(m=m,OBS=OBS,OFIELDS=OFIELDS,METHODS=METHODS,Graph=Graph)
U,I,B,V=m.U,m.I,m.B,m.V
T={'uint32':U,'int32':I,'bool':B,'fixed':m.F,'void':V,'observation':n.OBS,'decision':m.DECISION}
CT={'uint32':'uint32_t','int32':'int32_t','bool':'bool','fixed':'epok::Fixed','void':'void','observation':'DungeonPocketObservation','decision':'DungeonSupplyDecision'}
LT={'uint32':'UInt32','int32':'Int32','bool':'Bool'}
PROGRAM=p.build()
IO_CLASS='abd3923b-cc40-589c-b755-5d607835a22f'
IO_PREFIX='cpp:c:@N@epok@S@PocketTransactionComponent@F@'
IO_SUFFIX={'clear_request':'#','set_request_byte':'#i#i#','begin':'#i#I#','poll':'#$@N@epok@S@PocketTransferHandle#i#','snapshot':'#$@N@epok@S@PocketTransferHandle#1','reply_byte':'#$@N@epok@S@PocketTransferHandle#i#1','cancel':'#$@N@epok@S@PocketTransferHandle#','last_rejection':'#1'}
IO_PARAMS={'clear_request':[],'set_request_byte':[('index',U),('value',U)],'begin':[('length',U),('length_at',I)],'poll':[('value',None),('microseconds',U)],'snapshot':[('value',None)],'reply_byte':[('value',None),('index',U)],'cancel':[('value',None)],'last_rejection':[]}
NATIVE_PARAMS={
 'wire_word':[('index',U)],'wire_set_word':[('index',U),('value',U)],
 'wire_byte':[('index',U)],'wire_set_byte':[('index',U),('value',U)],
 'wire_u32':[('index',U)],'wire_set_u32':[('index',U),('value',U)],
 'wire_app_available':[],'wire_app_size':[],'wire_app_save_offset':[],'wire_app_image_byte':[('index',U)],'wire_app_name_byte':[('index',U)],
 'wire_channel_exists':[],'wire_configure_channel':[('port',U),('automatic_poll',B)],'wire_token':[],'wire_assign_token':[('value',None)],'wire_error_code':[('value',None)],'wire_epoch':[],
 'decision':[(name,ty)for name,ty in m.DECISION_FIELDS],
 'wire_make_observation':[(name,ty)for name,ty in n.OFIELDS]}
IMPURE={'wire_set_word','wire_set_byte','wire_set_u32','wire_assign_token','wire_configure_channel'}
for name in p.PUBLIC:m.EVENTS[name]=m.identity('fn/'+name)
m.COPIES.update(NATIVE_PARAMS)
class ProtocolGraph(n.Graph):
    def __init__(self,name,returns,parameters):super().__init__(name,returns,parameters);self.locals={}
    def expr(self,x,prolog,want=None):
        k,a=x.kind,x.args
        e=lambda v,w=None:self.expr(v,prolog,w)
        if k=='lit':return m.literal(a[0],B if isinstance(a[0],bool)else (want or(I if a[0]<0 else U)))
        if k=='var':return self.locals.get(a[0],self.p(a[0]))
        if k=='property':return self.r(*a)
        if k=='field':
            root=e(a[0]);self.split.append(root['node']+':out:'+root['pin'].split('.')[0]);return {**root,'pin':root['pin']+'.'+a[1]}
        if k=='not':return m.link(self.node('not',{'value':e(a[0])}))
        if k in('xor','wrapping_multiply'):
            identity='fd15a18f-5bec-56a2-86b5-4543deeba153'if k=='xor'else'36c9a85a-f38e-5dc1-bdde-4bb547859c44'
            return m.link(self.node({'kind':'operation','operation':identity},{identity+':parameter:0':e(a[0]),identity+':parameter:1':e(a[1])}))
        if k=='op':
            if a[0]=='modulo':return self.op('subtract',e(a[1]),self.op('multiply',self.op('divide',e(a[1]),e(a[2])),e(a[2])))
            return self.op(a[0],e(a[1]),e(a[2]))
        if k in('native','rule','io'):
            name,args=a
            if k=='io':params=IO_PARAMS[name]
            elif k=='native':params=NATIVE_PARAMS[name]
            elif name in PROGRAM:params=[(n,T[t])for n,t in PROGRAM[name][1]]
            elif name in {v[0]for v in n.METHODS}:params=next(v[2]for v in n.METHODS if v[0]==name)
            else:params=next(v[2]for v in m.METHODS if v[0]==name)
            inputs={n:e(v,ty)for(n,ty),v in zip(params,args)}
            if k=='io':
                inputs['__target']=self.r('pocket_channel')
                node=self.node({'kind':'call_on','class':IO_CLASS,'function':IO_PREFIX+name+IO_SUFFIX[name]},inputs)
            else:node=self.call(name,inputs)
            if k=='rule'or name in IMPURE or(k=='io'and name not in('snapshot','reply_byte','last_rejection')):prolog.append(node)
            return m.link(node)
        raise ValueError(k)
    def lower(self,steps):
        output=[]
        for st in steps:
            k=st[0];pre=[]
            if k=='eval':
                value=self.expr(st[1],pre);output+=pre
                if not pre or pre[-1]['id']!=value['node']:
                    # Explicit calls can be pure (the ignored snapshot etc.).
                    node=next(n for n in self.nodes if n['id']==value['node']);output.append(node)
            elif k=='let':
                value=self.expr(st[2],pre);output+=pre;self.locals[st[1]]=value
            elif k=='return':
                value=None if st[1]is None else self.expr(st[1],pre,self.returns);output+=pre;output.append(self.ret(value))
            elif k=='if':
                value=self.expr(st[1],pre,B);output+=pre;output.append(self.condition(value,self.lower(st[2]),self.lower(st[3])))
            elif k=='loop':output.append(('loop',st[1],self.lower(st[2])))
            else:raise ValueError(k)
        return output
    def finish_program(self,steps):
        def sequence(steps,continuation=None):
            for st in reversed(steps):
                if isinstance(st,tuple)and st[0]=='loop':
                    node=self.node({'kind':'loop','count':st[1]});body=sequence(st[2])
                    if body:node['outputs']['body']=[body]
                    if continuation:node['outputs']['next']=[continuation]
                elif isinstance(st,tuple):
                    cond,yes,no=st;node=self.node('branch',{'condition':cond})
                    for pin,branch in(('true',yes),('false',no)):
                        at=sequence(branch,continuation)
                        if at:node['outputs'][pin]=[at]
                else:
                    node=st
                    if node['kind']['kind']!='return'and continuation:node['outputs']['next']=[continuation]
                continuation=node['id']
            return continuation
        first=sequence(self.lower(steps));entry=self.node('entry');entry['id']=m.identity(self.name+'/entry');entry['outputs']['next']=[first]
        out={'id':m.identity(('graph/'if self.name in p.PUBLIC else'helper/')+self.name),'name':self.name,'parameters':[{'name':n,'value_type':ty,'direction':'value'}for n,ty in self.parameters],'returns':self.returns,'entry':entry['id'],'nodes':self.nodes}
        if self.name in p.PUBLIC:out['override_id']=m.EVENTS[self.name]
        return out

def extend(asset):
    asset=coordinator(asset)
    graphs=[];split=[]
    for name,(ret,params,steps)in PROGRAM.items():
        g=ProtocolGraph(name,T[ret],[(n,T[t])for n,t in params])
        graphs.append(g.finish_program(steps));split+=g.split
    asset['functions']+=graphs
    asset['layout']['split_pins']=sorted(set(asset['layout']['split_pins']+split))
    asset['layout']['positions'].update({node['id']:[float(i%8*280),float(i//8*170)]for graph in graphs for i,node in enumerate(graph['nodes'])})
    return asset
