SurveyData
==========

:class:`triple_s.SurveyData` holds the respondent data loaded using a datamap.
When working with a :doc:`survey`, access it through ``survey.data``.

Load and export respondent data
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

``datamap.record.load_data("survey.asc")`` also returns a
:class:`triple_s.SurveyData` when working directly with a record.

``load_data()`` selects fixed-width or CSV format from the ``.asc`` or ``.csv``
extension and updates the record's ``href`` and ``data_format`` after loading.
For CSV data, positions refer
to one-based CSV field numbers. The record's ``skip`` setting controls
how many initial records are skipped, including a header if present.

To check a file without loading it or changing metadata, use
``datamap.record.validate_data("survey.asc")``. Assigning ``record.href``
only edits metadata. ``load_data()`` returns a :class:`triple_s.SurveyData`;
keep this object to export the loaded responses.

Loading data through ``survey.datamap.load_data()`` returns a separate
``SurveyData`` object and does not replace ``survey.data``.

See :doc:`survey` for indexing and iterating through respondents.
