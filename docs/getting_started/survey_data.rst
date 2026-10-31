SurveyData
==========

:class:`triple_s.SurveyData` holds the respondent data loaded using a datamap.
When working with a :doc:`Survey <survey>`, access it through ``survey.data``.

Overview
---------------

Use `describe` to get a quick overview of the data:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("path/to/survey.sss", "path/to/data.asc")
   data = survey.data
   data.describe()

Work with Polars
-------------------------------

Convert the loaded data into a ``polars.DataFrame`` via `to_polars()`

.. code-block:: python

   df = data.to_polars()

Work with Pandas
-----------------------

Convert into pandas dataframe:

.. code-block:: python

   pandas_df = data.to_pandas()


Caching
-------

When you load a data file, ``triple-s`` saves the parsed responses in a
temporary Parquet file behind the scenes.
