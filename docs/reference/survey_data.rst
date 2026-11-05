SurveyData
==========

.. autoclass:: triple_s.SurveyData

Attributes
----------

.. autoattribute:: SurveyData.questions
   :annotation: -> list[str]

.. autoattribute:: SurveyData.num_response
   :annotation: -> int

Methods
-------

.. automethod:: SurveyData.has_data() -> bool
.. automethod:: SurveyData.empty() -> SurveyData
.. automethod:: SurveyData.from_polars(frame) -> SurveyData
.. automethod:: SurveyData.to_polars() -> polars.DataFrame
.. automethod:: SurveyData.to_pandas() -> pandas.DataFrame
.. automethod:: SurveyData.describe() -> polars.DataFrame
