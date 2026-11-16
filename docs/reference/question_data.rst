QuestionData
============

.. currentmodule:: triple_s

.. autoclass:: triple_s.QuestionData

Attributes
----------

.. autoattribute:: QuestionData.name
   :annotation: : str

.. autoattribute:: QuestionData.type
   :annotation: : str | None

.. autoattribute:: QuestionData.label
   :annotation: : Text

.. autoattribute:: QuestionData.range
   :annotation: : DataRange | None

.. autoattribute:: QuestionData.datamap
   :annotation: : Question

   Shared question definition.

.. autoattribute:: QuestionData.data
   :annotation: : tuple[object, ...]

.. autoattribute:: QuestionData.dataframe_format
   :annotation: : str

   Inherited from the parent survey.

.. autoattribute:: QuestionData.response_display
   :annotation: : str

   Inherited from the parent survey.

.. autoattribute:: QuestionData.response_label
   :annotation: : dict[int | float | str, Text]

Methods
-------

.. automethod:: QuestionData.value_counts(name: str | None = 'count', format: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame

.. automethod:: QuestionData.to_polars(response_label: bool | None = None) -> polars.Series

.. automethod:: QuestionData.to_pandas(response_label: bool | None = None) -> pandas.Series

.. automethod:: QuestionData.to_list(response_label: bool | None = None) -> list[object]

