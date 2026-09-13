import copy
from pathlib import Path
import struct
import unittest

from profile_forest_deterministic import (CAPACITY, FIELDS, META, STRIDE, TAIL,
                                        compare_replays, decode_rows, expected_schedule, instrument)
from profile_forest_route import profile


class DeterministicReplayTests(unittest.TestCase):
    def test_schedule_preserves_every_phase_target(self):
        for steps in ([1], [2], [1,2]):
            schedule=expected_schedule(steps)
            self.assertEqual(sum(row[2] for row in schedule), CAPACITY)
            self.assertEqual([row[1] for i,row in enumerate(schedule)
                              if i+1==len(schedule) or schedule[i+1][0]!=row[0]],
                             [12,40,12,40,30,110,35,12])
            self.assertTrue(all(1<=row[2]<=2 for row in schedule))

    def test_instrumentation_retains_update_and_pauses_clock(self):
        source='''using namespace epok;
void ForestController::start(Transform& t) { clip(3); }
void ForestController::frame_update(Transform&,uint32_t) { if(true){old();} }
void ForestController::update(Transform& t,Fixed dt) { original_move(t,dt); }
'''
        result=instrument(source,[1,2])
        self.assertIn('void ForestController::update(Transform& t,Fixed dt) { original_move(t,dt); }',result)
        self.assertIn('time.set_paused(true)',result)
        self.assertIn('time.begin_tick()',result)
        self.assertIn(f'forest_replay_rows[{len(expected_schedule([1,2]))}][{STRIDE}]',result)
        self.assertNotIn('time.reset(',result)
        self.assertNotIn('old();',result)
        with self.assertRaises(ValueError): instrument(result,[1,2])

    def test_decode_and_comparison_require_exact_workload(self):
        schedule=expected_schedule([1,2])
        words=[]
        for index,(phase,ticks,steps) in enumerate(schedule):
            meta=dict.fromkeys(META,0)
            meta.update(index=index,phase=phase,phase_ticks=ticks,replay_steps=steps)
            perf=dict.fromkeys(FIELDS,0);perf.update(frame=index+21,frame_scanlines=100)
            tail=dict.fromkeys(TAIL,0);tail['frame_microseconds']=6400
            words.extend([meta[n] for n in META]+[perf[n] for n in FIELDS]+[tail[n] for n in TAIL])
        rows=decode_rows(struct.pack(f'<{len(words)}I',*words),len(schedule))
        self.assertEqual(profile(rows,{})['samples'],len(schedule))
        self.assertEqual(rows[0]['steps'],0)
        self.assertEqual(rows[1]['replay_steps'],2)
        base=dict(frames=rows,build=dict(instrumentation_sha256='a',original_controller_sha256='b',steps_pattern=[1,2]))
        vram=bytes(1024*512*2)
        self.assertTrue(compare_replays(base,copy.deepcopy(base),vram,vram)['passed'])
        changed=copy.deepcopy(base);changed['frames'][7]['replay']['camera_x']=1
        with self.assertRaisesRegex(ValueError,'state/steps differ'):compare_replays(base,changed,vram,vram)
        changed=copy.deepcopy(base);changed['frames'][7]['visible_chunks']=1
        with self.assertRaisesRegex(ValueError,'geometry counter'):compare_replays(base,changed,vram,vram)
        slower=copy.deepcopy(base)
        for row in slower['frames']: row['frame_scanlines']+=1
        self.assertFalse(compare_replays(base,slower,vram,vram)['passed'])


if __name__=='__main__':unittest.main()
