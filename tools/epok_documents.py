"""Shared YAML document IO for Epok fixtures and developer tools.

Install tools/requirements.txt. JSON remains the format for external APIs,
reports explicitly requested as JSON, and metadata embedded in asset packages.
"""
import json
import re
import sys
from pathlib import Path
try:
    import yaml
except ModuleNotFoundError as error:  # Actionable next step instead of a bare traceback.
    raise SystemExit(
        f"{error}. Epok's developer tools need the packages in tools/requirements.txt.\n"
        f"Install them for this interpreter: {sys.executable} -m pip install -r tools/requirements.txt\n"
        "If that interpreter is externally managed, create a virtual environment first:\n"
        "  python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt"
    ) from error


class DocumentLoader(yaml.SafeLoader):
    """Match YAML 1.2 booleans and leave dates as ordinary schema strings."""


DocumentLoader.yaml_implicit_resolvers = {
    key: [(tag, pattern) for tag, pattern in rules
          if tag not in {'tag:yaml.org,2002:bool', 'tag:yaml.org,2002:timestamp'}]
    for key, rules in yaml.SafeLoader.yaml_implicit_resolvers.items()
}
DocumentLoader.add_implicit_resolver('tag:yaml.org,2002:bool',
    re.compile(r'^(?:true|True|TRUE|false|False|FALSE)$'), list('tTfF'))

EXTENSIONS = frozenset({
    '.epokproject', '.epokmap', '.epokbp', '.epokscript', '.epoksettings',
    '.epokconfig', '.epokprefs', '.epokcache', '.epokrequest',
    '.epokmanifest', '.epokdebug',
})


def loads(text):
    """Read YAML documents; JSON reports are also valid input."""
    return yaml.load(text, Loader=DocumentLoader)


def dumps(value):
    return yaml.safe_dump(value, allow_unicode=True, sort_keys=True)


def write_text(path, text, encoding='utf-8', errors=None, newline=None):
    """Write actual YAML for Epok documents, preserving source and report text.

    Fixture writers often construct JSON text for both protocols and disk. Only
    Epok document destinations are converted here. Invalid fixture input remains
    invalid so error-handling tests can still exercise the editor's diagnostics.
    """
    path = Path(path)
    if path.suffix.lower() in EXTENSIONS:
        try:
            value = json.loads(text)
        except json.JSONDecodeError:
            pass
        else:
            text = dumps(value)
    return path.write_text(text, encoding=encoding, errors=errors, newline=newline)
