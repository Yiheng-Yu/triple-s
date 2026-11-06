from _typeshed import Incomplete
from collections.abc import Sequence
from typing import Any, Final, final

@final
class Code:
    """
    Corresponds to the `<value>` tag of a triple-s xml.
    """
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, value: Any, label: Any | None, filter: Any | None, score: Any | None) -> Code: ...
    def __repr__(self, /) -> str: ...
    @property
    def filter(self, /) -> str | None:
        """
        response filter
        """
    @filter.setter
    def filter(self, /, filter: str | None) -> None: ...
    @staticmethod
    def float(value: "float", label: "Option[str]" = None, filter: "Option[str]" = None, score: "Option[str]" = None) -> Code: ...
    @staticmethod
    def from_dict(content: dict) -> Code:
        """
        creates new instance from dictionary with the following keys:

        'value' (float|int|str):
            code value of the new instance

        'label' (str|Text|dict|None, *optional*):
            code label of the new instance.
            default Text("")

        'filter' (str|None, *optional*):
            response filter, default 'None'

        'score' (float|int|None. *optional*):
            score
        """
    @property
    def label(self, /) -> Text:
        """
        text label of a response option for a question.
        """
    @label.setter
    def label(self, /, label: Any) -> None: ...
    @property
    def score(self, /) -> float | None:
        """
        Values to be used when computing statistics such as Mean, Standard
        Deviation etc.
        """
    @score.setter
    def score(self, /, score: float | None) -> None: ...
    @staticmethod
    def signed(value: "float", label: "Option[str]" = None, filter: "Option[str]" = None, score: "Option[str]" = None) -> Code: ...
    @staticmethod
    def string(value: "str", label: "Option[str]" = None, filter: "Option[str]" = None, score: "Option[str]" = None) -> Code:
        """
        Used for questions of type 'Single/ Multiple/ Date/ Time'
        """
    @staticmethod
    def unsigned(value: "int", label: "Option[str]" = None, filter: "Option[str]" = None, score: "Option[str]" = None) -> Code:
        """
        Used for questions of type 'Single/ Multiple/ Date/ Time'
        """
    @property
    def value(self, /) -> Any:
        """
        Code value. This corresponds to the `code` attribute of triple-s
        `<value>` tag. This can be float, string or int.
        Code value `string` is introduced in triple-s xml 2.0 (via
        format='literal' toggle).
        """

@final
class CodeFormat:
    """
    Flag for indicating the data type of `code` field in a `<value>` node. Only
    used for `Single` type of questions.

    Unsigned:
    The default. Code value being u32, `format` attribute `numeric`

    Literal:
    Introduced since SSS 2.0, basically something like:

    ```xml
    <variable ident="11" type="single" format="literal">
        <values>
            <value code="B">Between 3 months and 1 year</value>
            <value code="C">More than 1 years time</value>
        </values>
    </variable>
    ```
    """
    Literal: Final[CodeFormat]
    """
    Single response question with `format='literal'`, introduced since SSS
    2.0. Literal response codes are strings and do not support a value
    range. An example would be:

    ```xml
    <variable ident="11" type="single" format="literal">
        <values>
            <value code="A">Within 3 months</value>
            <value code="B">Between 3 months and 1 year</value>
            <value code="C">More than 1 years time</value>
        </values>
    </variable>
    ```
    """
    UnSigned: Final[CodeFormat]
    """
    Single/ Multiple response type with code value being u32, `format`
    attribute `numeric`
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class DataFormat:
    """
    Flags that collects all allowed values of `format` attribute of a `<record>`
    element. This is used to indicate types of the data the current datamap file
    binds to.

    Introduced in triple-s 2.0
    """
    Csv: Final[DataFormat]
    """
    Supported from sss 2.0 onwards.
    the data representation is comma separated values, using one field
    for each variable, with data values similar to the fixed format. Note
    that for data in csv format the position element refers to the
    field number.
    """
    Fixed: Final[DataFormat]
    """
    The data representation is fixed format fields.
    Basically `.asc` text file that formatted bit like csv but without any
    column header.
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class DataRange:
    """
    Minimum and maximum values allowed for a quantity response, or an
    implicit range of codes for a numeric categorical response.

    For character responses, the `end` stores the maximum size plus one
    (triple-s is 1-indexed). using `start-end` instead of `from-to` because
    `from` is a preserved keyword in python.

    Fields:

    - start: lower bound of the range.
    - end: upper bound of the range.
    """
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, start: Any, end: Any) -> DataRange: ...
    def __repr__(self, /) -> str: ...
    @property
    def end(self, /) -> Any:
        """
        End of the `<range>`
        """
    @end.setter
    def end(self, /, value: Any) -> None: ...
    @staticmethod
    def float(start: float, end: float) -> DataRange:
        """
        creates a new instance with values being decimals.
        This is used for questions of 'quantity' type only.
        """
    @staticmethod
    def from_dict(content: dict) -> DataRange:
        """
        creates new instance from dictionary with the following keys:

        - start (float|int|str): start of a data range
        - end (float|int|str): end of a data range
        """
    @staticmethod
    def signed(start: int, end: int) -> DataRange:
        """
        creates a new instance with values being integers that can either be
        positive/ negative. This is used for questions of 'quantity' type
        only.
        """
    @property
    def start(self, /) -> Any:
        """
        Start of the data range.
        """
    @start.setter
    def start(self, /, value: Any) -> None: ...
    @staticmethod
    def unsigned(start: int, end: int) -> DataRange:
        """
        creates a new instance with values being exclusively positive integers.
        This is used for questions of 'single', 'multiple', and 'date/ time'
        types.
        """

@final
class Datamap:
    """
    Datamap describes how to interpret the data file, a datamap consists of two parts:
    - survey metadata in ``info`` and,
    - question definitions in ``record``. 

    Read a datamap
    --------------

    Load an existing datamap with :func:`triple_s.read_datamap`. Both Triple-S
    Classic and XML datamaps are supported. This reads the definition only;
    it does not load the data file referenced by the record.

    .. code-block:: python

       from triple_s import read_datamap

       datamap = read_datamap("survey.sss")
       print(datamap.title)
       print(datamap.info.name)

    Create new datamap
    ------------------------

    Alternatively, you can build a datamap with ``Datamap.from_dict()``:

    .. code-block:: python

       from triple_s import Datamap

       datamap = Datamap.from_dict({
           "info": {"version": "2.0", "name": "example", "title": "Example survey"},
           "record": {"ident": "A"},
       })
       datamap.info.name = "updated-example"
       datamap.title = "Updated survey"
       assert datamap.info.title == "Updated survey"

    You can either pass your list of questions to the `record` child dictionary, 
    or ues :func:`Datamap.record.add_question` and :func:`Datamap.record.remove_question` later too add/ 
    remove survey questions:

    .. code-block:: python

       datamap.add_question(q1)
       datamap.remove_question(q2)


    Access questions
    ----------------

    To see out all questions in a datamap, use :py:attr:`Record.questions`, which lists question names in record order.


    Load and export survey data
    ------------------------------

    :func:`Datamap.load_data()`/ :func:`Record.load_data()`:

    *Loading and validation support UTF-8 and BOM-marked UTF-16; invalid sequences become ``�`` (U+FFFD), each occupying one fixed-width position.*

    .. code-block:: python

       data = datamap.load_data("survey.asc")
       df = data.to_polars()
       pandas_df = data.to_pandas()

    Loading returns a :class:`triple_s.SurveyData` instance, 
    it updates the datamap record's ``href`` to to the path of the data file, as well as ``data_format`` to the type of data file. 

    *To read the definition and respondent data together, use :func:`triple_s.read_triple_s`:*


    Export the datamap
    -----------------------

    ``to_classic()`` returns Triple-S classic document

    .. code-block:: python

       classic = datamap.to_classic()


    ``to_xml()`` returns a complete Triple-S XML document:

    .. code-block:: python

       xml = datamap.to_xml()

    """
    def __getitem__(self, index: Any, /) -> Any: ...
    def __iter__(self, /) -> Any: ...
    def __repr__(self, /) -> str: ...
    @staticmethod
    def from_dict(content: dict) -> Datamap:
        """
        Create a datamap from a dictionary that contains keys 'survey_info' and 'record':

        Example: Creating new empty :py:class:`Datamap` from dictionary:

        ```python

        from triple_s import Datamap, SurveyInfo, Record

        survey_info = SurveyInfo(version=1.2)  # must contain sss version
        record = Record.empty()

        datamap = Datamap.from_dict({
            'survey_info': survey_info,
            'record': record
        })
        ```
        """
    def get(self, /, question_name: "str") -> Question | None:
        """
        get question by question name, returns `None` if question name not exist.
        """
    @property
    def info(self, /) -> SurveyInfo:
        """
        Collects survey meta information, i.e., survey name & title, creation date, etc.,
        """
    def load_data(self, /, path: "str") -> SurveyData:
        """
        Load an `.asc` or `.csv` file using the record's skip setting.
        Invalid Unicode sequences become `�` (U+FFFD).
        Returns `SurveyData` and updates the record's `href` and `data_format`.
        The returned data is not stored on the datamap.
        """
    @property
    def num_questions(self, /) -> int:
        """
        Total number of questions in the survey datamap
        """
    @property
    def question_names(self, /) -> list[str]:
        """
        List of question names in the survey
        """
    @property
    def record(self, /) -> Record:
        """
        Datamap record. For more info please check docuentation of the `Record`
        class.
        """
    @property
    def title(self, /) -> str | None:
        """
        (optional) Title of the survey
        """
    @title.setter
    def title(self, /, title: str | None) -> None: ...
    def to_classic(self, /) -> str:
        """
        export content of datamap to triple-s classic string.
        """
    def to_xml(self, /) -> str:
        """
        Serialises the survey as a complete Triple-S XML `<sss>` document.
        """

@final
class Location:
    """
    Defines the part of the data record that is allocated to holding the value
    of the variable.
    """
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, start: "int", finish: "int" = None) -> Location:
        """
        Raises `ValueError` if `finish` is less than `start`.

        # Examples

        ```python
        from triple_s import Location

        location = Location(1, 20)
        single_column = Location(1)
        ```
        """
    def __repr__(self, /) -> str: ...
    @property
    def finish(self, /) -> int:
        """
        End of a data location, this corresponds to the `end` attribute of the
        `<position>` element
        """
    @finish.setter
    def finish(self, /, finish: int) -> None: ...
    @staticmethod
    def from_dict(value: dict) -> Location:
        """
        Creates new instance from dictionary with the following keys:

        'start' (int): start of a data location
        'finish' (int, *optional*): end of a data location
        """
    @property
    def start(self, /) -> int:
        """
        Start of a data location, this corresponse to the `start` attribute of
        the `<position>` element
        """
    @start.setter
    def start(self, /, start: int) -> None: ...

@final
class Mode:
    """
    Introduced in SSS Xml 2.0.
    Official documentation:
    Text modes, available from Triple-S XML 2.0:

    ```text
    (Optional) The use of specialised texts for interviewing and/or analysis within the
    specification must be signalled by a modes="mode_identifier_list"
    attribute on the initial <sss> element.
    Two explicit modes are available: “interview” and “analysis”. In the absence of
    a mode specification, the appropriate text is assumed to be used in both
    modes.
    ```
    """
    Analysis: Final[Mode]
    Interview: Final[Mode]
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...

@final
class Question:
    """
    A survey variable, represented by a `<variable>` element in Triple-S XML.
    # Question

    A question definition, represented by `<variable>` in Triple-S XML.

    Respondent data is returned by `Datamap.load_data()` or `Record.load_data()`.
    Use the returned `SurveyData.to_polars()` or `SurveyData.to_pandas()` to export it.

    Args:
    name (str):
        Question name.

    question_type (triple_s.QuestionType | str | None):
        The XML `type` attribute: "single", "multiple", "logical", "quantity",
        "character", "date", or "time". Use `None` for a question with no response values.

    label (str, *Optional*):
        Question text (`<label>`); defaults to an empty `Text`.

    filter (str|None, *Optional*):
        Question filter (`<filter>`); defaults to `None`.

    format (str, *Optional*):
        Code format: "numeric", "literal", or `None` (default).

    usage (str, *Optional*):
        Variable role (`use`): "serial", "weight", or `None` (default).

    location (triple_s.Location, *Optional*):
        Data position (`<position>`); defaults to `Location(0, 0)`.
        Set this before loading data.

    spread (triple_s.Spread, *Optional*):
        Subfields for multiple-choice questions (`<spread>`); defaults to `None`.
    """
    def __copy__(self, /) -> Question:
        """
        Creates an independent deep copy of this question.
        Changes to the copy do not affect the original.
        """
    def __deepcopy__(self, /) -> Question:
        """
        Creates an independent deep copy of this question.
        Changes to the copy do not affect the original.
        """
    def __new__(cls, /, name: "str", question_type: "str|QuestionType|None", label: "str" = None, filter: "str" = None, format: "'numeric'|'literal'|None" = None, usage: "'serial'|'weight'|None" = None, location: Location | None = None, spread: Spread | None = None) -> Question:
        """
        Create a question with index zero and an empty response definition.
        Accepts a type name or a `QuestionType` flag; optional metadata defaults
        to `None`.

        # Examples

        ```python
        from triple_s import Question

        question = Question("Q1", "single")
        ```
        """
    def __repr__(self, /) -> str:
        """
        Returns a representation containing the question name, type, and label.
        """
    def copy(self, /) -> Question:
        """
        Creates an independent deep copy of this question.
        Changes to the copy do not affect the original.
        """
    @property
    def response_label(self, /) -> dict:
        """
        Get the lookup table for items in the `Question.response.values`, asdictionary of { value: label }

        Note that this methods returns a *deepcopy* of the data, and modifying items returned by this
        method will not mutate the caller.
        """
    @property
    def filter(self, /) -> str | None:
        """
        Returns the optional name of a previously defined logical variable used
        as a filter. That variable determines whether this question is
        available for a record.
        """
    @filter.setter
    def filter(self, /, filter: str | None) -> None:
        """
        Sets the name of the logical variable used as a filter, or clears it
        with `None`. The referenced variable should precede this question in
        the record.
        """
    @property
    def format(self, /) -> str | None:
        """
        Flag for indicating the data type of `code` field in a <value> node.
        Only used for `Single` type of questions.

        'numeric':
        The default. Code value being u32, `format` attribute `numeric`

        'literal:
        Introduced since SSS 2.0, basically something like:
        <variable ident="11" type="single" format="literal">
            <values>
                <value code="A">Within 3 months</value>
                <value code="B">Between 3 months and 1 year</value>
                <value code="C">More than 1 years time</value>
            </values>
        </variable>
        ```
        """
    @format.setter
    def format(self, /, format: Any) -> None:
        """
        Sets the response code format, or clears the explicit format with
        `None`. Accepts a `CodeFormat` flag or a format string.
        """
    @staticmethod
    def from_dict(content: dict) -> Question:
        """
        Create new [ `Question` ] instance from dictionary with following keys and
        values:

        `'index'` (int|None, *optional*): index of a question, default 1,

        `'name'` (str):
            name of the question

        `'question_type'` (str|QuestionType ):
            type of the question.
            See QuestionType.available_options() for list of allowed string values.

        `'location'` (dict|Location|None, *optional*):
            data location.
            See help(Location.from_dict) for allowed keys in the dictionary.
            If unset, would defult to :py:meth:`Location(start=1, finish=1)`

        `'label'` (str|Text|dict):
            question title.
            See help(:py:meth:`Text.from_dict`) for allowed keys in the dictionary.
            If unset, would defult to ``Text("")``

        `'format'` (str|CodeFormat|None, *optional*):
            code format.
            See :py:meth:`CodeFormat.available_options` for list of allowed string values.

        `'filter'` (str|None, *optional*):
            Question filter.

        `'response'` (Response|dict|None, *optional*):
            List of possible responses for the question.

            Accepts :py:class:`Response` instance or dictionary to pass to :py:meth:`Response.from_dict`,
            
            Would default to the default response values according to each question type if left unset
        """
    @staticmethod
    def from_string(question_fragment: "str") -> Question:
        """
        Parses a question from a Triple-S classic or XML fragment, detecting the
        format automatically. Raises `ValueError` if the fragment cannot be
        parsed into a question.

        ```python
        from triple_s import Question

        original = r""\"
           VARIABLE 10
           NAME "WEBSITE"
           LABEL "Test label"
           TYPE CHARACTER
           POSITION 1042 TO 1241
           SIZE 200
           END VARIABLE
           ""\"
        question = Question.from_string(original)
        print(question)
        ```

        # Example: read from triple-s xml
        ```python
        from triple_s import Question

        original = r""\"
            <variable ident="8" type="multiple">
                <name>Q5</name>
                <label>Two favourite attractions visited</label>
                <position start="62" finish="63" />
                <spread subfields="2" />
                <values>
                    <value code="1">Sherwood Forest</value>
                </values>
            </variable>
        ""\"

        question = Question.from_string(original)
        print(question)
        ```
        """
    @property
    def label(self, /) -> Text:
        """
        Returns the label or original question text.
        The text may include language-specific or mode-specific variants.
        """
    @label.setter
    def label(self, /, label: Any | None) -> None:
        """
        Sets the label from a string or `Text`, or resets it to empty with
        `None`. A string updates the existing text; a `Text` object replaces
        it.
        """
    @property
    def location(self, /) -> Location:
        """
        Returns the data location described by `<position>`.
        Triple-S uses one-based character positions for fixed-width records and
        one-based field numbers for CSV records.
        """
    @location.setter
    def location(self, /, location: Location) -> None: ...
    @property
    def name(self, /) -> str:
        """
        Returns the variable name recorded in the `<name>` element.
        """
    @name.setter
    def name(self, /, name: str) -> None:
        """
        Sets the variable name recorded in the `<name>` element.
        """
    @property
    def response(self, /) -> Response:
        """
        Returns the response definition, including explicit codes and an
        optional range. Its interpretation depends on the question type;
        character variables use it to describe their maximum size.
        """
    @response.setter
    def response(self, /, response: Response) -> None: ...
    def set_location(self, /, start: "int" = None, finish: "int" = None) -> None:
        """
        Updates data location of the current question.

        Example:

        ```python
        from triple_s import Question

        question = Question("Q1", "single")
        print("Before update: ", question.location)
        question.set_location(start=0, finish=30)
        print("After update: ", question.location)
        ```
        """
    @property
    def spread(self, /) -> Spread | None:
        """
        Returns the optional `<spread>` layout for a multiple-response variable.

        `spread` field controls how to parse response data of questions with
        multi-choice reponses, more information can be found in the help
        documentation of `Spread` class:

        ```python
        from triple_s import Spread

        help(Spread)
        ```
        """
    def to_classic(self, /) -> str:
        """
        Serialises the question as a Triple-S classic `VARIABLE` block.
        """
    def to_xml(self, /) -> str:
        """
        Serialises the question as a Triple-S XML `<variable>` element.
        """
    @property
    def type(self, /) -> str | None: ...
    @type.setter
    def type(self, /, question_type: Any | None) -> None: ...
    @property
    def usage(self, /) -> str | None:
        """
        Returns the optional role declared by the `use` attribute.
        `serial` identifies records, while `weight` supplies their analysis
        weights.
        """
    @usage.setter
    def usage(self, /, value: Any) -> None:
        """
        Sets the variable role, or clears the explicit role with `None`.
        Accepts a `Usage` flag or a role string.
        """

@final
class QuestionData:
    """
    Data for a single question.
    """
    def __getitem__(self, index: int, /) -> Any: ...
    def __iter__(self, /) -> Any: ...
    def __repr__(self, /) -> str: ...
    @property
    def data(self, /) -> tuple:
        """
        Get a copy of question data as a Python tuple.
        """
    @property
    def response_label(self, /) -> dict:
        """
        Get the lookup table for items in the `Question.response.values`, asdictionary of { value: label }

        Note that this methods returns a *deepcopy* of the data, and modifying items returned by this
        method will not mutate the caller.
        """
    @property
    def datamap(self, /) -> Question:
        """
        Datamap of the current question

        Note:

        *Modifying the datamp would also modify the datamap in the survey that the
        current instance being a part of.*
        """
    @property
    def label(self, /) -> Text:
        """
        Title of the question
        """
    @property
    def name(self, /) -> str:
        """
        Name of the question
        """
    @property
    def range(self, /) -> DataRange | None:
        """
        Minimum and maximum values allowed for a quantity response,
        or an implicit range of codes for a numeric categorical response.
        For character responses, the end stores the maximum size plus one.
        String categorical responses do not support a range.
        """
    def to_list(self, /) -> Any:
        """
        Export the answers as a list.
        """
    def to_pandas(self, /) -> Any:
        """
        Export the answers as a pandas Series object.
        """
    def to_polars(self, /) -> Incomplete:
        """
        Return the answers as a polars Series.
        """
    @property
    def type(self, /) -> str | None:
        """
        Type of the question ('single', 'multiple' etc.,)
        """
    def value_counts(self, /, count_nan: "bool" = None) -> Incomplete:
        """
        Count unique values of this question data.

        This function basically calls ``poalrs.Series.value_counts`` under the hood, apart from multi-choice
        where the choice for each respondant is counted individually.

        Args:
            count_nan (`bool`, *optional*):
                Toogle on/ off for counting Null values, note that multi-choices entries of an empty list (`[]`) will also count as `null`.
                Default ``False``

        Returns:
            ``polars.DataFrame`` of the value counts
        """

@final
class QuestionType:
    """
    type of question
    """
    Character: Final[QuestionType]
    """
    character response type
    """
    Date: Final[QuestionType]
    """
    date response type
    """
    Logical: Final[QuestionType]
    """
    Logical response type
    """
    Multiple: Final[QuestionType]
    """
    Multiple response question with numeric response codes.
    """
    None: Final[QuestionType]
    """
    No response values. Used for non-standard note or instruction pages,
    such as a blank page with just a 'Next Page' button.
    """
    Quantity: Final[QuestionType]
    """
    Quantity type of response.
    """
    Single: Final[QuestionType]
    """
    Single response question with numeric or literal response codes.
    """
    Time: Final[QuestionType]
    """
    time response type
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...
    @staticmethod
    def available_options() -> list[str]: ...

@final
class Record:
    """
    Content of a datamap
    """
    def __getitem__(self, index: Any, /) -> Any: ...
    def __iter__(self, /) -> Any: ...
    def __len__(self, /) -> int: ...
    def add_question(self, /, question: Question) -> None:
        """
        Add a question to the current [ `Record` ] instance

        Would overwrite existing item if question with the same name already exists.
        """
    @property
    def content(self, /) -> tuple:
        """
        Collections of [ `Question` ] instances of a survey, this corresponds to the
        lists of `<variable>` child nodes
        """
    @property
    def data_format(self, /) -> DataFormat | None:
        """
        Indicating types of the data the current datamap file binds to
        This corresponds to the `format` attribute of a `<record>` element.

        Introduced in triple-s 2.0
        """
    @data_format.setter
    def data_format(self, /, data_format: Any | None) -> None: ...
    @staticmethod
    def empty() -> Record:
        """
        Creates a new empty [`Record`] instance.

        Example:

        ```python

        from triple_s import Record

        record = Record.empty()
        assert len(record) == 0
        ```
        """
    @staticmethod
    def from_dict(content: dict) -> Record:
        """
        Create new instance from dictionary with the following keys:

        'ident' (str|None, *optional*):
            id of the [ `Record` ] instance, default 'A'.

        'href' (str|None, *optional*):
            href,
            Default to be 'None'

        'data_format' (str|DataFormat|None, *optional*):
            data source, default 'None'

        'content' (List[Question|dict], *optional*):
            list of [`Question`] instance or dictionaries to pass to
            Question.from_dict,
            Default to be an empty list (`[]`)
        """
    def get(self, /, question_name: "str") -> Question | None:
        """
        Get a question by name, returning `None` if it doesn't exist.

        Name matching is not case sensitive.
        """
    @property
    def href(self, /) -> str | None:
        """
        (Optional) path to the data file the current datamap ties to.

        This corresponds to the href attribute of a <record> element.
        Setting it edits metadata only; use `load_data(path)` to load respondent data.
        """
    @href.setter
    def href(self, /, href: str | None) -> None: ...
    @property
    def ident(self, /) -> str:
        """
        Corresponds to the `ident` attribute of a `<record>` element.

        Basically it's a single-character from A-Z/ a-z for id.
        """
    @ident.setter
    def ident(self, /, value: str) -> None:
        """
        Set the id.
        """
    def load_data(self, /, path_data: str) -> SurveyData:
        """
        Load respondent data and return `SurveyData`, updating `href` and `data_format`.
        Invalid Unicode sequences become `�` (U+FFFD).
        """
    @property
    def num_questions(self, /) -> int:
        """
        Total num of questions in the record. Also used by `len()`
        """
    @property
    def question_names(self, /) -> list[str]:
        """
        List of question names in the record
        """
    def remove_question(self, /, question_name: str) -> Question | None:
        """
        Remove a question from current record, returns the removed question.

        Name matching is case insensitive.
        """
    @property
    def skip(self, /) -> str | None:
        """
        (Optional) corresponds to the 'skip' attribute of the `<record>` xml element.

        Used to ignore one or more initial records in the Data File.
        This will be most useful for csv Data Files where the first line is often used as
        documentation (e.g. names for the columns/fields) for the succeeding values.
        """
    @skip.setter
    def skip(self, /, skip: str | None) -> None: ...
    def to_xml(self, /) -> str: ...
    def validate_data(self, /, path: str) -> None:
        """
        Validate an `.asc` or `.csv` file without changing metadata or loading data.
        Invalid Unicode sequences become `�` (U+FFFD).
        Use `load_data(path)` to load respondent data.
        """

@final
class Response:
    """
    Response codes and an optional range for a datamap question.
    This corresponds to the `<values>` child-node inside a `<variable>` tag.
    The question's [`QuestionType`] determines how these values are interpreted:
    categorical questions list response codes, quantity questions describe
    allowed values and special codes, and character questions describe the
    response size.
    """
    def __getitem__(self, index: Any, /) -> Any: ...
    def __new__(cls, /, range: "DataRange" = None, values: "list[Code]" = None) -> Response: ...
    def __repr__(self, /) -> str: ...
    def add_value(self, /, value: Code) -> None:
        """
        Append a code, rejecting duplicate values and different Value variants.
        """
    @property
    def response_label(self, /) -> dict: ...
    @staticmethod
    def from_dict(content: dict) -> Response:
        """
        create new instance from dictionary with the following keys:

        'range' (dict|DataRange|None, *optional*):
            data range, default None

        'values' (list[Code|dict]):
            list of code values or dictionary to to `Code.from_dict`
        """
    def get_value(self, /, value: "str|int|float") -> Code | None:
        """
        get [ `Code` ] inside `self.values` with `code.value` given.

        Returns `None` if the input `code` was not found.
        """
    @property
    def range(self, /) -> DataRange | None:
        """
        Minimum and maximum values allowed for a quantity response, or an
        implicit range of codes for a numeric categorical response.

        For character responses, the end stores the maximum size plus one.
        String categorical responses do not support a range.
        """
    def set_value(self, /, value: Code) -> None:
        """
        Update or append a code, rejecting different Value variants.
        Will overwrite existing code if they share the same 'code' attribute.
        """
    @property
    def values(self, /) -> tuple:
        """
        Code values of a question, mostly used for Single/ Multiple question
        types.

        This is also used for indicating some special values and their meanings
        for Quantity question type. (i.e., value of '9999.9999' in a
        specific quantity question might mean 'NaN').
        """

@final
class Spread:
    """
    Controls how multiple-choice responses are stored: the optional `<spread>`
    element in Triple-S XML or `SPREAD` line in Triple-S Classic.

    Without spread (default)
    ------------------------------

    When spread is absent (None), each character in the question's data field
    represents a category: 1 means selected and 0 means not selected.

    Positions start at 1 and correspond directly to category codes:

    - Category codes: 1, 2, 3, 4, 5
    - Selection flags: 1, 0, 1, 0, 0 (stored as "10100")
    - Selected codes: [1, 3]

    With spread
    --------------

    When spread is present, the data contains consecutive slots called
    subfields. The subfield count specifies the number of slots; width
    specifies the number of characters in each slot.

    Width can be omitted for fixed-width (.asc) data when equal-width slots
    exactly fill the question's field. It is then calculated as the field
    width divided by the subfield count. Width is required for CSV data.

    Examples
    --------

    Two subfields, each one character wide:

    - Raw data: "13"
    - Split: "1" | "3"
    - Selected codes: [1, 3]

    One subfield, two characters wide:

    - Raw data: "13"
    - Split: "13"
    - Selected codes: `[13]`

    Two subfields, each two characters wide. Zero marks an unused slot:

    - Raw data: "1300"
    - Split: "13" | "00"
    - Selected codes: `[13]`

    Three subfields, each three characters wide:

    - Raw data: "001042999"
    - Split: "001" | "042" | "999"
    - Selected codes: [1, 42, 999]
    """
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, num_subfields: int, subfield_width: int | None = None) -> Spread:
        """
        # Examples

        ```python
        from triple_s import Spread

        spread = Spread(2, subfield_width=1)
        without_width = Spread(2)
        ```
        """
    @staticmethod
    def from_dict(value: dict) -> Spread:
        """
        Creates new instance from dictionary with the following keys:

        'num_subfields' (int): number of subfields.
        'subfield_width' (int, *optional*): width of each subfield.
        """
    @property
    def num_subfields(self, /) -> int:
        """
        The number of subfields within the overall field that is defined by the
        `<position>` element.
        """
    @num_subfields.setter
    def num_subfields(self, /, num_subfields: int) -> None: ...
    @property
    def subfield_width(self, /) -> int | None:
        """
        Denotes the width of each subfield.
        """
    @subfield_width.setter
    def subfield_width(self, /, subfield_width: int | None) -> None: ...

@final
class Survey:
    """
    Triple-s survey.
    """
    def __getitem__(self, index: Any, /) -> Any: ...
    def __iter__(self, /) -> None: ...
    def __new__(cls, /, datamap: "Datamap", data: "SurveyData" = None) -> Survey:
        """
        Creates new `Survey` instance
        """
    def __repr__(self, /) -> str: ...
    @property
    def data(self, /) -> SurveyData:
        """
        Survey data
        """
    @data.setter
    def data(self, /, data: SurveyData) -> None:
        """
        Replace respondent data while keeping existing question views attached.
        """
    @property
    def datamap(self, /) -> Datamap:
        """
        Datamap
        """
    def has_data(self, /) -> bool:
        """
        Check if data loaded or not
        """
    @property
    def info(self, /) -> SurveyInfo:
        """
        Survey metadata, shared with `datamap.info`.
        """
    def iter_questions(self, /) -> Any: ...
    def iter_rows(self, /) -> Any:
        """
        Iter each individual respondant data
        """
    @property
    def name(self, /) -> str | None:
        """
        (optional) Name of the survey
        """
    @name.setter
    def name(self, /, name: str | None) -> None: ...
    @property
    def num_response(self, /) -> int: ...
    @property
    def title(self, /) -> str | None:
        """
        (optional) Title of the survey
        """
    @title.setter
    def title(self, /, title: str | None) -> None: ...

@final
class SurveyData:
    """
    Respondent data with a lazy query plan and an optional backing cache.
    """
    def __iter__(self, /) -> Any: ...
    def __repr__(self, /) -> str: ...
    def describe(self, /) -> Any:
        """
        Get a brief overview of each column of data, equivalent of `self.to_polars().describe()`
        """
    @staticmethod
    def empty() -> SurveyData:
        """
        Create new empty [`SurveyData`] instance
        """
    @staticmethod
    def from_polars(frame: Incomplete) -> SurveyData:
        """
        Create new [`SurveyData`] instance from `polars.DataFrame`
        """
    def has_data(self, /) -> bool:
        """
        Check if data loaded or not
        """
    @property
    def num_response(self, /) -> int: ...
    @property
    def questions(self, /) -> list[str]:
        """
        List of question names in the current instance
        """
    def to_pandas(self, /) -> Any:
        """
        Export loaded data as a pandas DataFrame
        """
    def to_polars(self, /) -> Any:
        """
        Export data as polars DataFrame
        """

@final
class SurveyInfo:
    """
    meta information about the datamap entity.
    """
    def __new__(cls, /, version: "str | float", name: "str | None" = None, title: "str | None" = None, date: "str | None" = None, time: "str | None" = None, origin: "str | None" = None, user: "str | None" = None, style: "str | None" = None, language_options: "list[str] | None" = None, mode_options: "list[Mode | str | None] | None" = None, standard: "str | None" = None) -> SurveyInfo:
        """
        Create datamap metadata for a Triple-S version given as a string or
        float.

        # Examples

        ```python
        from triple_s import SurveyInfo

        meta = SurveyInfo("2.0", language_options=["en-GB"], mode_options=["analysis"])
        ```
        """
    def __repr__(self, /) -> str: ...
    @property
    def date(self, /) -> str | None:
        """
        date of the creation
        """
    @date.setter
    def date(self, /, date: str | None) -> None: ...
    @staticmethod
    def from_dict(content: dict) -> SurveyInfo:
        """
        Create survey metadata from a dictionary. `version` is required;
        other metadata fields are optional and default to `None` or an empty
        options list.
        """
    @property
    def language_options(self, /) -> tuple:
        """
        ** Introduced since sss 1.2 **
        list of all available language options
        """
    @language_options.setter
    def language_options(self, /, values: Sequence[str]) -> None: ...
    @property
    def mode_options(self, /) -> tuple:
        """
        ** Introduced since sss 1.2 **
        list of all available `mode` options
        """
    @mode_options.setter
    def mode_options(self, /, values: Sequence[Any]) -> None: ...
    @property
    def name(self, /) -> str | None:
        """
        name of the survey
        """
    @name.setter
    def name(self, /, name: str | None) -> None: ...
    @property
    def origin(self, /) -> str | None:
        """
        the originating system (program and operating system), for example(&self) -> RwLockReadGuard<'_,
        `<origin>MyProg v3, Windows 7</origin>`
        """
    @origin.setter
    def origin(self, /, origin: str | None) -> None: ...
    @property
    def standard(self, /) -> str | None:
        """
        ** Deprecated after sss version 1.2 **
        (Optional) indicate that some aspect of the triple-s definition meets a
        defined standard that may reduce the amount of checking required by an
        import program. The only value that option_setting may have in
        version 1.2 is 'standardnames'.
        """
    @standard.setter
    def standard(self, /, standard: str | None) -> None: ...
    @property
    def style(self, /) -> str | None:
        """
        html css styling
        """
    @style.setter
    def style(self, /, style: str | None) -> None: ...
    @property
    def time(self, /) -> str | None:
        """
        time of the creation
        """
    @time.setter
    def time(self, /, time: str | None) -> None: ...
    @property
    def title(self, /) -> str | None:
        """
        title of the survey
        """
    @title.setter
    def title(self, /, title: str | None) -> None: ...
    def to_dict(self, /) -> "dict[str, str]":
        """
        Export contents to a dictionary.

        Example:

        ```python
        from triple_s import SurveyInfo

        info = SurveyInfo(
            version=Version.V2_0,
            name='SP5201-1',
            title='Historic House Exit Survey<br/>First Wave',
            date='14 April 2005',
            time='16:00',
            origin='SurveyProg v1.3.05',
            user='User Site',
            style=None,
            language_options=(),
            mode_options=(),
            standard=None,
        )

        print(info.to_dict())
        ```
        """
    @property
    def user(self, /) -> str | None:
        """
        user who created it
        """
    @user.setter
    def user(self, /, user: str | None) -> None: ...
    @property
    def version(self, /) -> Version:
        """
        The version attribute is used to indicate the version of the Triple-S
        standard that applies to this specification.
        """
    @version.setter
    def version(self, /, version: Any) -> None: ...

@final
class Text:
    """
    `Text` represents a default text value together with zero or more
    specialised [`Variant`] values.
    """
    def __copy__(self, /) -> Text: ...
    def __deepcopy__(self, /) -> Text: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __new__(cls, /, value: str, variants: Sequence[Variant] | None = None) -> Text:
        """
        Create default text with an optional list of variants.

        # Examples

        ```python
        from triple_s import Text, Variant

        variant = Variant("original", mode="analysis", language="en-GB")
        text = Text("original", variants=[variant])
        ```
        """
    def __repr__(self, /) -> str: ...
    def add_variant(self, /, variant: Any, mode: str | None = None, language: str | None = None) -> None: ...
    @staticmethod
    def from_dict(content: dict) -> Text:
        """
        Create a new [`Text`] instance from a dictionary with the following
        keys:

        - 'value' (str): The text value.
        - 'variants' (list[dict[str, str] | Variant], optional): A list of text
          variants. Each item may be either an existing [`Variant`] instance or
          a dictionary used to construct a [`Variant`] via Variant.from_dict.
        """
    def remove_variant(self, /, index: int) -> Variant: ...
    def to_dict(self, /) -> dict:
        """
        Convert contents of current isntance to a dictionary.

        Instance with same content can later be constructed again via `from_dict` method.

        Example
        ----------

        ```python
        from triple_s import Text

        data = dict(
            value = "Text value",
            variants = [
                dict(value="variant 1", mode="analysis"),
                dict(value="variant 2", mode="analysis", language="tr")
            ]
        )

        original = Text.from_dict(data)
        converted = original.to_dict()

        new_instance = Text.from_dict(converted)
        assert new_instance == original
        ```
        """
    @property
    def value(self, /) -> str:
        """
        Default value
        """
    @value.setter
    def value(self, /, value: str) -> None: ...
    @property
    def variants(self, /) -> tuple: ...
    @variants.setter
    def variants(self, /, variants: Sequence[Variant]) -> None: ...

@final
class Usage:
    """
    describes the role of this variable in the datamap.
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
    @staticmethod
    def available_options() -> list[str]: ...

class Value:
    """
    Used for recording `code` attribute of triple-s ``<value>``` tag, as well as
    `from` `to` of `\\<range\\>`` tag.

    - Unsigned: used in Single/ Multiple/ Quantity question types.
    - Signed: used in Quantity/ Date/ Time question types.
    - Float: used in Quantity question type.
    - String: used in Single question type, introduced from triple-s 2.0 onwards

    # Example
    ```rust
    use triple_s::response::Value;

    let value = Value::Float(32.0);
    ```
    """
    @final
    class Float(Value):
        """
        float (used only for quantity qurstion type)
        """
        __match_args__: Final = ("_0",)
        @property
        def _0(self, /) -> float: ...
        def __getitem__(self, key: int, /) -> Any: ...
        def __len__(self, /) -> int: ...
        def __new__(cls, /, _0: float) -> Value.Float: ...
    @final
    class Signed(Value):
        """
        signed int (used only for quantity qurstion type)
        """
        __match_args__: Final = ("_0",)
        @property
        def _0(self, /) -> int: ...
        def __getitem__(self, key: int, /) -> Any: ...
        def __len__(self, /) -> int: ...
        def __new__(cls, /, _0: int) -> Value.Signed: ...
    @final
    class String(Value):
        """
        string (used only for triple-s 2.0 onwards for question type `Single`)
        """
        __match_args__: Final = ("_0",)
        @property
        def _0(self, /) -> str: ...
        def __getitem__(self, key: int, /) -> Any: ...
        def __len__(self, /) -> int: ...
        def __new__(cls, /, _0: str) -> Value.String: ...
    @final
    class UnSigned(Value):
        """
        unsigned int (used for single, mutiple, date and time question types)
        """
        __match_args__: Final = ("_0",)
        @property
        def _0(self, /) -> int: ...
        def __getitem__(self, key: int, /) -> Any: ...
        def __len__(self, /) -> int: ...
        def __new__(cls, /, _0: int) -> Value.UnSigned: ...

@final
class Variant:
    """
    Introduced since sss 2.0 onwards.

    Official documentation:

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
    def __new__(cls, /, value: "str", mode: "'analysis'|'interview'|Mode|None" = None, language: "str|None" = None) -> Variant:
        """
        Create a text variant.

        Mode and language default to `None`.

        # Examples

        ```python
        from triple_s import Variant

        variant = Variant("original", mode="analysis", language="en-GB")
        ```
        """
    def __repr__(self, /) -> str: ...
    def copy(self, /) -> Variant:
        """
        makes deepcopy of the current instance.
        """
    @staticmethod
    def from_dict(content: dict) -> Variant:
        """
        Create new `Variant` instance from dictionary with the following keys:

        - 'value' (str): content of the text variant

        - 'mode' (str | Mode, *optional*): mode, this can be one of ['analysis',
          'interview', None], or one of [`Mode`] enum member.

        - 'language' (str, *optional*): language name

        Note that one of the `mode`, `language` key must present.

        Example: Creating new variant from dictionary:

        ```python
        from triple_s import Variant

        value = {"value": "test text", "mode": "analysis", "language": "en-GB"}
        variant = Variant.from_dict(value)
        ```
        """
    @property
    def language(self, /) -> str | None:
        """
        Corresponds to the `xml::lang` attribute of a `<text>` node.
        """
    @language.setter
    def language(self, /, language: str | None) -> None: ...
    @property
    def mode(self, /) -> str | None:
        """
        Introduced in SSS Xml 2.0.
        Official documentation:
        Text modes, available from Triple-S XML 2.0:

        ```text
        (Optional) The use of specialised texts for interviewing and/or analysis within the
        specification must be signalled by a modes="mode_identifier_list"
        attribute on the initial <sss> element.
        Two explicit modes are available: “interview” and “analysis”. In the absence of
        a mode specification, the appropriate text is assumed to be used in both
        modes.
        ```
        """
    @mode.setter
    def mode(self, /, mode: str | None) -> None: ...
    def to_dict(self, /) -> dict:
        """
        Convert to dictionary

        Example
        ----------

        ```python
        from triple_s import Variant

        content = dict(
            value = "test value",
            mode = "interview",
            language = "tr"
        )
        variant = Variant.from_dict(content)

        output = variant.to_dict()
        print(output)  # {'value': 'test value', 'mode': 'interview', 'language': 'tr'}
        ```
        """
    @property
    def value(self, /) -> str: ...
    @value.setter
    def value(self, /, value: str) -> None: ...

@final
class Version:
    """
    All available Triple-S file versions
    """
    V1_0: Final[Version]
    """
    1.0 (triple-s classic)
    """
    V1_1: Final[Version]
    """
    1.1
    """
    V1_2: Final[Version]
    """
    1.2, this could either be classic or xml
    """
    V2_0: Final[Version]
    """
    2.0
    """
    V3_0: Final[Version]
    """
    3.0
    """
    def __int__(self, /) -> int: ...
    def __repr__(self, /) -> str: ...
    @staticmethod
    def available_options() -> list[str]: ...

def read_datamap(path_datamap: "str") -> Datamap:
    """
    Read triple-s datamap

    Args:
        path_datamap (str): path to the triple-s datamap file, supports both classic and xml format.
    """

def read_triple_s(path_datamap: "str", path_data: "str" = None) -> Survey:
    """
    Read triple-s datamap, and load data afterwards.

    Args:
        path_datamap (str):
            path to the triple-s datamap file, supports both classic and xml format.     
        path_data (str, *optional*):
            path to the `.asc` or `.csv` data file.

    Returns:
        [`Survey`] containing `datamap` and `data`. If `path_data` is omitted,
        `has_data()` returns `False`.
    """

def __getattr__(name: str) -> Incomplete: ...
