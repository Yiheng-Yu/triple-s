# Question

Represents questions in a triple-s datamap file, this corresponds to the `<variable>` tag in triple-s xml.

Attributes:
name (str):
    name of the question

question_type (triple_s.QuestionType | str | None):
    type of the question, corresponds to the `<type>` child node of a `<variable>` node.
    one of the following:
    ["single", "multiple", "logical", "quantity", "character", "date", "time"]
    Pass `None` for a question with no response values, for example `Question("n1", None)`.

label (triple_s.Text, *Optional*):
    Question text label, corresponds to the `<label>` child node of a `<variable>` node.
    default `Text(value="", variants=[])`

filter (str|None, *Optional*):
    Question filter, corresponds to the `<filter>` child node.
    default `None`

format (str|triple_s.CodeFormat):
    This corresponds to the `format` attribute in a `<variable>` tag.
    Flag for indicating the data type of `code` attribute in a `<value>` node.
    See `help(triple_s.CodeFormat`) for more details.
    Default `None`.

usage (str|triple_s.Usage, *Optional*)
    This corresponds to the `use` attribute in a `<variable>`.
    See `help(triple_s.Usage`) for more details.
    Default `None`.

location (triple_s.Location):
    Corresponds to the `<position>` child node in a `<variable>`.
    See `help(triple_s.Location`) for more details.
    Default Location(start=0, finish=0)

spread (triple_s.Spread, *Optional*):
    Used only when `question_type` is 'multiple'.
    Corresponds to the `<spread>` child node in a `<variable>`.
    See `help(triple_s.Spread`) for more details.

Respondent data is stored by `Record`. Use `Survey.to_polars()` or `Survey.to_pandas()` to export it.
