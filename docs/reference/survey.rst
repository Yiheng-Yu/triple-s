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

.. raw:: html

   <div class="separator">Content access</div>

.. automethod:: Survey.has_data() -> bool

.. automethod:: Survey.response_label_map(output: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame


.. raw:: html

   <div class="separator">Iteration & indexing</div>

.. py:method:: Survey.__iter__() -> Iterator[dict[str, object]]

   Iterate each response row by row, same as :meth:`Survey.iter_rows`.

.. py:method:: Survey.__getitem__(index: str | int) -> QuestionData | dict[str, object]

   Accept a question name to return its :class:`QuestionData`, or an integer
   row index to return that response as a dictionary.

.. automethod:: Survey.iter_rows() -> Iterator[dict[str, object]]

.. automethod:: Survey.iter_questions() -> Iterator[QuestionData]

.. raw:: html

   <div class="separator">Exporting</div>

.. automethod:: Survey.to_polars(response_label: bool | None = None) -> polars.DataFrame

.. automethod:: Survey.to_pandas(response_label: bool | None = None) -> pandas.DataFrame

.. raw:: html

   <div class="separator">Analysing data</div>

.. automethod:: Survey.crosstab(row: str, columns: str | list[str] | None = None, show_row_total: bool | None = None, show_column_total: bool | None = None, colname_stats: str | None = 'total', colname_label: str | None = 'label', colname_code: str | None = 'code', include_response_codes: bool | None = False, include_question_name: bool | None = False, output: str | triple_s.config.DataFrameFormat | None = None, normalise: bool | None = False) -> polars.DataFrame | pandas.DataFrame

.. automethod:: Survey.quick_summary(columns: str | list[str] | None = None, ignore: list[str] | None = None, normalise: bool = False, skip_text: bool = True, skip_numeric: bool = True, skip_datetime: bool = True, skip_single: bool = False, skip_multiple: bool = False, colname_stats: str = 'total', colname_label: str = 'label', colname_code: str = 'code', include_response_codes: bool = False, include_question_name: bool = False, output: str | triple_s.config.DataFrameFormat | None = None) -> list[tuple[str, polars.DataFrame | pandas.DataFrame]]