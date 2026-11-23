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

   Copied from the parent survey when this view is created.

.. autoattribute:: QuestionData.response_display
   :annotation: : str

   Copied from the parent survey when this view is created.

.. autoattribute:: QuestionData.response_label
   :annotation: : dict[int | float | str, Text]

Methods
-------

.. raw:: html

   <div class="separator">Iteration & indexing</div>

.. py:method:: QuestionData.__iter__() -> Iterator[object]

   Iterate over the stored question data values.

.. py:method:: QuestionData.__getitem__(index: int) -> object

   Return the value at an integer index. Negative indices are supported.

.. raw:: html

   <div class="separator">Exporting</div>

.. automethod:: QuestionData.to_polars(response_label: bool | None = None) -> polars.Series

.. automethod:: QuestionData.to_pandas(response_label: bool | None = None) -> pandas.Series

.. automethod:: QuestionData.to_list(response_label: bool | None = None) -> list[object]

.. raw:: html

   <div class="separator">Summarise</div>

.. py:method:: QuestionData.value_counts(colname_stats: str | None = 'total', colname_label: str | None = 'label', colname_code: str | None = 'code', include_response_codes: bool | None = False, include_question_name: bool | None = False, output: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame
   Examples:

      To include response codes and give the count column a name:

      .. code-block:: python

         counts = survey["Q2"].value_counts(
            include_response_codes=True, colname_stats="answers",
         )
         print(counts)
