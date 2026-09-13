"""Typed Python interface for the Rust-backed Triple-S model."""

from collections.abc import Mapping, Sequence
from typing import Any


class Location:
    """One-based inclusive character positions in a fixed-width record."""

    # The Rust extension exposes this field as ``from``; ``from_`` is the
    # identifier-safe spelling used by type checkers.
    from_: int
    to: int

    def __new__(cls, from_: int, to: int) -> Location: ...
    def __getattr__(self, name: str) -> Any: ...
    def __repr__(self) -> str: ...


class Spread:
    """Subfield count and width for spread-format MULTIPLE responses."""

    num_subfields: int
    width: int

    def __new__(cls, num_subfields: int, width: int) -> Spread: ...
    def __repr__(self) -> str: ...


class Choice:
    """A coded category value and its optional label and SPECIAL marker."""

    code: int
    label: str
    filter: str | None

    def __new__(
        cls, code: int, *, label: str = "", filter: str | None = None
    ) -> Choice: ...
    def __repr__(self) -> str: ...


class Response:
    """Type-specific definition for a survey variable."""

    @staticmethod
    def Quantity(min: float, max: float) -> Response: ...
    @staticmethod
    def Character(length: int) -> Response: ...
    @staticmethod
    def Note(note: str) -> Response: ...
    @staticmethod
    def Single(item: Mapping[int, Choice]) -> Response: ...
    @staticmethod
    def Multiple(item: Mapping[int, Choice], spread: Spread) -> Response: ...
    @property
    def choices(self) -> list[Choice]: ...
    def __repr__(self) -> str: ...


class Question:
    """A survey variable definition."""

    name: str
    text: str
    filter: str | None
    location: Location
    response: Response

    def __new__(
        cls,
        name: str,
        *,
        location: Location | None = None,
        response: Response | None = None,
        text: str = "",
        filter: str | None = None,
    ) -> Question: ...
    def __repr__(self) -> str: ...


class Record:
    """A respondent record definition and its ordered variables."""

    id: str
    content: list[Question]

    def __new__(cls, content: Sequence[Question], *, id: str = "") -> Record: ...
    def __repr__(self) -> str: ...


class Meta:
    """Optional metadata describing a Triple-S definition file."""

    version: str | None
    date: str | None
    time: str | None
    origin: str | None
    user: str | None
    style: str | None
    note: str | None
    standard: str | None

    def __new__(
        cls,
        *,
        version: str | None = None,
        date: str | None = None,
        time: str | None = None,
        origin: str | None = None,
        user: str | None = None,
        style: str | None = None,
        note: str | None = None,
        standard: str | None = None,
    ) -> Meta: ...
    def __repr__(self) -> str: ...


class Survey:
    """Survey. More details can be found on: https://triple-s.org/documentation/manuals/"""

    title: str
    meta: Meta
    record: Record

    def __new__(
        cls,
        title: str,
        meta: Meta | None = None,
        record: Record | None = None,
    ) -> Survey: ...
    def __getitem__(self, index: int | str) -> Any: ...
    def __repr__(self) -> str: ...


def read_sss(sss_path: str) -> Survey: ...
