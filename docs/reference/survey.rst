Survey
======

.. autoclass:: triple_s.Survey

Use :func:`triple_s.read_triple_s` to read a datamap and optional respondent
file together. ``info`` shares metadata with ``datamap.info``; changing either
view updates the survey. ``title`` is also writable.

Attributes
----------

.. autoattribute:: Survey.datamap
   :annotation: -> Datamap

.. autoattribute:: Survey.info
   :annotation: -> SurveyInfo

.. autoattribute:: Survey.title
   :annotation: -> str | None

.. autoattribute:: Survey.data
   :annotation: -> SurveyData

.. autoattribute:: Survey.dataframe_format
   :annotation: -> str

.. autoattribute:: Survey.response_display
   :annotation: -> str

Methods
-------

.. automethod:: Survey.has_data() -> bool
.. automethod:: Survey.iter_rows() -> Generator[dict[str, Any]]
.. automethod:: Survey.iter_questions() -> Generator[QuestionData]
