from typing import Final, final

__all__ = ["DataFrameFormat", "ResponseDisplay"]

@final
class DataFrameFormat:
    """
    Specifies the default DataFrame library used for Python output, default `Polars`.

    This can be overriden by individal functional calls.
    """
    Pandas: Final[DataFrameFormat]
    """
    Pandas dataframe
    """
    Polars: Final[DataFrameFormat]
    """
    Polars dataframe
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class ResponseDisplay:
    """
    Controls whether choice responses are represented by their values or labels.
    """
    Label: Final[ResponseDisplay]
    """Display choice text."""
    Value: Final[ResponseDisplay]
    """Display code values."""
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...
