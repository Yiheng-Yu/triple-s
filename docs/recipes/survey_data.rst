Survey Data
===========================


Get quick summaries for questions in a survey
----------------------------------------------

You can use :meth:`~triple_s.QuestionData.value_counts` to get a quick
summary for a survey question:

.. code-block:: python

   counts = survey["Q2"].value_counts(name="Count")
   print(counts.to_dicts())


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