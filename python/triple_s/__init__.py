"""Python interface for the Triple-S survey reader."""

from ._triple_s import (
    Choice,
    Location,
    Meta,
    Record,
    Spread,
)
from ._triple_s import read_sss as _read_sss
from .survey import Question, Response, Survey


def read_sss(path: str) -> Survey:
    """Read a Triple-S file and return a Python ``Survey`` wrapper."""
    return Survey._from_rust(_read_sss(path))


__all__ = [
    "Choice",
    "Location",
    "Meta",
    "Question",
    "Record",
    "Response",
    "Spread",
    "Survey",
    "read_sss",
]
