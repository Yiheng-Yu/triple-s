Survey Data
===========================


Get quick summaries for questions in a survey
----------------------------------------------

You can use :meth:`~triple_s.QuestionData.value_counts` to count the
answers for a survey question. Suppose ``Q2`` asks "Do you cycle to work?",
with codes ``1`` for "Yes" and ``2`` for "No", and the answers are
``[1, 1, 2, None]``:

.. code-block:: python

   counts = survey["Q2"].value_counts()
   print(counts.to_dicts())

.. code-block:: text

   [{'label': 'Total', 'total': 3},
    {'label': 'Yes', 'total': 2},
    {'label': 'No', 'total': 1}]

The first row counts the three respondents who answered. Missing answers
are excluded. Labels are shown even if ``survey.response_display`` is
``"value"``.

You can include the question name and response codes, and rename the columns:

.. code-block:: python

   counts = survey["Q2"].value_counts(
       include_question_name=True,
       include_code_value=True,
       colname_stats="answers",
       colname_label="answer",
       colname_code="value",
   )
   print(counts.to_dicts())

.. code-block:: text

   [{'question': 'Q2', 'value': None, 'answer': 'Total', 'answers': 3},
    {'question': None, 'value': 1, 'answer': 'Yes', 'answers': 2},
    {'question': None, 'value': 2, 'answer': 'No', 'answers': 1}]

The question name appears only in the first row. You can fill the remaining
rows with ``counts.with_columns(pl.col("question").forward_fill())``, using
``import polars as pl``, if you want the name on every row.

For a multiple-choice question, each selected response is counted separately.
For example, if ``Q3`` asks which ways people travel, with ``1`` for "Bus"
and ``2`` for "Bike", answers ``[[1, 2], [2], [], None]`` give you:

.. code-block:: python

   print(survey["Q3"].value_counts().to_dicts())

.. code-block:: text

   [{'label': 'Total', 'total': 3},
    {'label': 'Bus', 'total': 1},
    {'label': 'Bike', 'total': 2}]

The empty list counts towards the respondent total, but adds no selections.

The result uses the question's inherited dataframe format by default.
You can choose a different format for this call with ``output``:

.. code-block:: python

   pandas_counts = survey["Q2"].value_counts(output="pandas")

You can also pass ``config.DataFrameFormat.Pandas`` or
``config.DataFrameFormat.Polars``, using ``from triple_s import config``.


Get cross tabs for a survey question
------------------------------------

Suppose you want to look at ``Q2`` by ``Q4``. You can simply pass the row
question and a list of column questions to ``crosstab()``:

.. code-block:: python

   table = survey.crosstab("Q2", ["Q4"])
   print(table)

Or, to look at several questions at once:

.. code-block:: python

   combined = survey.crosstab("Q2", ["Q4", "Q5"])


Both totals are shown by default. You can switch them on or off separately:

.. code-block:: python

   # Keep the row-total column, but hide the first totals row.
   table = survey.crosstab(
       "Q2", ["Q4"], show_row_total=True, show_column_total=False,
   )

   # Keep the first totals row, but hide the row-total column.
   table = survey.crosstab(
       "Q2", ["Q4"], show_row_total=False, show_column_total=True,
   )

If you want a pandas dataframe instead, you can set the output format for
this call:

.. code-block:: python

   pandas_table = survey.crosstab("Q2", ["Q4"], format="pandas")


Working with Hugging Face Datasets
-------------------------------------------------

A. Export to parquet
*********************

Export the responses to a Parquet file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   survey.data.to_polars().write_parquet("survey.parquet")


Load it with `Hugging Face Datasets <https://huggingface.co/docs/datasets/index>`_:

.. code-block:: python

    from datasets import load_dataset
    dataset = load_dataset("parquet", data_files="survey.parquet")

B. Iterate rows directly
*************************

Alternatively, you can simply use `iter_rows` to create a generator that can pass to `Dataset.from_generator` directly.
Behind the scene, `SurveyData` uses a `lazyframe` that streams data from a `parquet` file, stored either in a temporary cache
or a parquet file provided by the user.

.. code-block:: python

    from datasets import Dataset
    from triple_s import read_triple_s

    def generate_responses():
        survey = read_triple_s("survey.sss", "survey.asc")
        yield from survey.iter_rows()

    dataset = Dataset.from_generator(generate_responses)

If you want it to return class labels instead of indicies (i.e., `'London'` instead of class index `1`), you can do so by
setting `response_display` attribute to `label`:

.. code-block:: python

    survey.response_display = "label"
    dataset = Dataset.from_generator(survey.data.iter_rows)