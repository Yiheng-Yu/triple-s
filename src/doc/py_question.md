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
