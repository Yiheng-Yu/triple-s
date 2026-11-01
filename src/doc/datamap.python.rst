A Triple-S datamap containing survey metadata in :class:`triple_s.SurveyInfo`
and question definitions in :class:`triple_s.Record`.
Use :class:`triple_s.Survey` to work with a datamap and respondent data together.

Read a datamap
--------------

Load an existing datamap with :func:`triple_s.read_datamap`. Both Triple-S
Classic and XML datamaps are supported. This reads the definition only;
it does not load the data file referenced by the record.

.. code-block:: python

   from triple_s import read_datamap

   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.name)

Create and edit metadata
------------------------

Build a datamap with ``Datamap.from_dict()``. The optional ``info`` (or
``survey_info``) and ``record`` entries accept objects or dictionaries for
``SurveyInfo.from_dict()`` and ``Record.from_dict()``. Include ``version`` in
metadata dictionaries. Missing or ``None`` metadata uses Triple-S version 1.2;
a missing or ``None`` record starts empty.

.. code-block:: python

   from triple_s import Datamap

   datamap = Datamap.from_dict({
       "info": {"version": "2.0", "name": "example", "title": "Example survey"},
       "record": {"ident": "A"},
   })
   datamap.info.name = "updated-example"
   datamap.title = "Updated survey"
   assert datamap.info.title == "Updated survey"

Edits through ``datamap.info``, ``survey.datamap``, or ``survey.info`` update
the shared metadata.

Access questions
----------------

``questions`` lists question names in record order. Use ``get()`` on the
datamap or record to look up a name. Matching is case insensitive; a missing
question returns ``None``.

.. code-block:: python

   datamap = read_datamap("survey.sss")
   print(datamap.questions)
   question = datamap.get("Q1")
   if question is not None:
       print(question.label)

Load and export data
--------------------

``load_data()`` uses the record's question definitions. The extension selects
fixed-width (``.asc``) or CSV (``.csv``) parsing. CSV fields use the questions'
locations; set ``record.skip = "1"`` to skip a header line.
Loading and validation support UTF-8 and BOM-marked UTF-16. Invalid sequences
become ``�`` (U+FFFD), each occupying one fixed-width position.

.. code-block:: python

   data = datamap.load_data("survey.asc")
   df = data.to_polars()
   pandas_df = data.to_pandas()

Loading returns a :class:`triple_s.SurveyData` backed by a temporary Parquet
cache and updates the datamap record's ``href`` and ``data_format``. Keep the
returned ``data`` object alive while using its cache. Loading does not attach
the returned data to the datamap or populate an existing ``Survey.data``.

To read the definition and respondent data together, use
:func:`triple_s.read_triple_s`:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   df = survey.data.to_polars()

Omitting the data path creates a survey with empty data;
``survey.has_data()`` then returns ``False``.

Export the datamap
------------------

.. code-block:: python

   classic = datamap.to_classic()
   xml = datamap.to_xml()

These methods return strings containing the definition, not respondent
data, and do not write files. ``to_xml()`` returns a complete Triple-S XML
``<sss>`` document. ``to_classic()`` raises ``ValueError`` for unsupported
question types, including date and time questions.
