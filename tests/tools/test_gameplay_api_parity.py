from __future__ import annotations

import copy
import importlib.util
import json
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "gameplay_api_parity", ROOT / "tools" / "gameplay_api_parity.py"
)
assert SPEC and SPEC.loader
parity = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = parity
SPEC.loader.exec_module(parity)


def catalog() -> dict:
    return {
        "schema": 1,
        "callables": [
            {
                "family": "epok",
                "module": "input",
                "qualified": "epok::Input::axis",
                "source": "runtime/input.hpp",
                "line": 10,
                "signature": "Fixed axis(unsigned port) const",
                "result": "Fixed",
                "parameters": [{"name": "port", "type": "unsigned"}],
                "static": False,
                "const": True,
                "template": False,
                "virtual": False,
            }
        ],
        "properties": [],
    }


def complete_coverage() -> dict:
    data = {
        "schema_version": parity.SCHEMA_VERSION,
        "frozen_denominator": {},
        "capabilities": [],
    }
    grouped = parity.grouped_candidates(catalog())
    key, declarations = next(iter(grouped.items()))
    support = {
        surface: {"state": "implemented", "evidence": "negative-control fixture"}
        for surface in parity.SURFACES
    }
    data["frozen_denominator"] = {
        "candidate_keys_sha256": parity.hashlib.sha256("\n".join(grouped).encode()).hexdigest(),
        "count": 1,
    }
    data["capabilities"].append(
        {
            "id": parity.stable_id(key),
            "candidate_key": key,
            "category": "input_time",
            "declaration_kind": "callable",
            "qualified": "epok::Input::axis",
            "module": "input",
            "declarations": declarations,
            "declaration_hash": parity.declaration_hash(declarations),
            "disposition": "public_gameplay",
            "rationale": "fixture",
            "canonical_public_declaration": declarations[0]["signature"],
            "types_and_directions": {"parameters": [], "returns": "Fixed"},
            "owner_domain": {"owner": "epok::Input", "domain": "input"},
            "lifecycle_phase": "tick",
            "error_contract": "zero on invalid port",
            "expected_complexity": "O(1)",
            "required_cooked_features_assets": [],
            "support": support,
            "example_ids": ["fixture-example"],
            "test_ids": ["fixture-test"],
            "cost_evidence": ["fixture-cost"],
            "compatibility_aliases": [],
            "status": "implemented",
        }
    )
    return data


def validate(data: dict, source: dict | None = None) -> None:
    parity.validate(
        data,
        source or catalog(),
        root=ROOT,
        require_complete=True,
        verify_files=False,
    )


class CoverageNegativeControls(unittest.TestCase):
    def test_complete_fixture_passes(self) -> None:
        validate(complete_coverage())

    def test_new_native_gameplay_operation_fails(self) -> None:
        source = copy.deepcopy(catalog())
        source["callables"].append(
            {
                **source["callables"][0],
                "qualified": "epok::Input::analog_present",
                "signature": "bool analog_present(unsigned port) const",
                "result": "bool",
                "line": 11,
            }
        )
        with self.assertRaisesRegex(parity.CoverageError, "new/uncovered declarations"):
            validate(complete_coverage(), source)

    def test_changed_result_shape_fails(self) -> None:
        source = copy.deepcopy(catalog())
        source["callables"][0]["result"] = "uint32_t"
        source["callables"][0]["signature"] = "uint32_t axis(unsigned port) const"
        with self.assertRaisesRegex(parity.CoverageError, "shape changed"):
            validate(complete_coverage(), source)

    def test_advertised_but_unusable_surface_fails(self) -> None:
        data = complete_coverage()
        data["capabilities"][0]["support"]["lua_vm_source"] = {
            "state": "missing",
            "evidence": "frontend rejects this value type",
        }
        with self.assertRaisesRegex(parity.CoverageError, "lua_vm_source"):
            validate(data)

    def test_declaration_hash_must_match_shape(self) -> None:
        data = complete_coverage()
        data["capabilities"][0]["declarations"][0]["line"] = 99
        with self.assertRaisesRegex(parity.CoverageError, "declaration hash is stale"):
            validate(data)


if __name__ == "__main__":
    unittest.main()
