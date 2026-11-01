Survey Data
===========================

Working with Hugging Face Datasets
-------------------------------------------------

Export the responses to a Parquet file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   survey.data.to_polars().write_parquet("survey.parquet")


Load it with `Hugging Face Datasets <https://huggingface.co/docs/datasets/index>`_:

.. code-block:: python

    from datasets import load_dataset
    dataset = load_dataset("parquet", data_files="survey.parquet")

You can also create a dataset from response rows. Iterating through
``survey.data`` loads the survey data into memory:

.. code-block:: python

    from datasets import Dataset

    dataset = Dataset.from_generator(survey.data.__iter__)
