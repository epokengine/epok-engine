import unittest
from epok_mcp_session import frame_sequence


class FrameSequenceTests(unittest.TestCase):
    def test_selects_active_runtime_and_does_not_invent_progress(self):
        state=dict(game_frame={'sequence':100},native_frame={'sequence':37})
        self.assertEqual(frame_sequence(state),100)
        state['play_profile']={'saved':{'runtime':'native_pc'}}
        self.assertEqual(frame_sequence(state),37)
        state['native_frame']=None
        self.assertEqual(frame_sequence(state),0)
        self.assertEqual(frame_sequence({}),0)


if __name__=='__main__':unittest.main()
