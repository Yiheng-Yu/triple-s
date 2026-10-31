SurveyData
==========

:class:`triple_s.SurveyData` holds the respondent data loaded using a datamap.
When working with a :doc:`Survey <overview>`, access it through ``survey.data``.

Export respondent data
-------------------------------

Use the datamap to interpret its matching data file, then convert the loaded
data into a ``polars.DataFrame`` or a ``pandas.DataFrame``:

.. code-block:: python

   from triple_s import read_datamap

   datamap = read_datamap("survey.sss")
   data = datamap.load_data("survey.asc")
   # to polars dataframe
   df = data.to_polars()
   # to pandas dataframe
   pandas_df = data.to_pandas()

``load_data()`` selects fixed-width or CSV format from the ``.asc`` or ``.csv``
extension and updates the record's ``href`` and ``data_format`` after loading.
For CSV data, positions refer
to one-based CSV field numbers. The record's ``skip`` setting controls
how many initial records are skipped, including a header if present.

Caching
-------

When you load a data file, ``triple-s`` saves the parsed responses in a
temporary Parquet file behind the scenes.
