from . import triple_s as rs
import pandas as pd


class Survey:
    __slot__ = ["_survey"]

    def __init__(self, inner: rs.Survey):
        if not isinstance(inner, rs.Survey):
            raise TypeError(type(inner))
        self._survey = inner

    def __repr__(self) -> str:
        return repr(self._survey)

    @property
    def label(self) -> str:
        return self._survey.label

    @label.setter
    def label(self, value: str) -> None:
        self._survey.label = value

    @property
    def meta(self) -> rs.Meta:
        return self._survey.meta

    @meta.setter
    def meta(self, value: rs.Meta) -> None:
        self._survey.meta = value

    def __getitem__(self, index: int | str) -> "Question":
        if isinstance(index, int):
            return Question(self._survey.record[index])
        elif isinstance(index, str):
            for question in self._survey.record:
                if question.name.lower() == index.lower():
                    return Question(question)
            return IndexError(index)
        else:
            return TypeError(f"Unsupported type: {type(index)}. Expect one of: [int, str]")

    def to_sss_classic(self) -> str:
        return self._survey.to_sss_classic()


class Question:
    __slot__ = ["_question"]

    def __init__(self, inner: rs.Question):
        if not isinstance(inner, rs.Question):
            raise TypeError(type(inner))
        self._question = inner

    def __repr__(self) -> str:
        return repr(self._question)

    @property
    def name(self) -> str:
        return self._question.name

    @name.setter
    def name(self, value: str) -> None:
        self._question.name = value

    @property
    def label(self) -> str:
        return self._question.label

    @label.setter
    def label(self, value: str) -> None:
        self._question.label = value

    @property
    def filter(self) -> str | None:
        return self._question.filter

    @filter.setter
    def filter(self, value: str) -> None:
        self._question.filter = value

    # @property
    def kind(self) -> rs.Kind:
        # todo: create a setter for this!
        return self._question.kind

    @property
    def response(self) -> "Response":
        return Response(self._question.response)

    def to_sss_classic(self) -> str:
        return self._question.to_sss_classic()


class Response:
    __slot__ = ["_response"]

    def __init__(self, inner: rs.Response):
        if not isinstance(inner, rs.Response):
            raise TypeError(type(inner))
        self._response = inner

    def __repr__(self) -> str:
        return repr(self._response)

    @property
    def item(self) -> dict[int, rs.Choice]:
        return self._response.item

    def to_dict(self) -> dict[int, dict[str, str]]:
        if self._response.is_categorical():
            return {choice.code: {"label": choice.label, "filter": choice.filter} for choice in self.item.values()}
        else:
            return {}

    @property
    def pandas(self) -> pd.DataFrame:
        content = self.to_dict()
        if content:
            content = pd.DataFrame.from_dict(content, "index").convert_dtypes()
            content.index.name = "code"
        else:
            content = pd.DataFrame(
                data=None, index=pd.Index(data=[], name="code"), columns=pd.Index(["label", "filter"])
            )
        return content
