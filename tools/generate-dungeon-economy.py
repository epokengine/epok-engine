"""Regenerate the editable project economy policy graphs."""
import importlib.util
import json
from pathlib import Path
import uuid
ROOT=Path(__file__).resolve().parents[1]
NS = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
identity = lambda name: str(uuid.uuid5(NS, "DungeonEconomyPolicy/" + name))
F, U, I, B, V = ({"kind": k} for k in ("fixed", "uint32", "int32", "bool", "void"))
record = lambda name, fields: {"kind": "record", "cpp_name": name, "fields": [{"name": n, "value_type": t} for n, t in fields]}
RECEIPT = record("DungeonSupplyReceipt", [("serial", U), ("total", U)])
TRANSFER = record("DungeonSupplyTransfer", [("result", I), ("amount", U)])
SAVE_FIELDS = [(n, U) for n in ("signature", "version", "ore", "credits", "medkits", "tanks", "buster")] + [(f"r{i}", RECEIPT) for i in range(8)]
SAVE = record("DungeonEconomySave", SAVE_FIELDS)
TX_FIELDS = [("phase", U), ("request", U), ("serial", U), ("total", U), ("heal", U), ("ack_pending", B), ("ack_time", F)]
TX = record("DungeonSupplyTransaction", TX_FIELDS)
DECISION_FIELDS = [("action", U), ("notice", U), ("result", I), ("amount", U), ("heal", U), ("serial", U), ("total", U), ("changed", B)]
DECISION = record("DungeonSupplyDecision", DECISION_FIELDS)
TUNING = [("balance_limit", 1000000), ("medkit_cap", 9), ("tank_cap", 9), ("buster_cap", 3), ("sale_rate", 10),
          ("medkit_price", 30), ("tank_price", 80), ("buster_price", 120), ("medkit_heal", 35), ("tank_heal", 100), ("buster_damage", 8)]
METHODS = [
    ("valid", B, [("value", SAVE)], True),
    ("price", U, [("value", SAVE), ("item", U)], True),
    ("damage_bonus", U, [], True),
    ("find_receipt", U, [("serial", U)], False),
    ("transfer", TRANSFER, [("serial", U), ("total", U), ("claimed", U)], False),
    ("sell", B, [], False), ("buy", B, [("item", U)], False),
    ("activate", DECISION, [("menu", U), ("selection", U), ("can_heal", B)], False),
    ("receive", DECISION, [("running", B), ("signature", U), ("version", U), ("serial", U), ("total", U), ("claimed", U)], False),
    ("can_save", B, [], True),
    ("save_started", DECISION, [("request", U), ("accepted", B)], False),
    ("save_completed", DECISION, [("request", U), ("succeeded", B)], False),
    ("ack_started", DECISION, [("accepted", B)], False),
    ("ack_sent", DECISION, [("succeeded", B)], False),
    ("advance_ack", DECISION, [("dt", F)], False),
    ("receipt_checked", DECISION, [("succeeded", B), ("serial", U), ("signature", U), ("claimed", U)], False),
    ("load_result", DECISION, [("candidate", SAVE), ("succeeded", B), ("missing", B)], False),
    ("leave", DECISION, [], False),
]
spec = importlib.util.spec_from_file_location('supply_graph_base', ROOT/'tools/generate-dungeon-run-flow.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
literal, link = base.literal, base.link
u = lambda x: literal(x, U)
i = lambda x: literal(x, I)
f = lambda x: literal(float(x), F)
b = lambda x: literal(bool(x), B)
EVENTS = {n: identity('fn/'+n) for n, _, _, _ in METHODS}
COPIES = {'assign_saved','assign_pending','assign_tx','assign_loaded','empty_save','wallet','make_tx','make_transfer','decision','pending_receipt','saved_receipt','write_receipt'}


class Graph(base.Graph):
    def node(self, kind, inputs=None):
        node = {'id': identity(f'{self.name}/node/{len(self.nodes)}'), 'kind': kind if isinstance(kind,dict) else {'kind':kind}, 'inputs': inputs or {}, 'outputs': {}}
        self.nodes.append(node)
        return node

    def p(self, name, field=''):
        if field:self.split.append(identity(self.name+'/entry')+':out:'+name)
        return {'kind':'link','node':identity(self.name+'/entry'),'pin':name+('.'+field if field else '')}

    def r(self, name, field=''):
        return self.read(identity(name), field)

    def q(self, name):return self.r('pending',name)
    def s(self, name):return self.r('saved',name)
    def t(self, name):return self.r('tuning',name)
    def x(self, name):return self.r('tx',name)
    def neg(self, value):return link(self.node('not',{'value':value}))
    def eq(self, a, value):return self.op('equal',a,u(value))

    def call(self, name, inputs=None):
        fn = EVENTS.get(name, identity(name) if name in COPIES else identity('helper/'+name))
        return self.node({'kind':'call','function':fn},inputs)

    def field(self, node, field):
        self.split.append(node['id']+':out:value')
        return link(node,'value.'+field)

    def out(self, action=0, notice=0, result=None, amount=None, heal=None, changed=False):
        return link(self.call('decision',{'action':u(action),'notice':u(notice),'result':i(0) if result is None else result,
            'amount':u(0) if amount is None else amount,'heal':u(0) if heal is None else heal,'serial':self.x('serial'),'total':self.x('total'),'changed':b(changed)}))

    def tx(self, **changes):
        return self.call('assign_tx',{'value':link(self.call('make_tx',{n:changes.get(n,self.x(n)) for n in ('phase','request','serial','total','heal','ack_pending','ack_time')}))})

    def intent(self, phase, heal=None, ack=False, serial=None, total=None):
        return self.tx(phase=u(phase),request=u(0),serial=u(0) if serial is None else serial,total=u(0) if total is None else total,
                       heal=u(0) if heal is None else heal,ack_pending=b(ack),ack_time=f(0))

    def wallet(self, **changes):return self.call('wallet',{n:changes.get(n,self.q(n)) for n in ('ore','credits','medkits','tanks','buster')})
    def transfer_result(self, result=0, amount=None):return link(self.call('make_transfer',{'result':i(result),'amount':u(0) if amount is None else amount}))

    def finish(self, steps):
        graph=super().finish(steps)
        self.nodes[-1]['id']=identity(self.name+'/entry')
        graph['entry']=self.nodes[-1]['id']
        graph['id']=identity('graph/'+self.name) if self.name in EVENTS else identity('helper/'+self.name)
        if self.name in EVENTS:graph['override_id']=EVENTS[self.name]
        else:graph.pop('override_id',None)
        return graph


def inventory_graphs():
    graphs,split=[],[]
    def add(g,steps):graphs.append(g.finish(steps));split.extend(g.split)
    def event(name):
        _,returns,params,_=next(m for m in METHODS if m[0]==name)
        return Graph(name,returns,params)
    def failed(g,condition,notice):return g.condition(condition,[g.ret(g.out(0,notice))])

    g=event('valid')
    checks=[g.eq(g.p('value','signature'),0x41535431),g.eq(g.p('value','version'),1)]
    checks += [g.op('less_equal',g.p('value',field),g.t(cap)) for field,cap in [('ore','balance_limit'),('credits','balance_limit'),('medkits','medkit_cap'),('tanks','tank_cap'),('buster','buster_cap')]]
    checks += [g.op('less_equal',g.p('value',f'r{n}.total'),g.t('balance_limit')) for n in range(8)]
    add(g,[g.ret(g.all(*checks))])

    g=event('price');level=g.p('value','buster');divisor=g.op('add',level,u(1))
    add(g,[g.condition(g.eq(g.p('item'),0),[g.ret(g.t('medkit_price'))]),g.condition(g.eq(g.p('item'),1),[g.ret(g.t('tank_price'))]),
        g.condition(g.eq(level,0xffffffff),[g.ret(u(0))]),g.condition(g.op('greater',g.t('buster_price'),g.op('divide',u(0xffffffff),divisor)),[g.ret(u(0))]),g.ret(g.op('multiply',g.t('buster_price'),divisor))])
    g=event('damage_bonus')
    add(g,[g.condition(g.op('greater',g.t('buster_damage'),u(0)),[
        g.condition(g.op('greater',g.s('buster'),g.op('divide',u(0xffffffff),g.t('buster_damage'))),[g.ret(u(0xffffffff))])]),g.ret(g.op('multiply',g.s('buster'),g.t('buster_damage')))])

    g=Graph('durable',B,[('serial',U),('total',U)])
    steps=[]
    for n in range(8):
        r=g.call('saved_receipt',{'index':u(n)})
        steps.append(g.condition(g.all(g.op('equal',g.field(r,'serial'),g.p('serial')),g.op('greater_equal',g.field(r,'total'),g.p('total')),g.op('greater',g.field(r,'total'),u(0))),[g.ret(b(True))]))
    add(g,steps+[g.ret(b(False))])
    g=event('find_receipt');steps=[]
    for n in range(8):
        r=g.call('pending_receipt',{'index':u(n)})
        steps.append(g.condition(g.all(g.op('equal',g.field(r,'serial'),g.p('serial')),g.op('greater',g.field(r,'total'),u(0))),[g.ret(u(n))]))
    for n in range(8):
        r=g.call('pending_receipt',{'index':u(n)})
        steps.append(g.condition(g.eq(g.field(r,'total'),0),[g.call('write_receipt',{'index':u(n),'serial':g.p('serial'),'total':u(0)}),g.ret(u(n))]))
    add(g,steps+[g.ret(u(8))])

    g=Graph('transfer_delta',TRANSFER,[('index',U),('total',U),('claimed',U)])
    r=g.call('pending_receipt',{'index':g.p('index')})
    def delta(previous):
        amount=g.op('subtract',g.p('total'),previous)
        return [g.condition(g.op('less',g.p('total'),previous),[g.ret(g.transfer_result(-3))]),
            g.condition(g.op('greater',amount,g.op('subtract',g.t('balance_limit'),g.q('ore'))),[g.ret(g.transfer_result(-4))]),g.ret(g.transfer_result(0,amount))]
    add(g,[g.condition(g.op('greater',g.p('claimed'),g.field(r,'total')),delta(g.p('claimed')),delta(g.field(r,'total')))])
    g=event('transfer')
    index=g.call('find_receipt',{'serial':g.p('serial')})
    change=g.call('transfer_delta',{'index':link(index),'total':g.p('total'),'claimed':g.p('claimed')})
    add(g,[g.condition(g.any(g.op('greater',g.p('total'),g.t('balance_limit')),g.op('greater',g.p('claimed'),g.p('total'))),[g.ret(g.transfer_result(-1))]),
        g.condition(g.neg(link(g.call('valid',{'value':g.r('pending')}))),[g.ret(g.transfer_result(-5))]),index,
        g.condition(g.op('greater_equal',link(index),u(8)),[g.ret(g.transfer_result(-2))]),change,
        g.condition(g.op('less',g.field(change,'result'),i(0)),[g.ret(link(change))]),
        g.wallet(ore=g.op('add',g.q('ore'),g.field(change,'amount'))),g.call('write_receipt',{'index':link(index),'serial':g.p('serial'),'total':g.p('total')}),g.ret(link(change))])

    g=event('sell')
    add(g,[g.condition(g.any(g.neg(link(g.call('valid',{'value':g.r('pending')}))),g.eq(g.q('ore'),0),g.eq(g.t('sale_rate'),0)),[g.ret(b(False))]),
        g.condition(g.op('greater',g.q('ore'),g.op('divide',g.op('subtract',g.t('balance_limit'),g.q('credits')),g.t('sale_rate'))),[g.ret(b(False))]),
        g.wallet(ore=u(0),credits=g.op('add',g.q('credits'),g.op('multiply',g.q('ore'),g.t('sale_rate')))),g.ret(b(True))])
    g=event('buy');item=g.p('item');cost=g.call('price',{'value':g.r('pending'),'item':item})
    steps=[g.condition(g.any(g.op('greater',item,u(2)),g.neg(link(g.call('valid',{'value':g.r('pending')})))),[g.ret(b(False))])]
    for n,(field,cap) in enumerate([('medkits','medkit_cap'),('tanks','tank_cap'),('buster','buster_cap')]):
        steps.append(g.condition(g.all(g.eq(item,n),g.op('greater_equal',g.q(field),g.t(cap))),[g.ret(b(False))]))
    steps.append(g.condition(g.eq(item,2),[g.condition(g.eq(g.q('buster'),0xffffffff),[g.ret(b(False))]),
        g.condition(g.op('greater',g.t('buster_price'),g.op('divide',u(0xffffffff),g.op('add',g.q('buster'),u(1)))),[g.ret(b(False))])]))
    steps += [g.condition(g.op('less',g.q('credits'),link(cost)),[g.ret(b(False))])]
    for n,field in enumerate(('medkits','tanks','buster')):
        steps.append(g.condition(g.eq(item,n),[g.wallet(**{field:g.op('add',g.q(field),u(1)),'credits':g.op('subtract',g.q('credits'),link(cost))})]))
    add(g,steps+[g.ret(b(True))])

    g=event('activate');menu,selection=g.p('menu'),g.p('selection')
    sold=g.call('sell');bought=g.call('buy',{'item':g.op('subtract',selection,u(1))})
    shop=[g.condition(g.eq(selection,4),[g.ret(g.out(1))]),g.condition(g.eq(selection,0),[sold,g.condition(g.neg(link(sold)),[g.ret(g.out(0,3))]),g.intent(1),g.ret(g.out(4,11,changed=True))]),
        bought,g.condition(g.neg(link(bought)),[g.condition(g.all(g.eq(selection,3),g.op('equal',g.s('buster'),g.t('buster_cap'))),[g.ret(g.out(0,5))]),g.ret(g.out(0,4))]),
        g.intent(1),g.condition(g.eq(selection,3),[g.ret(g.out(4,10,changed=True))]),g.ret(g.out(4,9,changed=True))]
    inventory=[g.condition(g.eq(selection,3),[g.ret(g.out(1))]),g.condition(g.eq(selection,2),[g.ret(g.out(0,6))]),g.condition(g.op('greater',selection,u(3)),[g.ret(g.out(0,27))])]
    for n,(field,heal,notice) in enumerate([('medkits','medkit_heal',12),('tanks','tank_heal',13)]):
        inventory.append(g.condition(g.eq(selection,n),[g.condition(g.eq(g.q(field),0),[g.ret(g.out(0,7))]),g.condition(g.neg(g.p('can_heal')),[g.ret(g.out(0,8))]),
            g.wallet(**{field:g.op('subtract',g.q(field),u(1))}),g.intent(1,heal=g.t(heal)),g.ret(g.out(4,notice,changed=True))]))
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(0)),26),g.condition(g.eq(menu,1),[g.condition(g.eq(selection,0),[g.ret(g.out(2))]),g.condition(g.eq(selection,1),[failed(g,g.neg(g.r('loaded')),1),g.ret(g.out(3))]),g.ret(g.out(1))]),
        g.condition(g.neg(g.r('loaded')),[g.condition(g.any(g.eq(selection,4),g.all(g.eq(menu,3),g.eq(selection,3))),[g.ret(g.out(1))]),g.ret(g.out(0,1))]),
        failed(g,g.neg(link(g.call('valid',{'value':g.r('saved')}))),2),g.call('assign_pending',{'value':g.r('saved')}),g.intent(0),
        g.condition(g.eq(menu,2),shop),g.condition(g.eq(menu,3),inventory),g.ret(g.out())])

    g=event('receive');change=g.call('transfer',{n:g.p(n) for n in ('serial','total','claimed')})
    durable=g.call('durable',{'serial':g.p('serial'),'total':g.p('total')})
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(0)),26),failed(g,g.neg(g.r('loaded')),1),failed(g,g.neg(g.p('running')),14),
        failed(g,g.any(g.op('not_equal',g.p('signature'),u(0x41535431)),g.op('not_equal',g.p('version'),u(1))),15),
        g.call('assign_pending',{'value':g.r('saved')}),change,g.condition(g.op('less',g.field(change,'result'),i(0)),[
            g.condition(g.op('equal',g.field(change,'result'),i(-2)),[g.ret(g.out(0,16,result=g.field(change,'result')))]),g.ret(g.out(0,17,result=g.field(change,'result')))]),
        g.condition(g.eq(g.field(change,'amount'),0),[durable,g.condition(g.all(g.op('less',g.p('claimed'),g.p('total')),link(durable)),[
            g.intent(3,ack=True,serial=g.p('serial'),total=g.p('total')),g.ret(g.out(5,19))]),g.ret(g.out(0,18))]),
        g.intent(1,ack=True,serial=g.p('serial'),total=g.p('total')),g.ret(g.out(4,19,amount=g.field(change,'amount'),changed=True))])
    g=event('can_save');add(g,[g.ret(g.all(g.r('loaded'),g.eq(g.x('phase'),1),link(g.call('valid',{'value':g.r('pending')}))))])
    g=event('save_started')
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(1)),25),g.condition(g.any(g.neg(g.p('accepted')),g.neg(link(g.call('can_save')))),[g.intent(0),g.ret(g.out(0,30))]),g.tx(phase=u(2),request=g.p('request')),g.ret(g.out(0,20))])

    g=Graph('committed_output',DECISION)
    durable=g.call('durable',{'serial':g.x('serial'),'total':g.x('total')})
    add(g,[durable,g.condition(g.all(g.x('ack_pending'),link(durable)),[g.ret(g.out(5,22,heal=g.x('heal'),changed=True))]),g.ret(g.out(0,22,heal=g.x('heal'),changed=True))])
    g=event('save_completed');output=g.call('committed_output')
    add(g,[failed(g,g.any(g.op('not_equal',g.x('phase'),u(2)),g.op('not_equal',g.p('request'),g.x('request'))),25),
        g.condition(g.neg(g.p('succeeded')),[g.intent(0),g.ret(g.out(0,21))]),g.call('assign_saved',{'value':g.r('pending')}),output,
        g.condition(g.eq(g.field(output,'action'),5),[g.tx(phase=u(3),heal=u(0))],[g.tx(phase=u(0),heal=u(0))]),g.ret(link(output))])
    g=event('ack_started');durable=g.call('durable',{'serial':g.x('serial'),'total':g.x('total')})
    add(g,[durable,failed(g,g.any(g.op('not_equal',g.x('phase'),u(3)),g.neg(g.x('ack_pending')),g.neg(link(durable))),25),
        g.condition(g.neg(g.p('accepted')),[g.tx(phase=u(0)),g.ret(g.out(0,23))]),g.tx(phase=u(4),ack_time=f(0)),g.ret(g.out())])
    g=event('ack_sent')
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(4)),25),g.condition(g.p('succeeded'),[g.tx(phase=u(5)),g.ret(g.out())],[g.tx(phase=u(0)),g.ret(g.out(0,23))])])

    g=event('advance_ack');elapsed=g.op('add',g.x('ack_time'),g.p('dt'))
    def bucket(value):
        op='29249825-c121-54e6-a908-8cc041db8971'
        return link(g.node({'kind':'operation','operation':op},{op+':parameter:0':g.op('divide',value,f(1365/4096))}))
    add(g,[g.condition(g.any(g.op('less_equal',g.p('dt'),f(0)),g.op('greater',g.p('dt'),f(1)),g.all(g.op('not_equal',g.x('phase'),u(5)),g.op('not_equal',g.x('phase'),u(6)))),[g.ret(g.out())]),
        g.condition(g.all(g.eq(g.x('phase'),5),g.op('not_equal',bucket(g.x('ack_time')),bucket(elapsed))),[g.tx(phase=u(6),ack_time=elapsed),g.ret(g.out(6))]),g.tx(ack_time=elapsed),g.ret(g.out())])
    g=event('receipt_checked')
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(6)),25),g.condition(g.any(g.neg(g.p('succeeded')),g.op('not_equal',g.p('serial'),g.x('serial'))),[g.tx(phase=u(0)),g.ret(g.out(0,23))]),
        g.condition(g.all(g.eq(g.p('signature'),0x41535431),g.op('greater_equal',g.p('claimed'),g.x('total'))),[g.tx(phase=u(0),ack_pending=b(False)),g.ret(g.out(0,24))]),
        g.condition(g.op('greater',g.x('ack_time'),f(4)),[g.tx(phase=u(0)),g.ret(g.out(0,23))]),g.tx(phase=u(5)),g.ret(g.out())])
    g=event('load_result')
    add(g,[failed(g,g.op('not_equal',g.x('phase'),u(0)),26),g.condition(g.p('missing'),[g.call('assign_saved',{'value':link(g.call('empty_save'))}),
        g.call('assign_pending',{'value':g.r('saved')}),g.call('assign_loaded',{'value':b(True)}),g.intent(0),g.ret(g.out(0,29,changed=True))]),
        g.condition(g.neg(g.p('succeeded')),[g.call('assign_loaded',{'value':b(False)}),g.ret(g.out(0,21))]),
        g.condition(g.neg(link(g.call('valid',{'value':g.p('candidate')}))),[g.call('assign_loaded',{'value':b(False)}),g.ret(g.out(0,2))]),
        g.call('assign_saved',{'value':g.p('candidate')}),g.call('assign_pending',{'value':g.r('saved')}),g.call('assign_loaded',{'value':b(True)}),g.intent(0),g.ret(g.out(0,28,changed=True))])
    g=event('leave')
    add(g,[failed(g,g.op('greater',g.x('phase'),u(1)),26),g.call('assign_pending',{'value':g.r('saved')}),g.intent(0),g.ret(g.out())])
    positions={n['id']:[float(i%8*280),float(i//8*170)] for graph in graphs for i,n in enumerate(graph['nodes'])}
    return {'version':8,'id':identity('class'),'name':'DungeonEconomyPolicy','parent':identity('base'),'family':'component','defaults':{},'variables':[],
            'functions':graphs,'template':{'actors':[]},'layout':{'positions':positions,'comments':{},'split_pins':sorted(set(split))}}

def build():
    import sys
    sys.path.insert(0,str(ROOT/"tools"))
    from dungeon_economy_pocket import extend
    return extend(inventory_graphs())


if __name__=="__main__":
 destination=ROOT/"resources/templates/dungeon-complete-flavors/blueprint/DungeonEconomyPolicy.epokbp"
 destination.write_text(json.dumps(build(),indent=2)+"\n",encoding="utf-8")
 print(destination)
