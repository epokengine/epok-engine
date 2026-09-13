"""Expose the shared descriptor resolver to standalone integration scripts."""
from pathlib import Path
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from project_layout import project_manifest
