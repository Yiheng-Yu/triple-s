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

.. py:method:: QuestionData.value_counts(colname_stats: str | None = 'total', colname_label: str | None = 'label', colname_code: str | None = 'code', include_code_value: bool | None = False, include_question_name: bool | None = False, output: str | triple_s.config.DataFrameFormat | None = None) -> polars.DataFrame | pandas.DataFrame

   Count the answers for this question.

   By default, this gives you a ``label`` column and a ``total`` column.
   The first row, labelled ``Total``, counts respondents with non-null answers.
   The remaining rows show observed answers in response-value order. For
   multiple-choice questions, each selected response is counted separately;
   empty lists count towards the first row but add no response counts.

   :param include_question_name: Include a ``question`` column with the question name
      in the first row and nulls in the remaining rows. Default ``False``.
   :param include_code_value: Include the original response values in a separate column.
      The total row has a null code value. Default ``False``.
   :param colname_stats: Name of the count column. Default ``'total'``.
   :param colname_label: Name of the response-label column. Default ``'label'``.
   :param colname_code: Name of the response-value column, used when
      ``include_code_value=True``. Default ``'code'``.
   :param output: ``'polars'`` or ``DataFrameFormat.Polars`` for a Polars
      dataframe; ``'pandas'`` or ``DataFrameFormat.Pandas`` for a pandas
      dataframe. Default ``None`` uses the question's inherited
      ``dataframe_format`` setting.
   :returns: A Polars or pandas dataframe with response labels and counts.

   You can use ``None`` for any option to use its default. Response labels
   are shown regardless of the survey's ``response_display`` setting; values
   without a label are shown as strings.

   To include response codes and give the count column a name:

   .. code-block:: python

      counts = survey["Q2"].value_counts(
          include_code_value=True, colname_stats="answers",
      )
      print(counts)

   See :doc:`../recipes/survey_data` for examples with output.

.. automethod:: QuestionData.to_polars(response_label: bool | None = None) -> polars.Series

.. automethod:: QuestionData.to_pandas(response_label: bool | None = None) -> pandas.Series

.. automethod:: QuestionData.to_list(response_label: bool | None = None) -> list[object]

