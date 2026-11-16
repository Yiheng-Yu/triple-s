Survey
======

.. currentmodule:: triple_s

.. autoclass:: triple_s.Survey

Use :func:`triple_s.read_triple_s` to read a datamap and optional respondent
file together. ``info`` shares metadata with ``datamap.info``; changing either
view updates the survey. ``title`` is also writable.

Attributes
----------

.. autoattribute:: Survey.name
   :annotation: : str | None

.. autoattribute:: Survey.title
   :annotation: : str | None

.. autoattribute:: Survey.datamap
   :annotation: : Datamap

.. autoattribute:: Survey.info
   :annotation: : SurveyInfo

.. autoattribute:: Survey.data
   :annotation: : SurveyData

.. autoattribute:: Survey.dataframe_format
   :annotation: : str

.. autoattribute:: Survey.response_display
   :annotation: : str
Properties
-----------

.. autoattribute:: Survey.question_names
   :annotation: : list[str]

   List of question names.

.. autoattribute:: Survey.num_response
   :annotation: : int

   Total number of respondent in a Survey


Methods
-------

.. automethod:: Survey.has_data() -> bool

.. automethod:: Survey.response_label_map(format: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame

.. automethod:: Survey.iter_rows() -> Iterator[dict[str, object]]

.. automethod:: Survey.iter_questions() -> Iterator[QuestionData]

.. automethod:: Survey.to_polars(response_label: bool | None = None) -> polars.DataFrame

.. automethod:: Survey.to_pandas(response_label: bool | None = None) -> pandas.DataFrame

.. automethod:: Survey.crosstab(row: str, column: str, format: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame

