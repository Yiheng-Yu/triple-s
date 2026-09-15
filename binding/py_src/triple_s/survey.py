from . import triple_s as rs
import pandas as pd

class Proxy:
    BACKEND = rs.Proxy
    __slots__ = ("__backend__",)
    
    def __init_subclass__(cls, **kwargs):
        """Adapts documenatation from rust on default"""
        super().__init_subclass__(**kwargs)
        if cls.__doc__ is None:
            cls.__doc__ = cls.BACKEND.__doc__
        else:
            cls.__doc__ = cls.__doc__ + "\n" + cls.BACKEND.__doc__
        
    def __init__(self, inner: BACKEND):
        if not isinstance(inner, self.BACKEND):
            raise TypeError(
                f"expected {self.BACKEND.__name__}, "
                f"got {type(inner).__name__}"
            )
        self.__backend__ = inner
            
    def __repr__(self) -> str:
        return repr(self.__backend__)


class Survey(Proxy):
    __slots__ = ("__backend__",)
    BACKEND = rs.Survey
    
    @property
    def label(self) -> str:
        return self.__backend__.label

    @label.setter
    def label(self, value: str) -> None:
        self.__backend__.label = value

    @property
    def meta(self) -> rs.Meta:
        return self.__backend__.meta

    @meta.setter
    def meta(self, value: rs.Meta) -> None:
        self.__backend__.meta = value
        
    @property
    def record(self) -> "Record":
        return Record(self.__backend__.record)
    
    @property
    def questions(self) -> "Record":
        """Alias for `record`"""
        return self.record
        
    def __getitem__(self, index: int | str) -> "Question":
        if isinstance(index, int):
            return Question(self.__backend__.record[index])
        elif isinstance(index, str):
            for question in self.__backend__.record:
                if question.name.lower() == index.lower():
                    return Question(question)
            return IndexError(index)
        else:
            return TypeError(f"Unsupported type: {type(index)}. Expect one of: [int, str]")

    def to_sss_classic(self) -> str:
        return self.__backend__.to_sss_classic()

class Record(Proxy):
    __slots__ = ("__backend__",)
    BACKEND = rs.Record
    
    def __getitem__(self, index: int|str) -> "Question":
        if isinstance(index, int):
            return Question(self.__backend__[index])
        elif isinstance(index, str):
            for question in self.__backend__:
                if question.name.lower() == index.lower():
                    return Question(question)
            return IndexError(index)
        else:
            return TypeError(f"Unsupported type: {type(index)}. Expect one of: [int, str]")

class Question(Proxy):
    __slots__ = ("__backend__",)
    BACKEND = rs.Question

    @property
    def name(self) -> str:
        return self.__backend__.name

    @name.setter
    def name(self, value: str) -> None:
        self.__backend__.name = value

    @property
    def label(self) -> str:
        return self.__backend__.label

    @label.setter
    def label(self, value: str) -> None:
        self.__backend__.label = value

    @property
    def filter(self) -> str | None:
        return self.__backend__.filter

    @filter.setter
    def filter(self, value: str) -> None:
        self.__backend__.filter = value

    @property
    def kind(self) -> rs.Kind:
        # todo: create a setter for this!
        return self.__backend__.kind

    @property
    def response(self) -> "Response":
        return Response(self.__backend__.response)
    
    @property
    def codes(self) -> "Response":
        """Alias for `Response`"""
        return self.response

    def to_sss_classic(self) -> str:
        return self.__backend__.to_sss_classic()


class Response(Proxy):
    __slots__ = ("__backend__",)
    BACKEND = rs.Response

    @property
    def item(self) -> dict[int, rs.Choice]:
        return self.__backend__.item

    def to_dict(self) -> dict[int, dict[str, str]]:
        if self.__backend__.is_categorical():
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
