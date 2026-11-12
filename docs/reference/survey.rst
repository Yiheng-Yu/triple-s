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
   :annotation: -> str | None

.. autoattribute:: Survey.title
   :annotation: -> str | None

.. autoattribute:: Survey.datamap
   :annotation: -> Datamap

.. autoattribute:: Survey.info
   :annotation: -> SurveyInfo

.. autoattribute:: Survey.data
   :annotation: -> SurveyData

.. autoattribute:: Survey.dataframe_format
   :annotation: -> str

.. autoattribute:: Survey.response_display
   :annotation: -> str


Properties
-----------

.. :py:property:: Survey.question_names
   :type: list[str]

.. :py:property:: Survey.num_response
   :type: int

Methods
-------
.. automethod:: Survey.has_data() -> bool

.. autoattribute:: Survey.response_label_map
   :annotation: -> polars.DataFrame | pandas.DataFrame

.. automethod:: Survey.iter_rows() -> Generator[dict[str, Any]]

.. automethod:: Survey.iter_questions() -> Generator[QuestionData]

.. automethod:: Survey.to_polars() -> polars.DataFrame

.. automethod:: Survey.to_pandas() -> pandas.DataFrame
