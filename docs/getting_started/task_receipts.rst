Task Receipts
==================

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


Convert between Triple-s Classic and XML
-------------------------------------------------

You can easily convert between triple-s classic/ triple-s xml via 
:func:`Datamap.to_classic` and :func:`Datamap.to_xml`:

.. code-block:: python

    from triple_s import read_datamap
    datamap = read_datamap("triple-s_xml.sss")

    # or .to_classic() to conver to triple-s classic
    content = datamap.to_xml()  

And write to a file:

.. code-block:: python

    with open("converted.sss") as file:
        file.write(content)


Save survey data to a csv file
-------------------------------------------------

You can convert survey data to a `pandas.DataFrame` or a `polars.DataFrame` instance,
and then call `to_csv` to export to a csv file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   df = survey.data.to_pandas()  # or to_polars() to get polars dataframe
   df.to_csv("csv_file.csv")  # or df.write_csv() for polars dataframe



