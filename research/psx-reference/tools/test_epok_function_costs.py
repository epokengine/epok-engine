import struct
import unittest

from analyze_epok_function_costs import costs
from capture_epok_cadence import function_cost_hooks
from test_epok_cadence import fixture


def cost_fixture():
    p,c=fixture()
    c['function_costs']=[dict(label='aim',returns=[0x80010108])]
    p['function_costs']=dict(version=1,incomplete=[],calls=[
        [1,15000,17000,0x80010108,0x801ffff0],
        [1,1110000,1112000,0x80010108,0x801ffff0]])
    return p,c


class FunctionCostsTests(unittest.TestCase):
    def test_frame_accounting_includes_zero_call_frames(self):
        p,c=cost_fixture();r=costs(p,c)
        self.assertEqual(r['complete_frames'],3)
        f=r['functions'][0]
        self.assertEqual((f['calls'],f['frames_with_calls']),(2,2))
        self.assertEqual(f['inclusive_per_complete_frame_ms']['minimum'],0)
        self.assertAlmostEqual(f['inclusive_call_ms']['maximum'],2000/33868.8)

    def test_boundary_call_excluded_not_charged_to_next_frame(self):
        p,c=cost_fixture();p['function_costs']['calls'][0][1]=1
        self.assertEqual(costs(p,c)['excluded_boundary_calls'],1)

    def test_rejects_corrupt_rows(self):
        for index,value in ((0,2),(1,-1),(2,100),(3,0x80012340),(4,0x10000)):
            p,c=cost_fixture();p['function_costs']['calls'][0][index]=value
            with self.assertRaises(ValueError):costs(p,c)
        p,c=cost_fixture();p['function_costs']['calls'].reverse()
        with self.assertRaises(ValueError):costs(p,c)

    def test_nested_scopes_remain_separate(self):
        p,c=cost_fixture();c['function_costs'].append(dict(label='rail',returns=[0x80010208]))
        p['function_costs']['calls'].insert(1,[2,10000,19000,0x80010208,0x801ffff8])
        r=costs(p,c)
        self.assertAlmostEqual(r['functions'][1]['inclusive_call_ms']['maximum'],9000/33868.8)

    def test_direct_calls_are_derived_from_instructions(self):
        names=['advance_rail_aim','advance_rail_navigation','OrderedDrawCursor7prepare','draw_ordered_world']
        symbols={name:(0x80010040+i*4,4,2) for i,name in enumerate(names)}
        code=bytearray(80)
        for i,record in enumerate(symbols.values()):struct.pack_into('<I',code,i*4,0x0c000000|((record[0]>>2)&0x3ffffff))
        result=function_cost_hooks(symbols,[(0x10000,bytes(code))])
        self.assertEqual([h['returns'] for h in result],[[0x80010008+i*4] for i in range(4)])
        for modified in ({}, {**symbols,'second_advance_rail_aim':(0x80010040,4,2)}):
            with self.assertRaises(ValueError):function_cost_hooks(modified,[(0x10000,bytes(code))])
        with self.assertRaises(ValueError):function_cost_hooks(symbols,[(0x10000,bytes(80))])


if __name__=='__main__':unittest.main()
