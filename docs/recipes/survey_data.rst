Survey Data
===========================

Working with Huggingface Datasets
-------------------------------------------------

Call `to_parquet()` to dump data to a parquet database to be used later:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   survey.data.to_parquet("survey.parquet")


This allows you then create `Huggingface datasets <https://huggingface.co/docs/datasets/index>`_ easily from the exported parquet database:

.. code-block:: python

    from datasets import load_dataset
    dataset = load_dataset("parquet", data_files="survey.parquet")

Survey data is cached to a temporary file and loaded to the memory on-demand, 
hence you can you can create new `Dataset <https://huggingface.co/docs/datasets/package_reference/main_classes#datasets.Dataset>`_/ `IterableDataset <https://huggingface.co/docs/datasets/package_reference/main_classes#datasets.IterableDataset>`_ instance 
from survey instances' `__iter__` function without 
worrying the memory footprints:

.. code-block:: python

    from datasets import Dataset

    dataset = Dataset.from_generator(survey.data.__iter__) 

And then do whatever you need from here.