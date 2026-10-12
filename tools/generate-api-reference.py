"""Generate the Epok and pinned PsyQo callable API reference from C++ headers.

The checked-in Markdown is the publishable artifact. Regenerate it after public
runtime headers or the Nugget revision change, then review the prose and diff.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path


CALLABLE_KINDS = {
    "CONSTRUCTOR",
    "CONVERSION_FUNCTION",
    "CXX_METHOD",
    "DESTRUCTOR",
    "FUNCTION_DECL",
    "FUNCTION_TEMPLATE",
}

MODULE_CONTEXT = {
    "affine": "fixed-point affine transforms and matrix composition",
    "audio": "SPU sound playback and voice ownership",
    "blueprint": "compiled Blueprint execution and object interaction",
    "camera": "camera selection, projection and shared render resources",
    "collision": "bounded AABB collision queries and movement",
    "effect": "runtime effects and their deterministic playback state",
    "fixed-math": "shared Q12 integer math primitives",
    "font": "GPU text rendering with the built-in or uploaded font atlas",
    "gpu": "GPU setup, command submission and frame synchronization",
    "gte": "Geometry Transformation Engine math and register operations",
    "hud": "native HUD layout, drawing and focus navigation",
    "input": "controller sampling and simulation-tick input edges",
    "kernel": "PSX kernel ownership, interrupts and low-level services",
    "lifecycle": "entity creation, activation and destruction",
    "loading": "loading-screen rendering and transitions",
    "matrix": "fixed-point matrices and transformations",
    "memory-card": "asynchronous Memory Card access and files",
    "music": "XA music streaming and playback state",
    "ordering-table": "GPU ordering tables and packet ordering",
    "palette": "CLUT animation and palette ownership",
    "particle": "bounded particle simulation and rendering",
    "polygon": "PSX polygon preparation, clipping and submission",
    "primitive": "typed PlayStation GPU primitives",
    "resource": "bounded runtime resource registries and accounting",
    "scene": "scene lifecycle and scene-stack control",
    "shadow": "static and blob shadow rendering",
    "skeletal": "rigid skeletal animation and pose evaluation",
    "sprite": "sprites, flipbooks and screen-facing rendering",
    "spu": "SPU RAM, voices, ADSR and sound transfer",
    "streaming": "bounded CD or PC geometry-page streaming",
    "task": "cooperative asynchronous tasks and callbacks",
    "text": "native text data and HUD text components",
    "time": "fixed-step simulation time and frame timing",
    "timeline": "deterministic timelines, markers and bindings",
    "transition": "scene loading transitions and fade state",
    "trigonometry": "fixed-point trigonometric helpers",
    "vector": "fixed-point vector arithmetic",
    "visibility": "frustum tests and chunk visibility",
}

INTERNAL_EPOK_HEADERS = {
    "blueprint_playback_service.hpp",
    "blueprint_runtime.hpp",
    "loading_renderer.hpp",
    "particle_effect_runtime.hpp",
    "retained.hpp",
    "scene_service.hpp",
    "streaming_pool.hpp",
    "timeline_runtime.hpp",
    "transform_cache.hpp",
}

# Runtime headers that the engine never includes on their own. The umbrella
# translation unit reaches them the way a real build does, through the header
# that owns them, and documents them from the configuration that compiles them.
# Including them directly instead produces a translation unit the engine never
# compiles: the declarations they own become redefinitions of the owning
# header's, and Clang keeps a redefined tag as an unnamed one, so the owning
# header's types lose their names.
DEPENDENT_EPOK_HEADERS = {
    # The body sequence_service.hpp selects with EPOK_NATIVE_SEQUENCES_ONLY.
    # It redeclares that header's sequence state for native-only projects.
    "native_music_runtime.hpp",
    # Included inside namespace epok, so its members only resolve there.
    "native_music_service.hpp",
}


@dataclass
class Parameter:
    name: str
    type: str
    description: str = ""


@dataclass
class Symbol:
    name: str
    qualified: str
    signature: str
    result: str
    kind: str
    line: int
    comment: str
    brief: str
    details: str
    returns: str
    warnings: list[str]
    params: list[Parameter] = field(default_factory=list)
    is_const: bool = False
    is_static: bool = False
    is_template: bool = False
    is_virtual: bool = False
    namespace: str = ""
    owner: str = ""


@dataclass
class ApiType:
    name: str
    qualified: str
    kind: str
    line: int
    brief: str = ""
    details: str = ""
    namespace: str = ""
    owner: str = ""
    bases: list[str] = field(default_factory=list)


@dataclass
class ApiProperty:
    name: str
    qualified: str
    type: str
    kind: str
    line: int
    signature: str
    brief: str = ""
    details: str = ""
    namespace: str = ""
    owner: str = ""
    is_static: bool = False
    value: int | None = None


@dataclass
class Module:
    family: str
    source: Path
    relative: str
    slug: str
    title: str
    include: str
    source_url: str
    context: str
    stability: str
    symbols: list[Symbol]
    types: list[str]
    diagnostics: list[str]
    type_details: list[ApiType] = field(default_factory=list)
    properties: list[ApiProperty] = field(default_factory=list)


def run(*command: str, cwd: Path) -> str:
    return subprocess.check_output(command, cwd=cwd, text=True).strip()


def find_clang_binding(engine: Path, explicit: str | None) -> Path:
    candidates = [Path(explicit).resolve()] if explicit else []
    candidates.extend(
        path.parent.parent
        for path in engine.glob(".tools/**/clang/cindex.py")
        if "site-packages" not in str(path)
    )
    for candidate in candidates:
        if (candidate / "clang" / "cindex.py").is_file():
            return candidate
    try:
        import clang.cindex  # noqa: F401

        return Path()
    except ImportError as error:
        raise SystemExit(
            "libclang Python bindings were not found. Run host setup or pass "
            "--clang-python <directory containing clang/cindex.py>."
        ) from error


def find_target_compiler(engine: Path, explicit: str | None) -> Path:
    candidates = [Path(explicit).resolve()] if explicit else []
    candidates.extend(sorted(engine.glob(".tools/**/bin/mipsel-none-elf-g++")))
    located = shutil.which("mipsel-none-elf-g++")
    if located:
        candidates.append(Path(located).resolve())
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise SystemExit(
        "The pinned mipsel-none-elf compiler was not found; without it the parse "
        "has no standard headers and silently mis-reads the runtime. Run host "
        "setup or pass --target-compiler <path to mipsel-none-elf-g++>."
    )


def freestanding_arguments(compiler: Path) -> tuple[list[str], list[Path]]:
    """Give the host parse exactly the headers the target build compiles against.

    The runtime is built bare metal with `-nostdlib -ffreestanding`, so the only
    standard headers it may legitimately see are the pinned toolchain's
    freestanding C set and its header-only C++ library. Host SDK headers are a
    different ABI, and the libc++ shipped beside them needs a newer Clang than
    the pinned libclang, so they are shut out entirely. Asking the compiler for
    its own search list keeps the set exact instead of guessed.
    """
    listing = subprocess.run(
        [str(compiler), "-std=c++20", "-ffreestanding", "-x", "c++", "-E", "-v", "-"],
        input="", capture_output=True, text=True, check=True,
    ).stderr
    roots: list[Path] = []
    collecting = False
    for line in listing.splitlines():
        if line.startswith("#include <...> search starts here:"):
            collecting = True
        elif line.startswith("End of search list."):
            break
        elif collecting:
            directory = Path(line.strip())
            if directory.is_dir():
                roots.append(directory.resolve())
    if not roots:
        raise SystemExit(f"{compiler} reported no include search list to parse against.")
    arguments = ["-target", "mipsel-none-elf", "-ffreestanding", "-fno-exceptions", "-fno-rtti",
                 "-nostdinc", "-nostdinc++"]
    for directory in roots:
        arguments.extend(["-isystem", str(directory)])
    return arguments, roots


def clean_comment(raw: str | None) -> tuple[str, str, str, dict[str, str], list[str]]:
    if not raw:
        return "", "", "", {}, []
    text = re.sub(r"^\s*/\*\*?", "", raw)
    text = re.sub(r"\*/\s*$", "", text)
    lines = [re.sub(r"^\s*(?://[/!<]*|\*)\s?", "", line).rstrip() for line in text.splitlines()]
    brief_lines: list[str] = []
    detail_lines: list[str] = []
    return_lines: list[str] = []
    parameters: dict[str, str] = {}
    warnings: list[str] = []
    target = detail_lines
    param_name = ""
    for line in lines:
        stripped = line.strip()
        match = re.match(r"[@\\](brief|details?|return|returns?|param|tparam|warning|note)\b\s*(.*)", stripped)
        if match:
            tag, value = match.groups()
            if tag == "brief":
                target = brief_lines
                target.append(value)
            elif tag.startswith("detail"):
                target = detail_lines
                target.append(value)
            elif tag.startswith("return"):
                target = return_lines
                target.append(value)
            elif tag in {"warning", "note"}:
                warnings.append(value.strip())
                target = detail_lines
            elif tag in {"param", "tparam"}:
                value = re.sub(r"^\[[^]]+\]\s*", "", value)
                pieces = value.split(maxsplit=1)
                param_name = pieces[0] if pieces else "parameter"
                parameters[param_name] = pieces[1] if len(pieces) > 1 else ""
                target = []
            continue
        if stripped in {"@code", "@endcode", "\\code", "\\endcode"}:
            continue
        if param_name and target == []:
            if stripped:
                parameters[param_name] = f"{parameters[param_name]} {stripped}".strip()
            continue
        target.append(line)
    collapse = lambda values: re.sub(r"\s+", " ", " ".join(values)).strip()
    brief = collapse(brief_lines)
    details = collapse(detail_lines)
    returns = collapse(return_lines)
    if not brief:
        sentences = re.split(r"(?<=[.!?])\s+", details, maxsplit=1)
        brief = sentences[0] if sentences and sentences[0] else ""
        details = sentences[1] if len(sentences) > 1 else ""
    return brief, details, returns, parameters, [item for item in warnings if item]


def qualified_name(cursor) -> str:
    parts = [cursor.spelling]
    parent = cursor.semantic_parent
    while parent and parent.kind.name != "TRANSLATION_UNIT":
        if parent.spelling:
            parts.append(parent.spelling)
        parent = parent.semantic_parent
    return "::".join(reversed(parts))


def namespace_name(cursor) -> str:
    parts = []
    parent = cursor.semantic_parent
    while parent and parent.kind.name != "TRANSLATION_UNIT":
        if parent.kind.name == "NAMESPACE" and parent.spelling:
            parts.append(parent.spelling)
        parent = parent.semantic_parent
    return "::".join(reversed(parts))


def owning_type(cursor) -> str:
    parent = cursor.semantic_parent
    if parent and parent.kind.name in {"CLASS_DECL", "CLASS_TEMPLATE", "STRUCT_DECL", "ENUM_DECL"}:
        return qualified_name(parent)
    return ""


def has_inaccessible_owner(cursor) -> bool:
    parent = cursor.semantic_parent
    type_kinds = {"CLASS_DECL", "CLASS_TEMPLATE", "STRUCT_DECL", "ENUM_DECL"}
    while parent and parent.kind.name != "TRANSLATION_UNIT":
        if parent.kind.name in type_kinds and parent.access_specifier.name in {"PRIVATE", "PROTECTED"}:
            return True
        parent = parent.semantic_parent
    return False


def source_extent(cursor, source_text: str) -> str:
    lines = source_text.splitlines(keepends=True)
    start_line = cursor.extent.start.line - 1
    end_line = cursor.extent.end.line - 1
    selected = lines[start_line:end_line + 1]
    if not selected:
        return ""
    selected[0] = selected[0][cursor.extent.start.column - 1:]
    selected[-1] = selected[-1][:cursor.extent.end.column - 1]
    return re.sub(r"\s+", " ", "".join(selected)).strip().rstrip(";")


def source_signature(cursor, source_text: str) -> str:
    lines = source_text.splitlines(keepends=True)
    start_line = cursor.extent.start.line - 1
    end_line = cursor.extent.end.line - 1
    selected = lines[start_line:end_line + 1]
    if not selected:
        return ""
    selected[0] = selected[0][cursor.extent.start.column - 1:]
    selected[-1] = selected[-1][:cursor.extent.end.column - 1]
    snippet = "".join(selected)
    snippet = re.sub(r"/\*.*?\*/", " ", snippet, flags=re.S)
    snippet = re.sub(r"//[^\n]*", " ", snippet)
    paren_depth = 0
    bracket_depth = 0
    stop = len(snippet)
    for index, character in enumerate(snippet):
        if character == "(":
            paren_depth += 1
        elif character == ")":
            paren_depth = max(0, paren_depth - 1)
        elif character == "[":
            bracket_depth += 1
        elif character == "]":
            bracket_depth = max(0, bracket_depth - 1)
        elif character in "{;" and paren_depth == 0 and bracket_depth == 0:
            stop = index
            break
    signature = re.sub(r"\s+", " ", snippet[:stop]).strip()
    signature = re.sub(r"\s+([,)>])", r"\1", signature)
    signature = re.sub(r"([(<])\s+", r"\1", signature)
    return signature


def module_context(name: str) -> str:
    normalized = name.lower().replace("_", "-")
    for key, value in MODULE_CONTEXT.items():
        if key in normalized:
            return value
    return f"the {name.replace('-', ' ').replace('_', ' ')} module"


def parse_module(cindex, family: str, header: Path, base: Path, engine: Path, nugget_revision: str) -> Module:
    relative = header.relative_to(base).as_posix()
    source_text = header.read_text(encoding="utf-8")
    runtime = engine / "runtime"
    nugget = base.parent if family == "psyqo" else engine / "third_party" / "nugget"
    include_args = [
        "-x", "c++-header", "-std=c++20", "-fparse-all-comments",
        f"-I{engine}", f"-I{runtime}", f"-I{nugget}",
        f"-I{nugget / 'third_party' / 'EASTL' / 'include'}",
        f"-I{nugget / 'third_party' / 'EABase' / 'include' / 'Common'}",
        "-DEPOK_API_REFERENCE=1",
    ]
    unsaved = []
    missing_display = runtime / "display.hh"
    if not missing_display.exists():
        unsaved.append((str(missing_display), "#pragma once\n"))
    translation = cindex.Index.create().parse(
        str(header),
        args=include_args,
        unsaved_files=unsaved,
        options=cindex.TranslationUnit.PARSE_SKIP_FUNCTION_BODIES,
    )
    symbols: list[Symbol] = []
    types: list[str] = []
    seen: set[tuple[int, str, str]] = set()

    def walk(cursor):
        location_file = cursor.location.file
        same_file = location_file and Path(str(location_file)).resolve() == header.resolve()
        if same_file and cursor.kind.name in {"CLASS_DECL", "CLASS_TEMPLATE", "ENUM_DECL", "STRUCT_DECL", "TYPE_ALIAS_DECL"} and cursor.spelling:
            access = cursor.access_specifier.name
            if access in {"PUBLIC", "INVALID"}:
                types.append(qualified_name(cursor))
        if same_file and cursor.kind.name in CALLABLE_KINDS and cursor.spelling:
            access = cursor.access_specifier.name
            parents = []
            owner = cursor.semantic_parent
            while owner and owner.kind.name != "TRANSLATION_UNIT":
                parents.append(owner.kind.name)
                owner = owner.semantic_parent
            if access in {"PUBLIC", "INVALID"} and "LAMBDA_EXPR" not in parents:
                qname = qualified_name(cursor)
                if qname.startswith(("epok::", "psyqo::")):
                    signature = source_signature(cursor, source_text)
                    key = (cursor.location.line, qname, signature)
                    if key not in seen and signature:
                        seen.add(key)
                        brief, details, returns, parameter_docs, warnings = clean_comment(cursor.raw_comment)
                        arguments = list(cursor.get_arguments() or [])
                        if not arguments and cursor.kind.name == "FUNCTION_TEMPLATE":
                            arguments = [child for child in cursor.get_children() if child.kind.name == "PARM_DECL"]
                        parameters = []
                        for number, argument in enumerate(arguments):
                            name = argument.spelling or f"arg{number + 1}"
                            parameters.append(Parameter(name, argument.type.spelling, parameter_docs.get(name, "")))
                        result = ""
                        if cursor.kind.name not in {"CONSTRUCTOR", "DESTRUCTOR"}:
                            result = cursor.result_type.spelling
                        symbols.append(Symbol(
                            name=cursor.spelling,
                            qualified=qname,
                            signature=signature,
                            result=result,
                            kind=cursor.kind.name,
                            line=cursor.location.line,
                            comment=cursor.raw_comment or "",
                            brief=brief,
                            details=details,
                            returns=returns,
                            warnings=warnings,
                            params=parameters,
                            is_const=(cursor.kind.name == "CXX_METHOD" and cursor.is_const_method()) or bool(re.search(r"\)\s*(?:noexcept\s*)?const\b", signature)),
                            is_static=(cursor.kind.name == "CXX_METHOD" and cursor.is_static_method()) or bool(re.search(r"\bstatic\b", signature)),
                            is_template=cursor.kind.name == "FUNCTION_TEMPLATE" or "template" in signature,
                            is_virtual="virtual" in signature,
                        ))
        for child in cursor.get_children():
            walk(child)

    walk(translation.cursor)
    symbols.sort(key=lambda item: (item.qualified.lower(), item.signature, item.line))
    types = sorted(set(types), key=str.lower)
    diagnostics = [str(item) for item in translation.diagnostics if item.severity >= item.Warning]
    stem = relative.rsplit(".", 1)[0]
    slug = re.sub(r"[^a-z0-9]+", "-", stem.lower()).strip("-")
    title = stem.replace("/", " / ").replace("-", " ").replace("_", " ").title()
    if family == "epok":
        include = f'#include "{relative}"'
        source_url = f"../../../runtime/{relative}"
        stability = "Engine internal" if header.name in INTERNAL_EPOK_HEADERS else "Epok runtime API"
    else:
        include = f'#include "psyqo/{relative}"'
        source_url = f"https://github.com/pcsx-redux/nugget/blob/{nugget_revision}/psyqo/{relative}"
        stability = "PsyQo low-level API" if relative.startswith(("hardware/", "internal/")) else "Pinned PsyQo API"
    return Module(
        family=family,
        source=header,
        relative=relative,
        slug=slug,
        title=title,
        include=include,
        source_url=source_url,
        context=module_context(stem),
        stability=stability,
        symbols=symbols,
        types=types,
        diagnostics=diagnostics,
    )


def parse_family(cindex, family: str, headers: list[Path], base: Path, engine: Path, nugget: Path, nugget_revision: str, freestanding: list[str]) -> list[Module]:
    """Parse one umbrella translation unit, then assign declarations to headers."""
    runtime = engine / "runtime"
    include_args = [
        "-x", "c++", "-std=c++20", "-fparse-all-comments",
        f"-I{engine}", f"-I{runtime}", f"-I{nugget}",
        f"-I{nugget / 'third_party' / 'EASTL' / 'include'}",
        f"-I{nugget / 'third_party' / 'EABase' / 'include' / 'Common'}",
        "-DEPOK_API_REFERENCE=1",
        # Report every error. A fatal one silences the rest, which is how a
        # translation unit missing its standard headers reads as a single line.
        "-ferror-limit=0",
        *freestanding,
    ]
    umbrella = engine / f".epok-api-{family}.cpp"
    include_lines = []
    for header in headers:
        if family == "epok" and header.name in DEPENDENT_EPOK_HEADERS:
            continue
        relative = header.relative_to(base).as_posix()
        include_lines.append(f'#include "{"psyqo/" if family == "psyqo" else ""}{relative}"')
    unsaved = [(str(umbrella), "#include <array>\n" + "\n".join(include_lines) + "\n")]
    generated_headers = {
        "display.hh": """#pragma once
namespace epok {
inline constexpr int display_width=640, display_height=480;
inline constexpr bool display_interlaced=true, retained_geometry=true;
inline constexpr bool precomputed_visibility=false, streaming_geometry=false;
inline constexpr bool streaming_prefetch_enabled=true;
}
""",
        "hud-config.hh": """#pragma once
namespace epok {
inline constexpr unsigned hud_layout_budget=128, hud_rectangle_budget=256;
inline constexpr unsigned hud_text_budget=64, hud_glyph_budget=1024;
inline constexpr unsigned hud_rotated_budget=128;
}
""",
        "hud-font.hh": """#pragma once
#include <stdint.h>
namespace epok { alignas(4) inline constexpr uint16_t hud_font_pixels[4096]={}; }
""",
        # Per-project font table; Font comes from font_types.hpp through hud_core.hpp.
        "fonts.hh": """#pragma once
#include <stddef.h>
namespace epok { inline constexpr size_t font_count=0; inline constexpr const Font* font_assets=nullptr; }
""",
        "loading-image.hh": """#pragma once
namespace epok { inline constexpr LoadingImage default_loading_image{}; }
""",
        "data-config.hh": """#pragma once
#include <stddef.h>
#define EPOK_HOST_DATA 0
namespace epok {
inline constexpr size_t stream_page_count=0, stream_pool_pages=4;
inline constexpr char stream_archive_path[]="";
}
""",
        "transition-config.hh": """#pragma once
#define EPOK_TRANSITIONS 1
#define EPOK_FADE_OUT_MS 300
#define EPOK_FADE_IN_MS 300
#define EPOK_LOADING_TEXT "Now loading..."
""",
        "lua-config.hh": """#pragma once
#define EPOK_LUA_MODE 0
""",
        # Every counter on, so the reference documents the debug HUD itself
        # rather than the empty stubs a project without it compiles.
        "debug-hud.hh": """#pragma once
#define EPOK_DEBUG_FPS 1
#define EPOK_DEBUG_CPU 1
#define EPOK_DEBUG_GTE 1
#define EPOK_DEBUG_GPU 1
#define EPOK_DEBUG_SPU 1
""",
    }
    for name, contents in generated_headers.items():
        path = runtime / name
        if not path.exists():
            unsaved.append((str(path), contents))
    translation = cindex.Index.create().parse(
        str(umbrella),
        args=include_args,
        unsaved_files=unsaved,
        options=cindex.TranslationUnit.PARSE_SKIP_FUNCTION_BODIES,
    )
    resolved_headers = {header.resolve(): header for header in headers}
    source_text = {header.resolve(): header.read_text(encoding="utf-8") for header in headers}
    symbols: dict[Path, list[Symbol]] = {header.resolve(): [] for header in headers}
    types: dict[Path, list[str]] = {header.resolve(): [] for header in headers}
    type_details: dict[Path, list[ApiType]] = {header.resolve(): [] for header in headers}
    properties: dict[Path, list[ApiProperty]] = {header.resolve(): [] for header in headers}
    seen: set[tuple[Path, int, str, str]] = set()
    seen_usrs: set[str] = set()
    seen_types: set[tuple[Path, int, str]] = set()
    seen_properties: set[tuple[Path, int, str, str]] = set()

    def walk(cursor):
        location_file = cursor.location.file
        resolved = Path(str(location_file)).resolve() if location_file else None
        if cursor.kind.name != "TRANSLATION_UNIT" and resolved not in resolved_headers:
            return
        if resolved in resolved_headers and cursor.kind.name in {"CLASS_DECL", "CLASS_TEMPLATE", "ENUM_DECL", "STRUCT_DECL", "TYPE_ALIAS_DECL"} and cursor.spelling:
            if cursor.access_specifier.name in {"PUBLIC", "INVALID"}:
                qname = qualified_name(cursor)
                types[resolved].append(qname)
                type_key = (resolved, cursor.location.line, qname)
                if type_key not in seen_types:
                    seen_types.add(type_key)
                    brief, details, _, _, _ = clean_comment(cursor.raw_comment)
                    bases = [child.type.spelling for child in cursor.get_children() if child.kind.name == "CXX_BASE_SPECIFIER"]
                    type_details[resolved].append(ApiType(
                        name=cursor.spelling,
                        qualified=qname,
                        kind=cursor.kind.name,
                        line=cursor.location.line,
                        brief=brief,
                        details=details,
                        namespace=namespace_name(cursor),
                        owner=owning_type(cursor),
                        bases=bases,
                    ))
        if resolved in resolved_headers and cursor.kind.name in {"FIELD_DECL", "VAR_DECL", "ENUM_CONSTANT_DECL"} and cursor.spelling:
            parents = []
            parent = cursor.semantic_parent
            while parent and parent.kind.name != "TRANSLATION_UNIT":
                parents.append(parent.kind.name)
                parent = parent.semantic_parent
            if cursor.access_specifier.name in {"PUBLIC", "INVALID"} and not has_inaccessible_owner(cursor) and not any(kind in parents for kind in CALLABLE_KINDS | {"LAMBDA_EXPR"}):
                qname = qualified_name(cursor)
                property_key = (resolved, cursor.location.line, qname, cursor.kind.name)
                if qname.startswith(("epok::", "psyqo::")) and property_key not in seen_properties:
                    seen_properties.add(property_key)
                    brief, details, _, _, _ = clean_comment(cursor.raw_comment)
                    value = cursor.enum_value if cursor.kind.name == "ENUM_CONSTANT_DECL" else None
                    properties[resolved].append(ApiProperty(
                        name=cursor.spelling,
                        qualified=qname,
                        type=cursor.type.spelling,
                        kind=cursor.kind.name,
                        line=cursor.location.line,
                        signature=(f"{cursor.spelling} = {value}" if cursor.kind.name == "ENUM_CONSTANT_DECL" else source_extent(cursor, source_text[resolved])),
                        brief=brief,
                        details=details,
                        namespace=namespace_name(cursor),
                        owner=owning_type(cursor),
                        is_static=cursor.kind.name == "VAR_DECL" and bool(owning_type(cursor)),
                        value=value,
                    ))
        if resolved in resolved_headers and cursor.kind.name in CALLABLE_KINDS and cursor.spelling:
            parents = []
            owner = cursor.semantic_parent
            while owner and owner.kind.name != "TRANSLATION_UNIT":
                parents.append(owner.kind.name)
                owner = owner.semantic_parent
            lexical = cursor.lexical_parent
            semantic = cursor.semantic_parent
            is_friend_redeclaration = lexical and semantic and lexical != semantic
            if cursor.access_specifier.name in {"PUBLIC", "INVALID"} and not has_inaccessible_owner(cursor) and "LAMBDA_EXPR" not in parents and not is_friend_redeclaration:
                qname = qualified_name(cursor)
                if qname.startswith(("epok::", "psyqo::")):
                    signature = source_signature(cursor, source_text[resolved])
                    key = (resolved, cursor.location.line, qname, signature)
                    usr = cursor.get_usr()
                    if key not in seen and (not usr or usr not in seen_usrs) and signature:
                        seen.add(key)
                        if usr:
                            seen_usrs.add(usr)
                        brief, details, returns, parameter_docs, warnings = clean_comment(cursor.raw_comment)
                        arguments = list(cursor.get_arguments() or [])
                        if not arguments and cursor.kind.name == "FUNCTION_TEMPLATE":
                            arguments = [child for child in cursor.get_children() if child.kind.name == "PARM_DECL"]
                        parameters = []
                        for number, argument in enumerate(arguments):
                            name = argument.spelling or f"arg{number + 1}"
                            parameters.append(Parameter(name, argument.type.spelling, parameter_docs.get(name, "")))
                        result = "" if cursor.kind.name in {"CONSTRUCTOR", "DESTRUCTOR"} else cursor.result_type.spelling
                        symbols[resolved].append(Symbol(
                            name=cursor.spelling, qualified=qname, signature=signature,
                            result=result, kind=cursor.kind.name, line=cursor.location.line,
                            comment=cursor.raw_comment or "", brief=brief, details=details,
                            returns=returns, warnings=warnings, params=parameters,
                            is_const=(cursor.kind.name == "CXX_METHOD" and cursor.is_const_method()) or bool(re.search(r"\)\s*(?:noexcept\s*)?const\b", signature)),
                            is_static=(cursor.kind.name == "CXX_METHOD" and cursor.is_static_method()) or bool(re.search(r"\bstatic\b", signature)),
                            is_template=cursor.kind.name == "FUNCTION_TEMPLATE" or "template" in signature,
                            is_virtual="virtual" in signature,
                            namespace=namespace_name(cursor),
                            owner=owning_type(cursor),
                        ))
        for child in cursor.get_children():
            walk(child)

    walk(translation.cursor)
    diagnostic_map: dict[Path, list[str]] = {header.resolve(): [] for header in headers}
    global_diagnostics = []
    for diagnostic in translation.diagnostics:
        if diagnostic.severity < diagnostic.Error:
            continue
        location_file = diagnostic.location.file
        resolved = Path(str(location_file)).resolve() if location_file else None
        if resolved in diagnostic_map:
            diagnostic_map[resolved].append(str(diagnostic))
        else:
            global_diagnostics.append(str(diagnostic))
    modules = []
    for header in headers:
        resolved = header.resolve()
        relative = header.relative_to(base).as_posix()
        stem = relative.rsplit(".", 1)[0]
        slug = re.sub(r"[^a-z0-9]+", "-", stem.lower()).strip("-")
        title = stem.replace("/", " / ").replace("-", " ").replace("_", " ").title()
        if family == "epok":
            include = f'#include "{relative}"'
            source_url = f"../../../runtime/{relative}"
            stability = "Engine internal" if header.name in INTERNAL_EPOK_HEADERS else "Epok runtime API"
        else:
            include = f'#include "psyqo/{relative}"'
            source_url = f"https://github.com/pcsx-redux/nugget/blob/{nugget_revision}/psyqo/{relative}"
            stability = "PsyQo low-level API" if relative.startswith(("hardware/", "internal/")) else "Pinned PsyQo API"
        module_symbols = symbols[resolved]
        module_symbols.sort(key=lambda item: (item.qualified.lower(), item.signature, item.line))
        modules.append(Module(
            family=family, source=header, relative=relative, slug=slug, title=title,
            include=include, source_url=source_url, context=module_context(stem),
            stability=stability, symbols=module_symbols,
            types=sorted(set(types[resolved]), key=str.lower),
            diagnostics=diagnostic_map[resolved] + (global_diagnostics if header == headers[0] else []),
            type_details=sorted(type_details[resolved], key=lambda item: item.qualified.lower()),
            properties=sorted(properties[resolved], key=lambda item: (item.qualified.lower(), item.line)),
        ))
    return modules


def humanize(name: str) -> str:
    name = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", name)
    return name.replace("_", " ").replace("operator", "operator ").strip().lower()


def fallback_brief(symbol: Symbol, module: Module) -> str:
    verb = humanize(symbol.name)
    if symbol.kind == "CONSTRUCTOR":
        return f"Constructs `{symbol.qualified.rsplit('::', 1)[0]}` for {module.context}."
    if symbol.kind == "DESTRUCTOR":
        return f"Releases the resources owned by `{symbol.qualified.rsplit('::', 1)[0]}`."
    prefixes = {
        "get": "Returns", "is": "Reports whether", "has": "Reports whether",
        "set": "Sets", "add": "Adds", "remove": "Removes", "clear": "Clears",
        "reset": "Resets", "create": "Creates", "destroy": "Destroys",
        "begin": "Begins", "end": "Ends", "start": "Starts", "stop": "Stops",
        "load": "Loads", "read": "Reads", "write": "Writes", "draw": "Draws",
        "render": "Renders", "update": "Updates", "request": "Requests",
        "find": "Finds", "push": "Pushes", "pop": "Pops", "poll": "Polls",
        "play": "Starts", "pause": "Pauses", "resume": "Resumes",
    }
    for prefix, phrase in prefixes.items():
        if symbol.name.lower().startswith(prefix):
            remainder = humanize(symbol.name[len(prefix):]) or verb
            return f"{phrase} {remainder} as part of {module.context}."
    return f"Performs `{verb}` as part of {module.context}."


def parameter_role(parameter: Parameter) -> str:
    spelling = parameter.type
    if "&&" in spelling:
        return "Consumed or moved input"
    if ("*" in spelling or "&" in spelling) and "const" not in spelling:
        return "Input/output; inspect the function contract"
    if "(*)" in spelling or "function" in spelling.lower() or "callback" in parameter.name.lower():
        return "Callback"
    return "Input"


def template_arguments(signature: str) -> list[str]:
    match = re.search(r"\btemplate\s*<([^>]*)>", signature)
    if not match:
        return []
    arguments = []
    for declaration in match.group(1).split(","):
        declaration = declaration.split("=", 1)[0]
        names = re.findall(r"[A-Za-z_]\w*", declaration)
        if names:
            arguments.append(names[-1])
    return arguments


def call_example(symbol: Symbol, module: Module) -> str:
    arguments = ", ".join(parameter.name for parameter in symbol.params)
    owner = symbol.qualified.rsplit("::", 1)[0]
    lines = [module.include]
    template_args = template_arguments(symbol.signature) if symbol.is_template else []
    template_suffix = f"<{', '.join(template_args)}>" if template_args else ""
    if template_args:
        lines.extend(["", "// Replace these template arguments with types or values accepted by the declaration:", f"// {', '.join(template_args)}"])
    if symbol.params:
        lines.append("")
        lines.append("// Assume these named values have been initialized with valid data:")
        for parameter in symbol.params:
            lines.append(f"// {parameter.type} {parameter.name}")
    if symbol.kind == "CONSTRUCTOR":
        call = f"{owner} value({arguments});"
    elif symbol.kind == "DESTRUCTOR":
        call = f"// `{owner}` cleans up when its owning scope ends."
    elif "::" not in symbol.qualified.removeprefix("epok::").removeprefix("psyqo::") or symbol.kind == "FUNCTION_DECL":
        invocation = f"{symbol.qualified}{template_suffix}({arguments})"
        call = f"{invocation};" if symbol.result == "void" else f"auto result = {invocation};"
    elif symbol.is_static:
        invocation = f"{owner}::{symbol.name}{template_suffix}({arguments})"
        call = f"{invocation};" if symbol.result == "void" else f"auto result = {invocation};"
    else:
        lines.extend(["", f"{owner}& object = /* obtain a valid instance */;"])
        invocation = f"object.{symbol.name}{template_suffix}({arguments})"
        call = f"{invocation};" if symbol.result == "void" else f"auto result = {invocation};"
    lines.extend(["", call])
    return "\n".join(lines)


def recommendations(symbol: Symbol, module: Module) -> tuple[str, str]:
    signature = symbol.signature.lower()
    qualified = symbol.qualified.lower()
    benefits: list[str] = []
    cautions: list[str] = list(symbol.warnings)
    if symbol.is_const:
        benefits.append("The method is `const`, so it does not mutate the object through this API surface.")
    if symbol.is_template:
        benefits.append("Template dispatch is resolved at compile time and normally adds no runtime indirection.")
        cautions.append("Every instantiated type must satisfy the header's compile-time requirements; extra instantiations can increase code size.")
    if symbol.result == "bool":
        benefits.append("The boolean result makes success, availability or state explicit without exceptions.")
        cautions.append("Check the return value; `false` is part of normal control flow for many PSX resource operations.")
    if "fixedpoint" in signature or "epok::fixed" in signature:
        benefits.append("Fixed-point inputs keep console behavior deterministic and avoid software floating-point work.")
        cautions.append("Stay within the documented range and account for quantization before chaining several operations.")
    if any(word in qualified for word in ("callback", "task", "async", "queue", "schedule")):
        benefits.append("The asynchronous shape lets the frame loop continue while hardware or queued work completes.")
        cautions.append("Captured data and buffers must remain valid until the callback or task has completed.")
    if any(word in qualified for word in ("gpu", "primitive", "orderingtable", "draw", "render", "send")):
        benefits.append("The API maps closely to PSX GPU work, giving predictable ordering and low overhead.")
        cautions.append("Respect packet lifetime, ordering-table direction and per-frame GPU/VRAM budgets; submission is not a desktop immediate-mode draw call.")
    if any(word in qualified for word in ("cdrom", "memorycard", "spu", "audio", "music")):
        benefits.append("The API exposes the hardware service without hiding latency or bounded memory.")
        cautions.append("Treat device absence, busy state and I/O failure as expected outcomes; do not block the frame loop waiting for hardware.")
    if module.stability in {"Engine internal", "PsyQo low-level API"}:
        cautions.append(f"This is classified as **{module.stability}**. Prefer a higher-level Epok service unless you need this exact control.")
    if any("*" in parameter.type or "&" in parameter.type for parameter in symbol.params):
        cautions.append("Pointer/reference arguments are borrowed unless the source contract says otherwise; keep them valid for the complete operation and never assume null is accepted.")
    if not benefits:
        benefits.append(f"It provides direct, allocation-conscious access to {module.context}. No exception-based error path is implied by the signature.")
    if not cautions:
        cautions.append("Call it only in the lifecycle phase described by the module. Validate indices, capacities and object state before use.")
    return " ".join(dict.fromkeys(benefits)), " ".join(dict.fromkeys(cautions))


def symbol_anchor_base(symbol: Symbol) -> str:
    return re.sub(r"[^a-z0-9]+", "-", symbol.qualified.lower()).strip("-")


def symbol_anchor(symbol: Symbol, number: int) -> str:
    return f"{symbol_anchor_base(symbol)}-{number}"


def render_module(module: Module, nugget_revision: str) -> str:
    family_name = "Epok" if module.family == "epok" else "PsyQo"
    lines = [
        f"# {family_name} API: {module.title}", "",
        f"> **Header:** `{module.include.replace('#include ', '')}` · **Tier:** {module.stability} · "
        f"**Source:** [open header]({module.source_url})", "",
        f"This module covers {module.context}. It documents {len(module.symbols)} public callable"
        f"{'s' if len(module.symbols) != 1 else ''} declared directly in this header.", "",
    ]
    if module.family == "psyqo":
        lines.extend([
            f"PsyQo is pinned through Nugget revision `{nugget_revision}`. Signatures and comments below come from that exact revision, not from whichever upstream version happens to be newest.", "",
        ])
    if module.types:
        lines.extend(["## Declared types", "", ", ".join(f"`{name}`" for name in module.types), ""])
    lines.extend(["## Callable index", ""])
    if not module.symbols and module.source.name in DEPENDENT_EPOK_HEADERS:
        lines.extend(["The engine never includes this header on its own, and the configuration this reference documents does not select it. Its declarations appear under the header that includes it.", ""])
    elif not module.symbols:
        lines.extend(["This header declares no public callable symbols. It is retained in the reference because it defines types, constants or concepts used by neighboring modules.", ""])
    else:
        groups: dict[str, int] = {}
        for symbol in module.symbols:
            base = symbol_anchor_base(symbol)
            groups[base] = groups.get(base, 0) + 1
            anchor = symbol_anchor(symbol, groups[base])
            lines.append(f"- [`{symbol.qualified}`](#{anchor}) — {symbol.brief or fallback_brief(symbol, module)}")
        lines.append("")
    groups = {}
    for symbol in module.symbols:
        base = symbol_anchor_base(symbol)
        groups[base] = groups.get(base, 0) + 1
        anchor = symbol_anchor(symbol, groups[base])
        brief = symbol.brief or fallback_brief(symbol, module)
        benefits, cautions = recommendations(symbol, module)
        qualifiers = []
        if symbol.is_static:
            qualifiers.append("static")
        if symbol.is_const:
            qualifiers.append("const")
        if symbol.is_virtual:
            qualifiers.append("virtual")
        if symbol.is_template:
            qualifiers.append("template")
        lines.extend([
            f'<a id="{anchor}"></a>', "",
            f"## `{symbol.qualified}`", "",
            f"**Purpose.** {brief}", "",
        ])
        if symbol.details:
            lines.extend([f"**Details.** {symbol.details}", ""])
        lines.extend([
            "**Exact declaration**", "", "```cpp", symbol.signature, "```", "",
            f"- **Declared at:** [line {symbol.line}]({module.source_url}#L{symbol.line})",
            f"- **Kind:** `{symbol.kind.lower().replace('_', ' ')}`" + (f"; qualifiers: `{', '.join(qualifiers)}`" if qualifiers else ""),
        ])
        if symbol.params:
            lines.extend(["", "**Parameters**", "", "| Name | Type | Role | Meaning |", "| --- | --- | --- | --- |"])
            for parameter in symbol.params:
                meaning = parameter.description or f"Value supplied for `{parameter.name}`. See the exact type and module contract."
                lines.append(f"| `{parameter.name}` | `{parameter.type}` | {parameter_role(parameter)} | {meaning.replace('|', '\\|')} |")
        if symbol.kind not in {"CONSTRUCTOR", "DESTRUCTOR"}:
            return_text = symbol.returns or ("No value is returned; observe the documented state change or callback." if symbol.result == "void" else f"Returns `{symbol.result}`. Check the purpose and failure notes before using the value.")
            lines.extend(["", f"**Returns.** {return_text}"])
        lines.extend([
            "", "**Use it when.** " + (symbol.details or f"You need {module.context} and the preconditions in the declaration are already satisfied."),
            "", "**Usage pattern**", "", "```cpp", call_example(symbol, module), "```", "",
            f"**Why choose it.** {benefits}", "",
            f"**Trade-offs and warnings.** {cautions}", "",
        ])
    return "\n".join(lines).rstrip() + "\n"


def render_family_index(family: str, modules: list[Module], nugget_revision: str) -> str:
    title = "Epok runtime API" if family == "epok" else "PsyQo API"
    intro = (
        "Epok's gameplay-facing and engine-runtime callables. Start here for entities, input, collision, audio, scenes, timelines, effects and resource budgets."
        if family == "epok"
        else f"The complete callable surface found in the pinned `psyqo/` headers at Nugget `{nugget_revision}`. High-level modules come first; hardware and internal modules are clearly marked."
    )
    total = sum(len(module.symbols) for module in modules)
    lines = [f"# {title}", "", intro, "", f"**Coverage:** {len(modules)} headers · {total} public callables.", "", "## Modules", "", "| Module | Header | Callables | Tier |", "| --- | --- | ---: | --- |"]
    for module in sorted(modules, key=lambda item: ("internal" in item.stability.lower() or "low-level" in item.stability.lower(), item.title)):
        lines.append(f"| [{module.title}]({family}/{module.slug}.md) | `{module.relative}` | {len(module.symbols)} | {module.stability} |")
    lines.extend(["", "## How to read an entry", "", "Each callable records the exact declaration, source line, parameter direction, return contract, a usage pattern, reasons to choose it and warnings. Usage patterns show the call in isolation: create the named arguments with valid game data first.", ""])
    return "\n".join(lines)


def render_root_index(epok: list[Module], psyqo: list[Module], nugget_revision: str) -> str:
    epok_count = sum(len(module.symbols) for module in epok)
    psyqo_count = sum(len(module.symbols) for module in psyqo)
    type_count = len({item.qualified for module in epok + psyqo for item in module.type_details})
    property_count = len({(module.family, item.qualified, item.signature) for module in epok + psyqo for item in module.properties})
    return f"""# C++ API reference

This reference is generated from the exact C++ headers shipped with Epok. It is the symbol-by-symbol companion to the workflow guides: use those guides to learn a system, then use this section while writing code.

The [web API explorer](https://epokengine.github.io/docs/api/) presents the same snapshot like a scripting reference: browse {type_count} classes, structs, enums and aliases, {epok_count + psyqo_count} callable overloads and {property_count} public fields, constants and enum values. Every item has its own permanent page and usage snippet. The Markdown modules below remain the compact, header-oriented version for offline reading and repository reviews.

## Choose the right layer

| Layer | Start here | Public callables | Best for |
| --- | --- | ---: | --- |
| Epok runtime | [Browse Epok modules](epok.md) | {epok_count} | Normal game code, engine components and bounded runtime services |
| PsyQo | [Browse PsyQo modules](psyqo.md) | {psyqo_count} | Lower-level GPU, GTE, SPU, CD-ROM, pad, task and kernel control |

Prefer Epok when both layers solve the same problem. It preserves the editor/runtime contract and its resource accounting. Reach for PsyQo when you need hardware control Epok does not expose. That extra freedom is useful, but it also makes synchronization, packet lifetime and memory budgets your responsibility.

## Version contract

- Epok declarations come from the runtime headers in this documentation snapshot.
- PsyQo declarations come from the pinned Nugget revision `{nugget_revision}`.
- EASTL, the C standard library, OpenBIOS internals and third-party implementation helpers are outside this API reference.
- “Public callable” means a free function, constructor, destructor, conversion, function template or public method declared in the documented namespaces. Private/protected members and compiler-generated lambda call operators are excluded.

## Fast lookup

The web documentation indexes qualified names, declarations and descriptions. Search for a full name such as `epok::raycast`, a method such as `GPU::sendPrimitive`, or a concept such as “memory card callback”. Each module also begins with a compact callable index.

## Reading the warnings

PSX APIs are intentionally explicit. A pointer may refer to DMA-visible memory, a callback may complete on a later frame, a fixed-capacity container can fill, and a successful host preview is not proof of real-console timing. The warning block on every entry calls out these ownership, timing and capacity risks.
"""


def property_brief(prop: ApiProperty, module: Module) -> str:
    if prop.brief:
        return prop.brief
    if prop.kind == "ENUM_CONSTANT_DECL":
        return f"Named value `{prop.name}` in `{prop.owner}`."
    subject = f"`{prop.owner}`" if prop.owner else f"the `{prop.namespace}` namespace"
    return f"Exposes `{prop.name}` on {subject} for {module.context}."


def property_example(prop: ApiProperty, module: Module) -> str:
    lines = [module.include, ""]
    if prop.kind == "ENUM_CONSTANT_DECL" or prop.is_static or not prop.owner:
        lines.append(f"auto value = {prop.qualified};")
    else:
        lines.extend([
            f"{prop.owner}& object = /* obtain a valid instance */;",
            f"auto value = object.{prop.name};",
        ])
        if "const" not in prop.type:
            lines.extend(["", f"// When mutation is valid for this object:", f"// object.{prop.name} = replacement;"])
    return "\n".join(lines)


def source_path(module: Module) -> str:
    return f"runtime/{module.relative}" if module.family == "epok" else module.source_url


def build_catalog(modules: list[Module], nugget_revision: str) -> dict:
    module_records = []
    callable_records = []
    property_records = []
    type_candidates: dict[str, tuple[int, dict]] = {}
    for module in modules:
        common = {
            "family": module.family,
            "module": module.slug,
            "header": module.relative,
            "include": module.include,
            "source": source_path(module),
            "context": module.context,
            "stability": module.stability,
        }
        module_records.append({
            **common,
            "title": module.title,
            "callables": len(module.symbols),
            "types": len(module.type_details),
            "properties": len(module.properties),
        })
        for api_type in module.type_details:
            record = {
                **common,
                "name": api_type.name,
                "qualified": api_type.qualified,
                "kind": api_type.kind,
                "line": api_type.line,
                "brief": api_type.brief,
                "details": api_type.details,
                "namespace": api_type.namespace,
                "owner": api_type.owner,
                "bases": api_type.bases,
            }
            score = int(bool(api_type.brief)) + int(bool(api_type.details)) + len(api_type.bases) + int(api_type.kind != "CLASS_DECL")
            if api_type.qualified not in type_candidates or score > type_candidates[api_type.qualified][0]:
                type_candidates[api_type.qualified] = (score, record)
        for symbol in module.symbols:
            benefits, cautions = recommendations(symbol, module)
            callable_records.append({
                **common,
                "name": symbol.name,
                "qualified": symbol.qualified,
                "kind": symbol.kind,
                "line": symbol.line,
                "signature": symbol.signature,
                "result": symbol.result,
                "brief": symbol.brief or fallback_brief(symbol, module),
                "details": symbol.details,
                "returns": symbol.returns,
                "warnings": symbol.warnings,
                "parameters": [vars(parameter) for parameter in symbol.params],
                "namespace": symbol.namespace,
                "owner": symbol.owner,
                "static": symbol.is_static,
                "const": symbol.is_const,
                "template": symbol.is_template,
                "virtual": symbol.is_virtual,
                "example": call_example(symbol, module),
                "benefits": benefits,
                "cautions": cautions,
            })
        for prop in module.properties:
            property_records.append({
                **common,
                "name": prop.name,
                "qualified": prop.qualified,
                "kind": prop.kind,
                "line": prop.line,
                "signature": prop.signature,
                "type": prop.type,
                "brief": property_brief(prop, module),
                "details": prop.details,
                "namespace": prop.namespace,
                "owner": prop.owner,
                "static": prop.is_static,
                "value": prop.value,
                "example": property_example(prop, module),
            })
    types = [record for _, record in type_candidates.values()]
    types.sort(key=lambda item: (item["family"], item["qualified"].lower()))
    callable_records.sort(key=lambda item: (item["family"], item["qualified"].lower(), item["signature"]))
    unique_properties = {}
    for prop in property_records:
        unique_properties[(prop["family"], prop["qualified"], prop["signature"])] = prop
    properties = sorted(unique_properties.values(), key=lambda item: (item["family"], item["qualified"].lower()))
    return {
        "schema": 1,
        "nugget_revision": nugget_revision,
        "modules": module_records,
        "types": types,
        "callables": callable_records,
        "properties": properties,
        "totals": {
            "types": len(types),
            "callables": len(callable_records),
            "properties": len(properties),
        },
    }


def path_scrubber(engine: Path, nugget: Path, toolchain: list[Path]) -> Callable[[str], str]:
    """Build the rewrite that keeps generated text independent of the checkout.

    Clang spells unnamed declarations and diagnostics with the absolute path of
    the file they came from, so the raw text carries whichever directory the
    generator happened to run in. Nugget is rewritten first and to its committed
    location, so a pinned checkout supplied through --psyqo-root reads the same
    as one initialized in place. The target toolchain's header directories are
    wherever host setup installed them, so they collapse to a fixed label.
    """
    prefixes = (
        *((f"{root.as_posix()}/", "<target-toolchain>/") for root in toolchain),
        (f"{nugget.as_posix()}/", "third_party/nugget/"),
        (f"{engine.as_posix()}/", ""),
    )

    def scrub(text: str) -> str:
        for absolute, relative in prefixes:
            text = text.replace(absolute, relative)
        return text

    return scrub


def write_if_changed(path: Path, content: str, scrub: Callable[[str], str]) -> None:
    content = scrub(content)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text(encoding="utf-8") != content:
        path.write_text(content, encoding="utf-8", newline="\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--engine", default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--psyqo-root", help="Override the pinned checkout's psyqo directory")
    parser.add_argument("--clang-python", help="Directory containing clang/cindex.py")
    parser.add_argument("--target-compiler", help="Path to the pinned mipsel-none-elf-g++")
    parser.add_argument("--check", action="store_true", help="Fail if regeneration changes committed output")
    args = parser.parse_args()
    engine = Path(args.engine).resolve()
    psyqo_root = Path(args.psyqo_root).resolve() if args.psyqo_root else engine / "third_party" / "nugget" / "psyqo"
    if not psyqo_root.is_dir():
        raise SystemExit("Pinned PsyQo checkout is missing. Initialize third_party/nugget or pass --psyqo-root.")
    binding = find_clang_binding(engine, args.clang_python)
    if binding:
        sys.path.insert(0, str(binding))
    from clang import cindex

    freestanding, toolchain = freestanding_arguments(find_target_compiler(engine, args.target_compiler))
    nugget_revision = run("git", "ls-tree", "HEAD", "third_party/nugget", cwd=engine).split()[2]
    # Load the shared library before parsing the two umbrella translation units.
    cindex.Index.create()
    epok_headers = sorted((engine / "runtime").glob("*.hpp"))
    psyqo_headers = sorted(path for path in psyqo_root.rglob("*.hh") if "examples" not in path.parts)
    nugget = psyqo_root.parent
    scrub = path_scrubber(engine, nugget, toolchain)
    epok_modules = parse_family(cindex, "epok", epok_headers, engine / "runtime", engine, nugget, nugget_revision, freestanding)
    psyqo_modules = parse_family(cindex, "psyqo", psyqo_headers, psyqo_root, engine, nugget, nugget_revision, freestanding)
    output = engine / "docs" / "api"
    temporary = engine / ".epok-api-reference.tmp" if args.check else output
    if temporary.exists():
        shutil.rmtree(temporary)
    temporary.mkdir(parents=True)
    write_if_changed(temporary / "index.md", render_root_index(epok_modules, psyqo_modules, nugget_revision), scrub)
    write_if_changed(temporary / "epok.md", render_family_index("epok", epok_modules, nugget_revision), scrub)
    write_if_changed(temporary / "psyqo.md", render_family_index("psyqo", psyqo_modules, nugget_revision), scrub)
    for module in epok_modules + psyqo_modules:
        write_if_changed(temporary / module.family / f"{module.slug}.md", render_module(module, nugget_revision), scrub)
    coverage = {
        "nugget_revision": nugget_revision,
        "epok": {
            "headers": len(epok_modules),
            "types": len({item.qualified for module in epok_modules for item in module.type_details}),
            "callables": sum(len(module.symbols) for module in epok_modules),
            "properties": sum(len(module.properties) for module in epok_modules),
        },
        "psyqo": {
            "headers": len(psyqo_modules),
            "types": len({item.qualified for module in psyqo_modules for item in module.type_details}),
            "callables": sum(len(module.symbols) for module in psyqo_modules),
            "properties": sum(len(module.properties) for module in psyqo_modules),
        },
        "modules": [
            {
                "family": module.family,
                "header": module.relative,
                "slug": module.slug,
                "callables": len(module.symbols),
                "types": len(module.types),
                "properties": len(module.properties),
                "source_sha256": hashlib.sha256(module.source.read_bytes()).hexdigest(),
                "diagnostics": module.diagnostics,
            }
            for module in epok_modules + psyqo_modules
        ],
    }
    write_if_changed(temporary / "coverage.json", json.dumps(coverage, indent=2) + "\n", scrub)
    catalog = build_catalog(epok_modules + psyqo_modules, nugget_revision)
    write_if_changed(temporary / "catalog.json", json.dumps(catalog, indent=2) + "\n", scrub)
    if args.check:
        existing = sorted(path.relative_to(output) for path in output.rglob("*") if path.is_file()) if output.exists() else []
        generated = sorted(path.relative_to(temporary) for path in temporary.rglob("*") if path.is_file())
        mismatch = existing != generated or any((output / path).read_bytes() != (temporary / path).read_bytes() for path in generated if (output / path).is_file())
        shutil.rmtree(temporary)
        if mismatch:
            print("API reference is stale; regenerate tools/generate-api-reference.py", file=sys.stderr)
            return 1
    print(f"Generated {len(epok_modules)} Epok headers/{coverage['epok']['callables']} callables and {len(psyqo_modules)} PsyQo headers/{coverage['psyqo']['callables']} callables.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
