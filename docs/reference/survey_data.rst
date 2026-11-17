SurveyData
==========

.. currentmodule:: triple_s

.. autoclass:: triple_s.SurveyData

Obtain data with :func:`read_data`, :meth:`SurveyData.empty`, or
:meth:`SurveyData.from_polars`. Direct construction is unsupported.

Properties
----------

.. autoattribute:: SurveyData.question_names
   :annotation: : list[str]

   List of question names in a SurveyData instance.

.. autoattribute:: SurveyData.num_response
   :annotation: : int

   Total number of respondents in a SurveyData instance.


Methods
-------

.. automethod:: SurveyData.has_data() -> bool

.. automethod:: SurveyData.empty() -> SurveyData

.. automethod:: SurveyData.from_polars(frame: polars.DataFrame) -> SurveyData

.. automethod:: SurveyData.to_polars() -> polars.DataFrame

.. automethod:: SurveyData.to_pandas() -> pandas.DataFrame

.. automethod:: SurveyData.describe() -> polars.DataFrame

