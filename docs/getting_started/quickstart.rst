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
   # to ploars dataframe
   df = survey.to_polars()
   # to pandas dataframe
   pandas_df = survey.to_pandas()

Or, selection question you'd like to export, and export the data to pandas/ polars `Series`:

.. code-block:: python
   # Or, export to polars series
   polars_series = survey["Q1"].to_polars()

   # export to pandas series
   pandas_series = survey["Q1"].to_pandas()

For CSV data, the datamap's record must specify ``format="csv"``;
pass the matching ``.csv`` path to ``load_data()``. Positions then refer
to one-based CSV field numbers. The record's ``skip`` setting controls
how many initial records are skipped, including a header if present.

Alternatively, :func:`triple_s.read_triple_s` loads both files in one call:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc", read_triple_s=True)

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
