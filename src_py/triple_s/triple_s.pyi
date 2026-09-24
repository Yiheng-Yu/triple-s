"""
A Python module implemented in Rust.
"""

from _typeshed import Incomplete
from collections.abc import Sequence
from typing import Final, final

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
class Meta:
    def __new__(cls, /, version: "str", name: "str | None" = None, title: "str | None" = None, date: "str | None" = None, time: "str | None" = None, origin: "str | None" = None, user: "str | None" = None, style: "str | None" = None, language_options: "list[str] | None" = None, mode_options: "list[str] | None" = None, standard: "str | None" = None) -> Meta:
        """
        Create survey metadata for the given Triple-S version.
        Optional text fields default to None; languages default to [], modes to
        [None]. Modes accept "analysis", "interview", or None. Invalid
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
    def language_options(self, /) -> list[str]:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
    @language_options.setter
    def language_options(self, /, value: Sequence[str]) -> None:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
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
class Survey: ...

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
