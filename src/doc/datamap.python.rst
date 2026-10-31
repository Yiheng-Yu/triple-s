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

Use ``Datamap.from_dict()`` to construct a datamap. The optional ``info``
(or ``survey_info``) and ``record`` entries accept their corresponding
objects or dictionaries accepted by ``SurveyInfo.from_dict()`` and
``Record.from_dict()``. An omitted or ``None`` metadata entry defaults to
Triple-S version 1.2; an omitted or ``None`` record creates an empty record.
When supplying a metadata dictionary, include its ``version``.

.. code-block:: python

   from triple_s import Datamap

   datamap = Datamap.from_dict({
       "info": {"version": "2.0", "name": "example", "title": "Example survey"},
       "record": {"ident": "A"},
   })
   datamap.info.name = "updated-example"
   datamap.title = "Updated survey"
   assert datamap.info.title == "Updated survey"

Metadata obtained through ``datamap.info`` shares its fields with the
original datamap. Editing those fields updates the datamap, including when
it is accessed through ``survey.datamap`` or ``survey.info``.

Access questions
----------------

The ``record`` property exposes the datamap's :class:`triple_s.Record`.
The ``questions`` property returns a list of question names in record order.
Use ``get_question()`` to look up a question by name. Names are case
insensitive; a missing question returns ``None``.

.. code-block:: python

   datamap = read_datamap("survey.sss")
   print(datamap.questions)
   question = datamap.get_question("Q1")
   if question is not None:
       print(question.label)

Load and export data
--------------------

``load_data()`` uses the record's question definitions and ``skip`` setting.
The file extension selects the format: ``.asc`` for fixed-width data or
``.csv`` for CSV data. CSV fields are selected by the questions' locations,
not by column names; use the record's skip setting to skip a header line.

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
