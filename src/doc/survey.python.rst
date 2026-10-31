The root of a Triple-S datamap, containing survey metadata and a record of
questions.

Read a datamap
----------------

Load an existing datamap with :func:`triple_s.read_datamap`. Both Triple-S
Classic and XML datamaps are supported.

.. code-block:: python

   from triple_s import read_datamap

   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.name)

Access questions
-------------------

Use ``get_question()`` to look up a question by name. Names are case
insensitive; a missing question returns ``None``.

.. code-block:: python

   question = datamap.get_question("Q1")
   if question is not None:
       print(question.label)

The ``record`` property exposes the survey's :class:`triple_s.Record`;
``info`` exposes its :class:`triple_s.SurveyInfo` metadata.

Load and export data
--------------------------

Load respondent data using the file extension and the record's skip setting, then
export it to a Polars or pandas DataFrame.

.. code-block:: python

   data = datamap.load_data("survey.asc")
   df = data.to_polars()
   pandas_df = data.to_pandas()

``load_data()`` returns a :class:`triple_s.SurveyData` and updates the record
metadata. It does not store responses on the datamap. Pass a ``.csv`` file
for CSV data or an ``.asc`` file for fixed-width data.

Export the datamap
---------------------

.. code-block:: python

   classic = datamap.to_classic()
   xml = datamap.to_xml()

These methods return datamap strings; they do not write files or export
respondent data.
``to_classic()`` raises ``ValueError`` if the survey uses features that
Triple-S Classic cannot represent, such as date questions.
