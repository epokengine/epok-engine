"""Common subexpression locals keep every guard and evaluate pure work once."""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("rule_dsl", ROOT / "tools/dungeon_player_controls.py")
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
F, B = d.F, d.B


def rule(name):
    d.rule(name, F, [("x", F)], [d.ret(d.param("x", F))], False)
    return d.call(name, d.param("x", F))


def text(steps):
    return "\n".join(s.strip() for s in d.body(steps, "cpp"))


class HoistTest(unittest.TestCase):
    def test_repeated_call_on_one_path_becomes_one_local(self):
        work = rule("work")
        steps = d.hoist([d.ret(work + work)])
        self.assertEqual(steps[0][0], "let")
        out = text(steps)
        self.assertEqual(out.count("work("), 1)
        self.assertIn("return (work_1 + work_1);", out)

    def test_each_branch_using_a_value_once_needs_no_local(self):
        work = rule("once")
        flag = d.param("flag", B)
        steps = d.hoist([d.branch(flag, [d.ret(work)]), d.ret(work)])
        self.assertFalse(any(step[0] == "let" for step in steps))

    def test_work_used_after_a_guard_never_moves_above_it(self):
        ready = d.param("ready", B)
        shared = rule("guarded")
        steps = d.hoist([d.branch(d.Not(ready), [d.ret(d.f(0))]), d.ret(shared + shared)])
        self.assertEqual(steps[0][0], "if")
        self.assertEqual(steps[1][0], "let")

    def test_short_circuit_right_operand_stays_conditional(self):
        ready = d.param("ready", B)
        divisor = d.param("divisor", F)
        quotient = d.f(1) / divisor
        guarded = d.And(ready, d.gt(quotient, d.f(0)))
        steps = d.hoist([d.branch(guarded, [d.ret(quotient)]), d.ret(d.f(0))])
        self.assertFalse(any(step[0] == "let" for step in steps))

    def test_nested_expressions_hoist_outermost_first(self):
        inner = rule("inner")
        outer = d.call("inner", inner)
        out = text(d.hoist([d.ret(outer + outer + inner)]))
        self.assertEqual(out.count("inner(x)"), 1)
        self.assertIn("return ((inner_1 + inner_1) + inner_2);", out)

    def test_generated_rules_stay_pure_trees(self):
        d.RULES.clear()
        d.define()
        for name, body in ((n, r["body"]) for n, r in d.RULES.items()):
            lowered = d.hoist(body)
            emitted = text(lowered)
            self.assertNotIn(" = ;", emitted, name)


if __name__ == "__main__":
    unittest.main()
