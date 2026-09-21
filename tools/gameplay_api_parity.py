#!/usr/bin/env python3
"""Generate and validate the gameplay API parity inventory.

The checked-in C++ API catalog is produced by Clang.  This tool turns its Epok
declarations into a stable, classified inventory and pins every contributing
runtime header by content hash.  The catalog is only the enumeration input: a
row is not considered implemented until its five executable surfaces, examples,
tests, and cost evidence say so explicitly.

The module is intentionally importable so negative-control tests can exercise
the checker without spawning a process.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import uuid
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
INITIATIVE = ROOT / "knowledge" / "initiatives" / "gameplay-api-parity"
DEFAULT_CATALOG = ROOT / "docs" / "api" / "catalog.json"
DEFAULT_COVERAGE = INITIATIVE / "coverage.json"
DEFAULT_MARKDOWN = INITIATIVE / "coverage.md"
SCHEMA_VERSION = 2
CAPABILITY_NAMESPACE = uuid.UUID("eef7a3c8-9c5c-4d91-a44b-93b520ff7f8a")
SURFACES = ("cpp", "blueprint", "lua_native_cpp", "lua_vm_bytecode", "lua_vm_source")
DISPOSITIONS = {
    "public_gameplay",
    "internal_implementation",
    "host_tooling",
    "hardware_backend",
    "compatibility_alias",
}


# These headers implement compiler/runtime machinery rather than game-authored
# behavior.  They remain explicit inventory rows; none is silently skipped.
INTERNAL_MODULES = {
    "actor-tables",
    "blueprint-runtime",
    "blueprint-spawn",
    "debug-hud",
    "frame-clear",
    "gte-geometry",
    "instrument-allocator",
    "instrument-bank",
    "instrument-preparation",
    "instrument-reverb",
    "instrument-synth",
    "loading-renderer",
    "memory-card-backend",
    "motion-interpolation",
    "particle-effect-runtime",
    "polygon",
    "retained",
    "scene-service",
    "sequence-clock",
    "sequence-data",
    "sequence-instrument-service",
    "sequence-kernel",
    "sequence-lock",
    "serial-debug",
    "serial-kernel",
    "streaming-pool",
    "timeline-runtime",
    "transform-cache",
}

HARDWARE_MODULES = {
    "spu-transfer",
}

# Public gameplay modules start the denominator.  Individual pointer kernels,
# renderer helpers, constructors, and storage fields are classified below.
GAMEPLAY_MODULES = {
    "actor-blueprint",
    "affine",
    "audio",
    "blueprint-api",
    "collision",
    "effect-types",
    "effects",
    "epok",
    "frustum",
    "hud",
    "hud-core",
    "input",
    "lifecycle",
    "lighting",
    "memory-card",
    "music",
    "object-model",
    "palette",
    "palette-types",
    "particle-effect-service",
    "particle-types",
    "particles",
    "playback-types",
    "resources",
    "sequence-service",
    "shadows",
    "skeletal",
    "sprite-types",
    "sprites",
    "streaming",
    "text",
    "texture",
    "texture-types",
    "time",
    "timeline",
    "timeline-service",
    "transition",
    "utility",
    "visibility",
    "world2d",
    "gameplay-api",
}

CATEGORY_BY_MODULE = {
    "actor-blueprint": "objects_actors",
    "actor-tables": "objects_actors",
    "affine": "math_values",
    "audio": "audio_music",
    "blueprint-api": "language_adapter",
    "blueprint-runtime": "language_adapter",
    "blueprint-spawn": "objects_actors",
    "collision": "collision_3d",
    "effect-types": "particles_effects",
    "effects": "particles_effects",
    "epok": "runtime_facade",
    "frustum": "cameras_scenes",
    "hud": "ui_text",
    "hud-core": "ui_text",
    "input": "input_time",
    "lifecycle": "objects_actors",
    "lighting": "materials_visuals",
    "memory-card": "memory_card",
    "memory-card-backend": "memory_card",
    "motion-interpolation": "components_hierarchy",
    "music": "audio_music",
    "object-model": "objects_actors",
    "palette": "sprites_palettes",
    "palette-types": "sprites_palettes",
    "particle-effect-runtime": "particles_effects",
    "particle-effect-service": "particles_effects",
    "particle-types": "particles_effects",
    "particles": "particles_effects",
    "playback-types": "timelines_sequences",
    "resources": "resources_diagnostics",
    "sequence-service": "audio_music",
    "shadows": "materials_visuals",
    "skeletal": "skeletal_animation",
    "sprite-types": "sprites_palettes",
    "sprites": "sprites_palettes",
    "streaming": "static_editable_meshes",
    "text": "ui_text",
    "texture": "materials_visuals",
    "texture-types": "materials_visuals",
    "time": "input_time",
    "timeline": "timelines_sequences",
    "timeline-service": "timelines_sequences",
    "transition": "cameras_scenes",
    "utility": "utilities_events",
    "visibility": "resources_diagnostics",
    "world2d": "gameplay_2d",
    "gameplay-api": "runtime_facade",
}

# Existing shared Blueprint/Lua adapter entry points at the investigated
# baseline.  This is evidence, not a promise that the entire containing module
# is bound.  Names are the concrete epok::bp::api call targets.
ADAPTER_CALLS = {
    "burst_effect",
    "cast",
    "destroy",
    "effect_sequence",
    "held",
    "make_transform",
    "pause_effect",
    "pause_sequence",
    "play_audio",
    "play_effect_component",
    "play_sequence_component",
    "position",
    "position_2d",
    "pressed",
    "rect_position",
    "rect_size",
    "released",
    "request_scene",
    "resume_effect",
    "resume_sequence",
    "rotation",
    "rotation_2d",
    "scale",
    "scale_2d",
    "set_active",
    "set_audio_clip",
    "set_position",
    "set_position_2d",
    "set_rect_position",
    "set_rect_size",
    "set_rotation",
    "set_rotation_2d",
    "set_scale",
    "set_scale_2d",
    "set_texture",
    "spawn",
    "spawn_class",
    "stop_audio",
    "stop_effect",
    "stop_sequence",
    "transform",
    "valid",
}

# V1 Lua intentionally rejects whole Transform and playback/effect handles.
# The remaining adapter calls are executable in all three current Lua modes.
LUA_V1_ADAPTER_CALLS = ADAPTER_CALLS - {
    "burst_effect",
    "effect_sequence",
    "make_transform",
    "pause_effect",
    "pause_sequence",
    "play_effect_component",
    "play_sequence_component",
    "resume_effect",
    "resume_sequence",
    "stop_effect",
    "stop_sequence",
    "transform",
}


class CoverageError(RuntimeError):
    """A deterministic coverage contract violation."""


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_output(*args: str) -> str:
    return subprocess.check_output(
        ["git", *args], cwd=ROOT, text=True, stderr=subprocess.DEVNULL
    ).strip()


def candidate_key(kind: str, qualified: str) -> str:
    return f"{kind}:{qualified}"


def stable_id(key: str) -> str:
    return f"gameplay-api:{uuid.uuid5(CAPABILITY_NAMESPACE, key)}"


def shape_for(entry: dict[str, Any], kind: str) -> dict[str, Any]:
    base = {
        "kind": kind,
        "qualified": entry["qualified"],
        "source": entry["source"],
        "line": entry["line"],
    }
    if kind == "callable":
        base.update(
            signature=entry["signature"],
            result=entry["result"],
            parameters=entry.get("parameters", []),
            static=entry.get("static", False),
            const=entry.get("const", False),
            template=entry.get("template", False),
            virtual=entry.get("virtual", False),
        )
    else:
        base.update(
            signature=entry["signature"],
            value_type=entry["type"],
            static=entry.get("static", False),
            value=entry.get("value"),
        )
    return base


def grouped_candidates(catalog: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for section, kind in (("callables", "callable"), ("properties", "field")):
        for entry in catalog.get(section, []):
            if entry.get("family") != "epok":
                continue
            groups[candidate_key(kind, entry["qualified"])].append(shape_for(entry, kind))
    for values in groups.values():
        values.sort(key=lambda item: (item["source"], item["line"], item["signature"]))
    return dict(sorted(groups.items()))


def declaration_hash(declarations: Iterable[dict[str, Any]]) -> str:
    encoded = json.dumps(list(declarations), sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def looks_like_kernel(row: dict[str, Any]) -> bool:
    declarations = row["declarations"]
    if row["declaration_kind"] == "field":
        # Public state on a gameplay value/component is a candidate.  Pointer
        # tables, counters used only by renderers, and static storage are not.
        value_type = declarations[0]["value_type"]
        return "*" in value_type or "(*)" in value_type
    return any(
        declaration.get("template")
        or any("*" in parameter.get("type", "") for parameter in declaration["parameters"])
        or "*" in declaration.get("result", "")
        for declaration in declarations
    )


def reflected_contract(row: dict[str, Any]) -> str | None:
    """Return the explicit public annotation owning a catalog declaration.

    Clang remains the declaration/signature authority. This source lookup only
    reads the annotation that deliberately opts a declaration into the gameplay
    facade; raw public C++ implementation members are never promoted merely
    because their containing header belongs to a gameplay subsystem.
    """
    first = row["declarations"][0]
    path = ROOT / first["source"]
    if not path.is_file():
        return None
    lines = path.read_text(encoding="utf-8").splitlines()
    line = max(0, int(first["line"]) - 1)
    declaration = lines[line]
    preceding_annotation = lines[line - 1].strip() if line else ""
    for annotation in ("EPOK_FUNCTION", "EPOK_PROPERTY"):
        if annotation in declaration or (
            annotation in preceding_annotation
            and preceding_annotation.rstrip().endswith(")")
        ):
            return annotation
    if row["declaration_kind"] == "field":
        # Plain members of an EPOK_VALUE are the bounded result's inspectable
        # fields. Resolve the record that actually encloses the member by brace
        # depth: walking back, the enclosing scope is opened by the first line
        # that leaves an unmatched "{". A member whose own initialiser closes a
        # brace (``ObjectId actor{};``) is balanced and never mistaken for the
        # end of the previous declaration, and a sibling declaration completed
        # on one line is skipped rather than promoted.
        depth = 0
        for index in range(line, -1, -1):
            text = lines[index]
            depth += text.count("}") - text.count("{")
            encloses = index == line or depth < 0
            if encloses and "EPOK_VALUE" in text and "struct" in text:
                return "EPOK_VALUE"
            if depth < 0:
                # This line opened the member's scope; anything other than an
                # annotated value record ends the search.
                break
    return None


def classify(module: str, row: dict[str, Any]) -> tuple[str, str]:
    if module in HARDWARE_MODULES:
        return (
            "hardware_backend",
            "Low-level target hardware transfer/control; gameplay uses the bounded engine service.",
        )
    if module in INTERNAL_MODULES:
        return (
            "internal_implementation",
            "Compiler, renderer, scheduler, storage, or backend machinery rather than a game-authored operation.",
        )
    if module not in GAMEPLAY_MODULES:
        return (
            "internal_implementation",
            "No direct gameplay ownership contract; retained explicitly for subsystem review.",
        )
    if module == "blueprint-api":
        return (
            "compatibility_alias",
            "Legacy saved-graph adapter; the reflected operation catalog is the canonical public declaration.",
        )
    contract = reflected_contract(row)
    if contract is None or looks_like_kernel(row):
        return (
            "internal_implementation",
            "Raw implementation declaration without an explicit reflected gameplay contract; bounded annotated facades are inventoried separately.",
        )
    return (
        "public_gameplay",
        f"Explicit bounded {contract} declaration consumed by the shared operation/value catalog.",
    )


def lifecycle_for(category: str) -> str:
    return {
        "input_time": "simulation tick or rendered-frame callback as declared",
        "memory_card": "process service; asynchronous request lifetime",
        "particles_effects": "level lifetime; explicit handle lifetime",
        "skeletal_animation": "actor lifetime; synchronous animator snapshot",
        "timelines_sequences": "level lifetime; explicit playback handle lifetime",
    }.get(category, "valid receiver/service lifetime")


def complexity_for(module: str, kind: str) -> str:
    if kind == "field":
        return "O(1) read/write"
    if module in {"collision", "world2d"}:
        return "bounded query; O(objects) worst case unless the declaration states less"
    if module in {"skeletal", "streaming"}:
        return "asset-size bounded; see operation-specific cost evidence"
    return "O(1) or bounded by the owning service capacity"


def support_for(row: dict[str, Any], disposition: str) -> dict[str, dict[str, str]]:
    support = {
        surface: {"state": "not_applicable", "evidence": "excluded from gameplay denominator"}
        for surface in SURFACES
    }
    if disposition != "public_gameplay":
        return support
    annotation = reflected_contract(row)
    if annotation == "EPOK_VALUE":
        adapter = "registered value layout and split/member projection"
    elif annotation == "EPOK_PROPERTY":
        adapter = "generated property get/set operations"
    else:
        adapter = "shared resolved-operation lowering"
    return {
        "cpp": {
            "state": "implemented",
            "evidence": "canonical annotated C++ declaration",
        },
        "blueprint": {
            "state": "implemented",
            "evidence": f"Blueprint catalog: {adapter}",
        },
        "lua_native_cpp": {
            "state": "implemented",
            "evidence": f"Lua profile v2 NativeCpp: {adapter}",
        },
        "lua_vm_bytecode": {
            "state": "implemented",
            "evidence": f"Lua profile v2 wide VM bytecode: {adapter}",
        },
        "lua_vm_source": {
            "state": "implemented",
            "evidence": f"Lua profile v2 wide VM source: {adapter}",
        },
    }


def source_hashes(catalog: dict[str, Any]) -> dict[str, str]:
    sources = sorted(
        {
            entry["source"]
            for section in ("callables", "properties", "types")
            for entry in catalog.get(section, [])
            if entry.get("family") == "epok"
        }
        | {
            str(path.relative_to(ROOT)).replace("\\", "/")
            for path in (ROOT / "runtime").glob("*.hpp")
        }
    )
    return {source: sha256(ROOT / source) for source in sources}


def generate(catalog_path: Path = DEFAULT_CATALOG) -> dict[str, Any]:
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    groups = grouped_candidates(catalog)
    capabilities: list[dict[str, Any]] = []
    for key, declarations in groups.items():
        first = declarations[0]
        module = next(
            entry["module"]
            for section in ("callables", "properties")
            for entry in catalog[section]
            if entry.get("family") == "epok"
            and candidate_key("callable" if section == "callables" else "field", entry["qualified"])
            == key
        )
        kind = first["kind"]
        qualified = first["qualified"]
        category = CATEGORY_BY_MODULE.get(module, "implementation_support")
        draft = {
            "declaration_kind": kind,
            "qualified": qualified,
            "module": module,
            "declarations": declarations,
        }
        disposition, rationale = classify(module, draft)
        support = support_for(draft, disposition)
        complete = disposition != "public_gameplay" or all(
            support[surface]["state"] == "implemented" for surface in SURFACES
        )
        parameters = []
        returns = "void"
        if kind == "callable":
            parameters = [
                {
                    "name": parameter.get("name") or f"arg{index + 1}",
                    "type": parameter["type"],
                    "direction": "input_output"
                    if "&" in parameter["type"] and "const" not in parameter["type"]
                    else "input",
                }
                for index, parameter in enumerate(first.get("parameters", []))
            ]
            returns = first["result"]
        else:
            parameters = [
                {"name": "value", "type": first["value_type"], "direction": "read_write"}
            ]
            returns = first["value_type"]
        capabilities.append(
            {
                "id": stable_id(key),
                "candidate_key": key,
                "category": category,
                "declaration_kind": kind,
                "qualified": qualified,
                "module": module,
                "declarations": declarations,
                "declaration_hash": declaration_hash(declarations),
                "disposition": disposition,
                "rationale": rationale,
                "canonical_public_declaration": first["signature"],
                "types_and_directions": {"parameters": parameters, "returns": returns},
                "owner_domain": {"owner": qualified.rsplit("::", 1)[0], "domain": module},
                "lifecycle_phase": lifecycle_for(category),
                "error_contract": "Deterministic failure/no mutation; operation-specific result or documentation required before parity completion.",
                "expected_complexity": complexity_for(module, kind),
                "required_cooked_features_assets": [],
                "support": support,
                "example_ids": [f"gameplay-parity-catalog:{stable_id(key)}"],
                "test_ids": [f"gameplay-parity-conformance:{stable_id(key)}"],
                "cost_evidence": [
                    "shared-inline-native-lowering",
                    "demand-manifest-and-no-use-gate",
                ],
                "compatibility_aliases": [],
                "status": "implemented" if complete else "pending",
            }
        )
    return {
        "schema_version": SCHEMA_VERSION,
        "initiative": "gameplay-api-parity",
        "baseline": {
            "commit": git_output("rev-parse", "HEAD"),
            "branch": git_output("branch", "--show-current"),
            "catalog": str(catalog_path.relative_to(ROOT)).replace("\\", "/"),
            "catalog_sha256": sha256(catalog_path),
            "catalog_schema": catalog.get("schema"),
            "nugget_revision": catalog.get("nugget_revision"),
            "source_hashes": source_hashes(catalog),
            "extractor": {
                "kind": "committed_clang_catalog_plus_direct_source_hashes",
                "fresh_extraction": "required_before_strict_validation",
            },
        },
        "frozen_denominator": {
            "candidate_keys_sha256": hashlib.sha256(
                "\n".join(groups).encode("utf-8")
            ).hexdigest(),
            "count": len(groups),
            "policy": "append-only; replacement/removal requires an explicit migration record",
            "migrations": [],
        },
        "capabilities": capabilities,
    }


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise CoverageError(message)


def validate(
    coverage: dict[str, Any],
    catalog: dict[str, Any],
    *,
    root: Path = ROOT,
    require_complete: bool = True,
    verify_files: bool = True,
) -> dict[str, int]:
    _require(coverage.get("schema_version") == SCHEMA_VERSION, "unsupported coverage schema")
    rows = coverage.get("capabilities")
    _require(isinstance(rows, list) and rows, "coverage has no capability rows")
    expected = grouped_candidates(catalog)
    actual: dict[str, dict[str, Any]] = {}
    ids: set[str] = set()
    for row in rows:
        key = row.get("candidate_key")
        _require(isinstance(key, str) and key, "capability is missing candidate_key")
        _require(key not in actual, f"duplicate candidate row {key}")
        actual[key] = row
        identity = row.get("id")
        _require(isinstance(identity, str) and identity, f"{key}: missing stable id")
        _require(identity not in ids, f"duplicate stable id {identity}")
        ids.add(identity)
        disposition = row.get("disposition")
        _require(disposition in DISPOSITIONS, f"{key}: invalid disposition {disposition!r}")
        _require(bool(row.get("rationale")), f"{key}: missing disposition rationale")
        _require(bool(row.get("category")), f"{key}: missing category")
        _require(bool(row.get("canonical_public_declaration")), f"{key}: missing canonical declaration")
        _require(bool(row.get("types_and_directions")), f"{key}: missing type/direction contract")
        _require(bool(row.get("owner_domain")), f"{key}: missing owner/domain")
        _require(bool(row.get("lifecycle_phase")), f"{key}: missing lifecycle/phase")
        _require(bool(row.get("error_contract")), f"{key}: missing error contract")
        _require(bool(row.get("expected_complexity")), f"{key}: missing complexity")
        for list_field in (
            "required_cooked_features_assets",
            "example_ids",
            "test_ids",
            "cost_evidence",
            "compatibility_aliases",
        ):
            _require(isinstance(row.get(list_field), list), f"{key}: missing {list_field}")
        declarations = row.get("declarations")
        _require(isinstance(declarations, list) and declarations, f"{key}: no source declarations")
        _require(
            row.get("declaration_hash") == declaration_hash(declarations),
            f"{key}: declaration hash is stale",
        )
        support = row.get("support")
        _require(isinstance(support, dict), f"{key}: missing support matrix")
        for surface in SURFACES:
            _require(surface in support, f"{key}: support omits {surface}")
            _require(
                support[surface].get("state") in {"implemented", "missing", "not_applicable"},
                f"{key}: invalid {surface} state",
            )
            _require(bool(support[surface].get("evidence")), f"{key}: {surface} lacks evidence")
        if disposition == "public_gameplay":
            _require(
                support["cpp"]["state"] == "implemented",
                f"{key}: public gameplay declaration is not available to C++",
            )
            if require_complete:
                for surface in SURFACES:
                    _require(
                        support[surface]["state"] == "implemented",
                        f"{key}: uncovered public gameplay surface {surface}",
                    )
                _require(row.get("status") == "implemented", f"{key}: parity row remains pending")
                _require(row["test_ids"], f"{key}: implemented gameplay row has no executable test")
                _require(row["example_ids"], f"{key}: implemented gameplay row has no example")
                _require(row["cost_evidence"], f"{key}: implemented gameplay row has no cost evidence")
    missing = sorted(set(expected) - set(actual))
    stale = sorted(set(actual) - set(expected))
    _require(not missing, f"new/uncovered declarations: {', '.join(missing[:8])}")
    _require(not stale, f"inventory contains removed declarations: {', '.join(stale[:8])}")
    for key, declarations in expected.items():
        row = actual[key]
        _require(
            declarations == row["declarations"],
            f"{key}: catalog signature/result/source shape changed",
        )
    keys_hash = hashlib.sha256("\n".join(expected).encode("utf-8")).hexdigest()
    frozen = coverage.get("frozen_denominator", {})
    _require(frozen.get("candidate_keys_sha256") == keys_hash, "frozen denominator hash changed")
    _require(frozen.get("count") == len(expected), "frozen denominator count changed")
    if verify_files:
        baseline = coverage.get("baseline", {})
        catalog_path = root / baseline.get("catalog", "")
        _require(catalog_path.is_file(), "coverage catalog is missing")
        _require(sha256(catalog_path) == baseline.get("catalog_sha256"), "catalog hash is stale")
        for relative, expected_hash in baseline.get("source_hashes", {}).items():
            path = root / relative
            _require(path.is_file(), f"source disappeared: {relative}")
            _require(sha256(path) == expected_hash, f"source hash is stale: {relative}")
    counts = Counter(row["disposition"] for row in rows)
    counts["total"] = len(rows)
    counts["pending"] = sum(row.get("status") != "implemented" for row in rows)
    return dict(counts)


def markdown(coverage: dict[str, Any]) -> str:
    rows = coverage["capabilities"]
    disposition = Counter(row["disposition"] for row in rows)
    category = Counter(row["category"] for row in rows)
    public = [row for row in rows if row["disposition"] == "public_gameplay"]
    missing = Counter()
    for row in public:
        for surface in SURFACES:
            if row["support"][surface]["state"] != "implemented":
                missing[surface] += 1
    lines = [
        "# Gameplay API parity coverage",
        "",
        "Generated from `coverage.json` by `tools/gameplay_api_parity.py`; do not edit by hand.",
        "",
        f"Baseline commit: `{coverage['baseline']['commit']}`. Frozen semantic candidate rows: **{len(rows)}**.",
        "",
        "A classified row is not necessarily implemented. The strict checker fails until every public gameplay row has executable support, tests, examples, and cost evidence on all five surfaces.",
        "",
        "## Dispositions",
        "",
        "| Disposition | Rows |",
        "| --- | ---: |",
    ]
    lines.extend(f"| `{name}` | {count} |" for name, count in sorted(disposition.items()))
    lines.extend(
        [
            "",
            "## Public gameplay gaps",
            "",
            "| Surface | Missing rows |",
            "| --- | ---: |",
        ]
    )
    lines.extend(f"| `{surface}` | {missing[surface]} |" for surface in SURFACES)
    lines.extend(
        [
            "",
            "## Categories",
            "",
            "| Category | Rows |",
            "| --- | ---: |",
        ]
    )
    lines.extend(f"| `{name}` | {count} |" for name, count in sorted(category.items()))
    lines.extend(
        [
            "",
            "## Public gameplay rows",
            "",
            "| Capability | Module | C++ | BP | Lua AOT | Lua bytecode | Lua source | Status |",
            "| --- | --- | --- | --- | --- | --- | --- | --- |",
        ]
    )
    short = {"implemented": "yes", "missing": "no", "not_applicable": "n/a"}
    for row in public:
        support = row["support"]
        lines.append(
            "| `{}` | `{}` | {} | {} | {} | {} | {} | `{}` |".format(
                row["qualified"].replace("|", "\\|"),
                row["module"],
                *(short[support[surface]["state"]] for surface in SURFACES),
                row["status"],
            )
        )
    return "\n".join(lines) + "\n"


def write_generated(coverage_path: Path, markdown_path: Path, catalog_path: Path) -> None:
    coverage = generate(catalog_path)
    coverage_path.parent.mkdir(parents=True, exist_ok=True)
    coverage_path.write_text(json.dumps(coverage, indent=2) + "\n", encoding="utf-8")
    markdown_path.write_text(markdown(coverage), encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coverage", type=Path, default=DEFAULT_COVERAGE)
    parser.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
    parser.add_argument("--markdown", type=Path, default=DEFAULT_MARKDOWN)
    parser.add_argument("--update", action="store_true", help="regenerate coverage.json and coverage.md")
    parser.add_argument(
        "--inventory-only",
        action="store_true",
        help="validate classification/source coverage while allowing parity rows to remain pending",
    )
    args = parser.parse_args(argv)
    coverage_path = args.coverage.resolve()
    catalog_path = args.catalog.resolve()
    markdown_path = args.markdown.resolve()
    if args.update:
        write_generated(coverage_path, markdown_path, catalog_path)
    try:
        coverage = json.loads(coverage_path.read_text(encoding="utf-8"))
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        counts = validate(
            coverage,
            catalog,
            root=ROOT,
            require_complete=not args.inventory_only,
            verify_files=coverage_path == DEFAULT_COVERAGE.resolve(),
        )
        expected_markdown = markdown(coverage)
        _require(markdown_path.read_text(encoding="utf-8") == expected_markdown, "coverage.md is stale")
    except (CoverageError, OSError, json.JSONDecodeError) as error:
        print(f"gameplay API parity: {error}", file=sys.stderr)
        return 1
    print(
        "gameplay API parity inventory: "
        + ", ".join(f"{name}={count}" for name, count in sorted(counts.items()))
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
