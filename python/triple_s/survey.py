"""High-level Python survey helpers."""

from __future__ import annotations

from typing import Any, Optional, TypeVar, List

from . import _triple_s

SurveyType = TypeVar("SurveyType", bound="Survey")


class Survey:
    """High-level wrapper around the Rust-backed survey object.

    The Rust object remains available as ``survey._survey``. Attributes not
    defined by this wrapper are delegated to it, so existing field access
    continues to work.
    """

    def __init__(
        self,
        title: str = "",
        meta: Optional[_triple_s.Meta] = None,
        record: Optional[_triple_s.Record] = None,
    ) -> None:
        self._survey = _triple_s.Survey(title, meta, record)

    @classmethod
    def from_rust(cls: type[SurveyType], survey: _triple_s.Survey) -> SurveyType:
        """Wrap an existing Rust survey without copying it."""
        instance = cls.__new__(cls)
        instance._survey = survey
        return instance  # type: ignore[return-value]

    @property
    def title(self) -> str:
        return self._survey.title

    @title.setter
    def title(self, value: str) -> None:
        self._survey.title = value

    @property
    def meta(self) -> _triple_s.Meta:
        return self._survey.meta

    @meta.setter
    def meta(self, value: _triple_s.Meta) -> None:
        self._survey.meta = value

    @property
    def record(self) -> _triple_s.Record:
        return self._survey.record
    
    @property
    def content(self) -> List[_triple_s.Choice]:
        return self._survey.record.content

    @record.setter
    def record(self, value: _triple_s.Record) -> None:
        self._survey.record = value

    def __getattr__(self, name: str) -> Any:
        return getattr(self._survey, name)

    def __repr__(self) -> str:
        return repr(self._survey)

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Survey):
            other = other._survey
        return self._survey == other
