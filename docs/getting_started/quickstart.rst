Quickstart
==========

Read a datamap
-----------------

Import the package as ``triple_s``. Use :func:`triple_s.read_datamap`
to load a Triple-S Classic or XML datamap without loading respondent data.
Replace the example paths with your own files.

.. code-block:: python

   from triple_s import read_datamap

   survey = read_datamap("survey.sss")
   print(survey.title)
   print(survey.survey_info.version)

Inspect and edit a question
---------------------------------

Look up a question by name with ``get_question()``. Names are case
insensitive, and a missing question returns ``None``.

.. code-block:: python

   question = survey.get_question("Q1")
   if question is not None:
       print(question.name, question.label)
       question.label = "Updated question label"

Question objects returned by this lookup share their fields with the survey,
so edits are included when you export the datamap.

Load respondent data
--------------------------

Use the datamap to interpret its matching data file, then convert the loaded
data into a `polars.DataFrame`, or a `pandas.DataFrame`:

.. code-block:: python

   survey.load_data("survey.asc")
   # to polars dataframe
   df = survey.data.to_polars()
   # to pandas dataframe
   pandas_df = survey.data.to_pandas()

``load_data()`` selects fixed-width or CSV format from the ``.asc`` or ``.csv``
extension and updates the record's ``href`` and ``data_format`` after loading.
For CSV data, positions refer
to one-based CSV field numbers. The record's ``skip`` setting controls
how many initial records are skipped, including a header if present.

To check a file without loading it or changing metadata, use
``survey.record.validate_data("survey.asc")``. Assigning ``record.href``
only edits metadata. ``validate_data()`` replaces the former
``update_data_source()`` method; use ``load_data()`` when you want to switch
the loaded data.

Alternatively, :func:`triple_s.read_triple_s` reads the datamap as well as the data file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")

Export a datamap
-------------------

Serialise the current survey to XML and save the returned string:

.. code-block:: python

   from pathlib import Path

   xml = survey.to_xml()
   Path("updated.sss").write_text(xml, encoding="utf-8")

``survey.to_classic()`` returns a Triple-S Classic string. It raises
``ValueError`` for features that Classic cannot represent, such as date
questions. These exports contain the datamap, not the respondent data.
