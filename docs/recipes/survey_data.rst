Survey Data
===========================


Get quick summaries for quesitions in a survey
----------------------------------------------

Use `value_counts()` to get a quick summary for a survey question:

.. code-block:: python

    >>> survey["Q1"].value_counts(name="Count")


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

    dataset = Dataset.from_generator(survey.data.iter_rows)

If you want it to return class labels instead of indicies (i.e., `'London'` instead of class index `1`), you can do so by
setting `response_display` attribute to `label`:

.. code-block:: python

    survey.response_display = "label"

    dataset = Dataset.from_generator(survey.data.iter_rows)