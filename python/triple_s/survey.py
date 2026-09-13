"""High-level Python survey helpers."""

from __future__ import annotations

from collections.abc import Iterator, Mapping
from typing import Any, Callable, TypeVar
import pandas as pd

from . import _triple_s

SurveyType = TypeVar("SurveyType", bound="Survey")


def _unwrap_response(
    response: Response | _triple_s.Response | None,
) -> _triple_s.Response | None:
    """Return the extension value behind a Python response wrapper."""
    return response._as_rust() if isinstance(response, Response) else response


class Response:
    """Python extension point around a Rust-backed response definition."""

    def __init__(
        self,
        response: _triple_s.Response,
        snapshot: Callable[[], _triple_s.Response] | None = None,
    ) -> None:
        self._response = response
        self._snapshot = snapshot

    @classmethod
    def _from_rust(
        cls,
        response: _triple_s.Response | Any,
        snapshot: Callable[[], _triple_s.Response] | None = None,
    ) -> "Response":
        instance = cls.__new__(cls)
        instance._response = response
        instance._snapshot = snapshot
        return instance

    @staticmethod
    def Quantity(min: float, max: float) -> "Response":
        return Response._from_rust(_triple_s.Response.Quantity(min, max))

    @staticmethod
    def Character(length: int) -> "Response":
        return Response._from_rust(_triple_s.Response.Character(length))

    @staticmethod
    def Note(note: str) -> "Response":
        return Response._from_rust(_triple_s.Response.Note(note))

    @staticmethod
    def Single(item: Mapping[int, _triple_s.Choice]) -> "Response":
        return Response._from_rust(_triple_s.Response.Single(item))

    @staticmethod
    def Multiple(
        item: Mapping[int, _triple_s.Choice], spread: _triple_s.Spread
    ) -> "Response":
        return Response._from_rust(_triple_s.Response.Multiple(item, spread))

    @property
    def choices(self) -> list[_triple_s.Choice]:
        response = self._snapshot() if self._snapshot is not None else self._response
        item = getattr(response, "item", None)
        return list(item.values()) if item is not None else []

    @property
    def responses(self) -> Iterator[_triple_s.Choice]:
        return iter(self)

    def __iter__(self) -> Iterator[_triple_s.Choice]:
        yield from self.choices

    @property
    def pandas(self) -> pd.DataFrame:
        content = [
            dict(code=choice.code, label=choice.label, filter=choice.filter)
            for choice in self.choices
        ]
        return pd.DataFrame(content, columns=["code", "label", "filter"]).set_index(
            "code"
        )
    def __getattr__(self, name: str) -> Any:
        return getattr(self._response, name)

    def __repr__(self) -> str:
        return repr(self._response)

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Response):
            other = other._response
        return self._response == other

    def _as_rust(self) -> _triple_s.Response:
        return self._response


class Question:
    """Python extension point around a Rust-backed question or live proxy."""

    def __init__(
        self,
        name: str | _triple_s.Question | Any,
        *,
        location: _triple_s.Location | None = None,
        response: Response | _triple_s.Response | None = None,
        text: str = "",
        filter: str | None = None,
        response_snapshot: Callable[[], _triple_s.Response] | None = None,
    ) -> None:
        self._response_snapshot = response_snapshot
        if isinstance(name, str):
            self._question = _triple_s.Question(
                name,
                location=location,
                response=_unwrap_response(response),
                text=text,
                filter=filter,
            )
        else:
            self._question = name

    @classmethod
    def _from_rust(
        cls,
        question: _triple_s.Question | Any,
        response_snapshot: Callable[[], _triple_s.Response] | None = None,
    ) -> "Question":
        instance = cls.__new__(cls)
        instance._question = question
        instance._response_snapshot = response_snapshot
        return instance

    @property
    def response(self) -> Response:
        return Response._from_rust(
            self._question.response, snapshot=self._response_snapshot
        )
    
    @response.setter
    def response(self, value: Response | _triple_s.Response) -> None:
        self._question.response = _unwrap_response(value)
        
    @property
    def codes(self)-> Response:
        return self.response
    
    @codes.setter
    def codes(self, value: Response | _triple_s.Response) -> None:
        self.response = value

    @property
    def responses(self) -> Response:
        return self.response

    @responses.setter
    def responses(self, value: Response | _triple_s.Response) -> None:
        self.response = value

    def __getattr__(self, name: str) -> Any:
        return getattr(self._question, name)

    def __repr__(self) -> str:
        return repr(self._question)

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Question):
            other = other._question
        return self._question == other

    def _as_rust(self) -> _triple_s.Question:
        return self._question


class Survey:
    """A Triple-S survey.
    """

    def __init__(
        self,
        title: str = "",
        meta: _triple_s.Meta | None = None,
        record: _triple_s.Record | None = None,
    ) -> None:
        self._survey = _triple_s.Survey(title, meta, record)

    @classmethod
    def _from_rust(cls: type[SurveyType], survey: _triple_s.Survey) -> SurveyType:
        """Wrap an existing Rust survey without copying its data."""
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
    def content(self) -> list[Question]:
        return [
            self._question_at(index)
            for index in range(len(self._survey.record.content))
        ]

    @property
    def questions(self) -> list[Question]:
        """Return the survey's variables in definition-file order."""
        return self.content

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

    def __getitem__(self, index: int | str) -> Question:
        """Return a live question view by name or zero-based position."""
        question = self._survey[index]
        position = next(
            index
            for index, candidate in enumerate(self._survey.record.content)
            if candidate.name.upper() == question.name.upper()
        )
        return Question._from_rust(
            question,
            response_snapshot=lambda: self._survey.record.content[position].response,
        )

    def _question_at(self, index: int) -> Question:
        return Question._from_rust(
            self._survey[index],
            response_snapshot=lambda: self._survey.record.content[index].response,
        )

    def __len__(self) -> int:
        """Return the number of variables in the survey."""
        return len(self._survey.record.content)

    def __iter__(self) -> Iterator[Question]:
        """Iterate over live question views in definition-file order."""
        for index in range(len(self)):
            yield self[index]
