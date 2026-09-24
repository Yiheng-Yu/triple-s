from _typeshed import Incomplete
from typing import Any, final

@final
class Format:
    """
    Numeric or literal response code format.
    """

@final
class Location:
    def __repr__(self, /) -> str: ...
    @property
    def end(self, /) -> int: ...
    @end.setter
    def end(self, /, value: int) -> None: ...
    @property
    def start(self, /) -> int:
        """
        Start of a data location
        """
    @start.setter
    def start(self, /, value: int) -> None: ...

@final
class SurveyInfo:
    """
    meta information about the survey entity.
    """
    def __new__(cls, /, version: "str", name: "str | None" = None, title: "str | None" = None, date: "str | None" = None, time: "str | None" = None, origin: "str | None" = None, user: "str | None" = None, style: "str | None" = None, language_options: "list[str] | None" = None, mode_options: "list[str | None] | None" = None, standard: "str | None" = None) -> SurveyInfo:
        """
        Create survey metadata for the given Triple-S version.
        Optional text fields default to None; languages default to [], modes to [None].
        Modes accept "analysis", "interview", or None. Invalid versions or modes raise ValueError.
        """
    def copy(self, /) -> "SurveyInfo":
        """
        make a shallow copy of current instance.
        """
    @property
    def date(self, /) -> str |None:
        """
        (Optional) The date the file was created.
        """
    @date.setter
    def date(self, /, value: str |None) -> None:
        """
        Set the date.
        """
    def deepcopy(self, /) -> "SurveyInfo":
        """
        make a deep copy of current instance
        """
    @property
    def language_options(self, /) -> tuple:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
    @property
    def mode_options(self, /) -> tuple:
        """
        ** Introduced since sss 1.2 **
        list of all available `mod` options
        """
    @property
    def name(self, /) -> str |None:
        """
        (Optional) name fo the survey
        """
    @name.setter
    def name(self, /, value: str |None) -> None:
        """
        Set the name.
        """
    @property
    def origin(self, /) -> str |None:
        """
        (Optional) The originating system (program and operating system), i.e., `Windows 95`
        """
    @origin.setter
    def origin(self, /, value: str |None) -> None:
        """
        Set the origin.
        """
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
        Set the standard.
        """
    @property
    def style(self, /) -> str |None:
        """
        Return the style.
        """
    @style.setter
    def style(self, /, value: str |None) -> None:
        """
        Set the style.
        """
    @property
    def time(self, /) -> str |None:
        """
        (Optional) The time the file was created.
        """
    @time.setter
    def time(self, /, value: str |None) -> None:
        """
        Set the time.
        """
    @property
    def title(self, /) -> str |None:
        """
        Return the title.
        """
    @title.setter
    def title(self, /, value: str |None) -> None:
        """
        Set the title.
        """
    def to_dict(self, /) -> "dict[str, str]":
        """
        Return version and populated name, title, date, time, origin, user, and standard fields.
        """
    @property
    def user(self, /) -> str |None:
        """
        (Optional) Name of the user who created this file
        """
    @user.setter
    def user(self, /, value: str |None) -> None:
        """
        Set the user.
        """
    @property
    def version(self, /) -> str:
        """
        The version attribute is used to indicate the version of the Triple-S
        standard that applies to this specification.
        """
    @version.setter
    def version(self, /, value: str) -> None:
        """
        Set the version.
        """

@final
class Question:
    """
    A survey question and its response definition.
    """
    def __new__(cls, /) -> Question:
        """
        Question construction is not implemented yet.
        """
    def __repr__(self, /) -> str:
        """
        Return a string representation.
        """
    @property
    def filter(self, /) -> str |None:
        """
        Return the filter.
        """
    @filter.setter
    def filter(self, /, value: str |None) -> None:
        """
        Set the filter.
        """
    @property
    def label(self, /) -> Text:
        """
        Return the question label and its text variants.
        """
    @label.setter
    def label(self, /, label: Any) -> None:
        """
        Set the label from text or a Text instance.
        """
    @property
    def location(self, /) -> Location: ...
    @property
    def name(self, /) -> str:
        """
        Return the name.
        """
    @name.setter
    def name(self, /, value: str) -> None:
        """
        Set the name.
        """
    def to_sss_classic(self, /) -> "str":
        """
        Export fragments of `Question` instance from from SSS classic
        """

@final
class Record:
    """
    Content of a survey
    """
    def __getitem__(self, index: Any, /) -> Question:
        """
        Return a question by index or name; raise IndexError if absent.
        """
    def __len__(self, /) -> int:
        """
        Return the number of questions.
        """
    def append(self, /, question: "Question") -> "None":
        """
        Append an independent copy of a question with a unique name.
        """
    @property
    def content(self, /) -> tuple:
        """
        List of questions/ variables that the current survey instance contains
        """
    def get(self, /, question_name: "str") -> "Question | None":
        """
        Find a question by name, ignoring case; return None if absent.
        """
    @property
    def id(self, /) -> str:
        """
        One RECORD keyword appears after SURVEY (or after TITLE if present).
        It is used to introduce the definition of the variables.
        The record_id is any single character A to Z (or a to z).
        The record_id can be used in conjunction with the variable_id (see the VARIABLE keyword later) to
        generate a unique variable name on import.
        For example: RECORD A
        """
    @id.setter
    def id(self, /, value: str) -> None:
        """
        Set the id.
        """
    def to_sss_classic(self, /) -> str:
        """
        Export content back to sss classic
        """

@final
class Survey:
    """
    Survey
    More detail can be found at: https://triple-s.org/documentation/manuals/
    """
    def __copy__(self, /) -> Survey:
        """
        Return a shallow copy sharing the underlying data.
        """
    def __deepcopy__(self, /) -> "Survey":
        """
        Return an independent deep copy.
        """
    @property
    def content(self, /) -> tuple:
        """
        Return the survey questions as a tuple.
        """
    def copy(self, /) -> Survey:
        """
        returns a shallow copy of current instance
        NOTE: updating the copied `Survey` instance would also mutate the original `Survey` instance too.
        To obtain a deep copy, use `copy.deepcopy` OR `deepcopy()` method
        """
    def deepcopy(self, /) -> Survey:
        """
        returns a deep copy of current instance.
        updating the copied `Survey` instance would not affect the original `Survey` instance it copied from.
        """
    def get_question(self, /, question_name: "str") -> "Question | None":
        """
        find question via question name, case insensitive
        """
    @property
    def meta(self, /) -> SurveyInfo:
        """
        Survey metadata; changes are shared with this survey.
        """
    @property
    def name(self, /) -> str |None:
        """
        Return the name.
        """
    @name.setter
    def name(self, /, name: str |None) -> None:
        """
        Set the name.
        """
    @property
    def record(self, /) -> Record:
        """
        Survey record; changes are shared with this survey.
        """
    @property
    def title(self, /) -> str |None:
        """
        Return the title.
        """
    @title.setter
    def title(self, /, title: str |None) -> None:
        """
        Set the title.
        """
    def to_sss_classic(self, /) -> str:
        """
        Export content back to sss classic
        """

@final
class Text:
    """
    Default text with optional language and mode variants.
    """
    def __copy__(self, /) -> Text:
        """
        Return a shallow copy sharing the underlying data.
        """
    def __deepcopy__(self, /) -> Text:
        """
        Return an independent deep copy.
        """
    def __new__(cls, /, content: "str", *variants: "Variant") -> Text:
        """
        Create default text with optional Variant arguments.
        """
    def _repr_html_(self, /) -> str:
        """
        Return an HTML representation for notebooks.
        """
    def add_variant(self, /, variant: "Variant | str", mode: "str | None" = None, language: "str | None" = None) -> "None":
        """
        Add a text variant. Raises error of variant with same `mode` and `language` option already exists.
        
        Args:
            variant (Variant | str): An existing variant or the text content of a
                new variant.
            mode (str | None): The mode of the new variant. Only valid when
                `variant` is a string.
            language (str | None): The language of the new variant. Only valid when
                `variant` is a string.
        
        Examples:
            Add an existing variant:
        
                obj.add_variant(variant)
        
            Create and add a variant from text:
        
                obj.add_variant("Hello world", mode="interview", language="en")
        """
    def copy(self, /) -> Text:
        """
        Return a shallow copy sharing the underlying data.
        """
    def deepcopy(self, /) -> Text:
        """
        Return an independent deep copy.
        """
    def has_variants(self, /) -> bool:
        """
        checks if there were any text variants
        """
    def set_variant(self, /, variant: "Variant | str", mode: "str | None" = None, language: "str | None" = None) -> "None":
        """
        Will modify existing variant if they have the same `mode` and `language`
        attribute or append to the current `self.variant` if not
        """
    @property
    def value(self, /) -> str:
        """
        the default value
        """
    @value.setter
    def value(self, /, value: str) -> None:
        """
        Set the value.
        """
    @property
    def variants(self, /) -> VariantList:
        """
        variants
        """
    @variants.setter
    def variants(self, /, variants: list) -> None:
        """
        Replace the text variants with the supplied list.
        """

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
    """
    def __copy__(self, /) -> Variant:
        """
        Return a shallow copy sharing the underlying data.
        """
    def __deepcopy__(self, /) -> Variant:
        """
        Return an independent deep copy.
        """
    def __new__(cls, /, content: "str", language: "str | None" = None, mode: "str | None" = None) -> Variant:
        """
        Create a text variant; supply a language tag or mode (analysis or interview).
        """
    def __repr__(self, /) -> str:
        """
        Return a string representation.
        """
    def _repr_html_(self, /) -> str:
        """
        Return an HTML representation for notebooks.
        """
    def copy(self, /) -> Variant:
        """
        Return a shallow copy sharing the underlying data.
        """
    def deepcopy(self, /) -> Variant:
        """
        Return an independent deep copy.
        """
    @property
    def language(self, /) -> str |None:
        """
        Return the language tag, or None if unspecified.
        """
    @language.setter
    def language(self, /, language: str |None) -> None:
        """
        set `language` to `Language::Name(value)`
        """
    @property
    def mode(self, /) -> str |None:
        """
        Return analysis, interview, or None for both modes.
        """
    @mode.setter
    def mode(self, /, mode: str |None) -> None:
        """
        Set analysis, interview, or None; raise ValueError for invalid modes.
        """
    @property
    def value(self, /) -> str:
        """
        Return the variant text.
        """
    @value.setter
    def value(self, /, value: str) -> None:
        """
        Set the value.
        """

@final
class VariantList:
    """
    A mutable collection of text variants.
    """
    def __getitem__(self, index: int, /) -> Variant:
        """
        Return a variant by index; negative indices count from the end.
        """
    def __iadd__(self, other: object, /) -> None:
        """
        Extend in place from a VariantList or list of variants.
        """
    def __len__(self, /) -> int:
        """
        Return the number of variants.
        """
    def __repr__(self, /) -> str:
        """
        Return a string representation.
        """
    def append(self, /, variant: "Variant") -> "None":
        """
        Append a variant.
        """
    def extend(self, /, variants: "list[Variant]") -> "None":
        """
        Append variants; raise ValueError for duplicate mode and language pairs.
        """
    def pop(self, /) -> "Variant | None":
        """
        remove the last text variant in the list, and returns it.
        """
    def popleft(self, /) -> "Variant | None":
        """
        remove first text variant in the list, and returns it.
        """

def read_file(path: "str") -> "Survey":
    """
    Read a Triple-S file and return its survey.
    """

def __getattr__(name: str) -> Incomplete: ...
