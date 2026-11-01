SurveyData
==========

.. autoclass:: triple_s.SurveyData

Attributes
----------

.. autoattribute:: triple_s.SurveyData.questions
	:annotation: -> list[str]
.. autoattribute:: triple_s.SurveyData.num_response
	:annotation: -> int

Methods
-------

.. automethod:: triple_s.SurveyData.has_data() -> bool
.. automethod:: triple_s.SurveyData.empty() -> SurveyData
.. automethod:: triple_s.SurveyData.from_polars(frame) -> SurveyData
.. automethod:: triple_s.SurveyData.to_polars() -> Any
.. automethod:: triple_s.SurveyData.to_pandas() -> Any
.. automethod:: triple_s.SurveyData.describe() -> Any
