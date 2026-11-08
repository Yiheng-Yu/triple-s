from typing import Final, final

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
    
    Notes:
        Members:
    
        ``Value``:
            display code value (the `code` attribute of a `<value>` tag).
            i.e., [1, 1, 2, ...]
    
        ``Label``:
            display choice text (content of a `<value>` tag).
            i.e., ["London", "London", "Scotland", ...]
    """
    Label: Final[ResponseDisplay]
    """
    Display choice text (content of a `<value>` tag), i.e. ["London", "London", "Scotland", ...].
    """
    Value: Final[ResponseDisplay]
    """
    Display code value (the `code` attribute of a `<value>` tag), i.e. [1, 1, 2, ...].
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...
