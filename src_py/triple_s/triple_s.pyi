"""
A Python module implemented in Rust.
"""

from _typeshed import Incomplete
from collections.abc import Sequence
from typing import Any, Final, final

@final
class CodeFormat:
    Literal: Final[CodeFormat]
    Numeric: Final[CodeFormat]
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class DataSource:
    Csv: Final[DataSource]
    """
    Supported from sss 2.0 onwards.
    the data representation is comma separated values, using one field
    for each variable, with data values similar to the fixed format. Note
    that for data in csv format the position element refers to the
    field number.
    """
    Fixed: Final[DataSource]
    """
    The data representation is fixed format fields.
    Basically `.asc` text file that formatted bit like csv but without any
    column header.
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class Location:
    """
    Defines the part of the data record that is allocated to holding the value
    of the variable.
    """
    def __new__(cls, /, from: "int", to: "int" = None) -> Location: ...
    @property
    def from(self, /) -> int: ...
    @from.setter
    def from(self, /, from: int) -> None: ...
    @property
    def to(self, /) -> int: ...
    @to.setter
    def to(self, /, to: int) -> None: ...

@final
class Mode:
    """
    Introduced in SSS Xml 2.0.
    
    Official documentation:
    
    ```text
    (Optional) The use of specialised texts for interviewing and/or analysis within the
    specification must be signalled by a modes="mode_identifier_list"
    attribute on the initial <sss> element.
    Two explicit modes are available: “interview” and “analysis”. In the absence of
    a mode specification, the appropriate text is assumed to be used in both
    modes.
    ```
    
    Variants:
    - None: default value when no 'mode' value was defined.
    - Analysis: mode 'analysis'
    - Interview: mode 'interview'
    """
    Analysis: Final[Mode]
    Interview: Final[Mode]
    None: Final[Mode]
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class Question:
    def __new__(cls, /, name: "str", label: "str" = None, filter: "str" = None, format: "'numeric'|'literal'|None" = None, usage: "'serial'|'weight'|None" = None, location: Location |None = None) -> Question: ...
    def __repr__(self, /) -> str: ...
    @property
    def filter(self, /) -> str |None: ...
    @filter.setter
    def filter(self, /, filter: str |None) -> None: ...
    @property
    def label(self, /) -> Text |None: ...
    @label.setter
    def label(self, /, label: Any |None) -> None: ...

@final
class Survey: ...

@final
class SurveyInfo:
    def __new__(cls, /, version: "str | float", name: "str | None" = None, title: "str | None" = None, date: "str | None" = None, time: "str | None" = None, origin: "str | None" = None, user: "str | None" = None, style: "str | None" = None, language_options: "list[str] | None" = None, mode_options: "list[Mode | str | None] | None" = None, standard: "str | None" = None) -> SurveyInfo:
        """
        Create survey metadata for a Triple-S version given as a string or float.
        Optional text fields default to None; languages and modes default to
        empty tuples. Modes accept "analysis", "interview", or None. Invalid
        versions or modes raise ValueError.
        """
    @property
    def date(self, /) -> str |None:
        """
        date of the creation
        """
    @date.setter
    def date(self, /, value: str |None) -> None:
        """
        date of the creation
        """
    @property
    def language_options(self, /) -> tuple: ...
    @property
    def language_options(self, /) -> list[str]:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
    @language_options.setter
    def language_options(self, /, values: Sequence[str]) -> None: ...
    @language_options.setter
    def language_options(self, /, value: Sequence[str]) -> None:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
    @property
    def mode_options(self, /) -> tuple: ...
    @property
    def mode_options(self, /) -> list[Mode]:
        """
        ** Introduced since sss 1.2 **
        list of all available `mode` options
        """
    @mode_options.setter
    def mode_options(self, /, value: Sequence[Mode]) -> None:
        """
        ** Introduced since sss 1.2 **
        list of all available `mode` options
        """
    @mode_options.setter
    def mode_options(self, /, values: Sequence[Any]) -> None: ...
    @property
    def name(self, /) -> str |None:
        """
        name of the survey
        """
    @name.setter
    def name(self, /, value: str |None) -> None:
        """
        name of the survey
        """
    @property
    def origin(self, /) -> str |None: ...
    @origin.setter
    def origin(self, /, value: str |None) -> None: ...
    @property
    def standard(self, /) -> str |None:
        """
        ** Deprecated after sss version 1.2 **
        (Optional) indicate that some aspect of the triple-s definition meets a
        defined standard that may reduce the amount of checking required by an
        import program. The only value that option_setting may have in
        version 1.2 is 'standardnames'.
        """
    @standard.setter
    def standard(self, /, value: str |None) -> None:
        """
        ** Deprecated after sss version 1.2 **
        (Optional) indicate that some aspect of the triple-s definition meets a
        defined standard that may reduce the amount of checking required by an
        import program. The only value that option_setting may have in
        version 1.2 is 'standardnames'.
        """
    @property
    def style(self, /) -> str |None:
        """
        html css styling
        """
    @style.setter
    def style(self, /, value: str |None) -> None:
        """
        html css styling
        """
    @property
    def time(self, /) -> str |None:
        """
        time of the creation
        """
    @time.setter
    def time(self, /, value: str |None) -> None:
        """
        time of the creation
        """
    @property
    def title(self, /) -> str |None:
        """
        title of the survey
        """
    @title.setter
    def title(self, /, value: str |None) -> None:
        """
        title of the survey
        """
    def to_dict(self, /) -> "dict[str, str]": ...
    @property
    def user(self, /) -> str |None:
        """
        user who created it
        """
    @user.setter
    def user(self, /, value: str |None) -> None:
        """
        user who created it
        """
    @property
    def version(self, /) -> Version:
        """
        The version attribute is used to indicate the version of the Triple-S
        standard that applies to this specification.
        """
    @version.setter
    def version(self, /, value: Version) -> None:
        """
        The version attribute is used to indicate the version of the Triple-S
        standard that applies to this specification.
        """

@final
class Text:
    def __copy__(self, /) -> Text: ...
    def __deepcopy__(self, /) -> Text: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, value: str, variants: Sequence[Variant] |None = None) -> Text: ...
    def __repr__(self, /) -> str: ...
    def add_variant(self, /, variant: Any, mode: str |None = None, language: str |None = None) -> None: ...
    def remove_variant(self, /, index: int) -> Variant: ...
    @property
    def value(self, /) -> str: ...
    @value.setter
    def value(self, /, value: str) -> None: ...
    @property
    def variants(self, /) -> tuple:
        """
        Immutable membership snapshot, containing shared mutable variants.
        """
    @variants.setter
    def variants(self, /, variants: Sequence[Variant]) -> None: ...

@final
class Usage:
    """
    describes the role of this variable in the survey.
    Only a subset of variable types may have a use attribute (as detailed below)
    """
    Null: Final[Usage]
    """
    plain data
    """
    Serial: Final[Usage]
    """
    this variable contains the serial number (or other identification field)
    """
    Weight: Final[Usage]
    """
    this variable contains the weighting
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class Variant:
    """
    From sss xml 2.0 onwards supports multi-language
    
    For sss xml 2.0:
    ```text
    The text for a survey <title>, variable <label> or values
    <value> element may contain any number of specialised text elements.
    ```
    
    Example sss xml 2.0 text, note that one entry can contain both `label` and
    `language` attribute
    ```xml
    <label>Age
        <text xml:lang="en-GB" mode="interview">How old are you?</text>
        <text xml:lang="fr" mode="interview">Quel est votre âge?</text>
        <text xml:lang="en-GB" mode="analysis">Age of respondent</text>
        <text xml:lang="fr" mode="analysis">Âge de répondant</text>
    </label>
    ```
    `clone()` shares fields; `deepcopy()` creates independent fields.
    """
    def __copy__(self, /) -> Variant:
        """
        makes deepcopy of the current instance.
        """
    def __deepcopy__(self, /, _memo: Any) -> Variant:
        """
        makes deepcopy of the current instance.
        """
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self, /) -> int: ...
    def __new__(cls, /, value: "str", mode: "'analysis'|'interview'|None" = None, language: "str|None" = None) -> Variant: ...
    def __repr__(self, /) -> str: ...
    def copy(self, /) -> Variant:
        """
        makes deepcopy of the current instance.
        """
    @property
    def language(self, /) -> str |None: ...
    @language.setter
    def language(self, /, language: str |None) -> None: ...
    @property
    def mode(self, /) -> str |None: ...
    @mode.setter
    def mode(self, /, mode: str |None) -> None: ...
    @property
    def value(self, /) -> str: ...
    @value.setter
    def value(self, /, value: str) -> None: ...

@final
class Version:
    V1_0: Final[Version]
    V1_1: Final[Version]
    V1_2: Final[Version]
    V2_0: Final[Version]
    V3_0: Final[Version]
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

def __getattr__(name: str) -> Incomplete: ...
