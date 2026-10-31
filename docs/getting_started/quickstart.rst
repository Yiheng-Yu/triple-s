Quickstart
==========

Load Survey
---------------------------

Load triple-s survey via :func:`read_triple_s`:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_datamap(
      "path/to/survey.sss", 
      path_data="path/to/data.asc"  # optional
      )

   print(survey)


Output:

.. code-block:: bash

   Survey({
      name: "SP5201-1"
      title: "Historic House Exit Survey<br/>First Wave"
      questions: ["RESPONDENT_ID", "Q1.a", "Q1.b", ...]
      num_response: 3
   })


Viewing response data
******************


Datamap
---------------------------

Getting datamap of a survey
******************

1) From :func::`Survey.datamap`:
~~~~~~~~~~~~~~~~~~~~~~~~~

Check survey datamap via attribute :func::`Survey.datamap`:

.. code-block:: python

   from triple_s import read_triple_s
   survey = read_datamap(
      "path/to/survey.sss",
      path_data = None
      )
   datamap = survey.datamap
   print(datamap)


2) Use :func:`triple_s.read_datamap`
~~~~~~~~~~~~~~~~~~~~~~~~~

Alternatively, to read just the datamap, use :func:`read_datamap` function:

.. code-block:: python

   from triple_s import read_datamap

   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.version)

Inspect and edit a question
~~~~~~~~~~~~~~~~~~~~~~~~~

Look up a question by name with ``get_question()``. Names are case
insensitive, and a missing question returns ``None``.

.. code-block:: python

   question = datamap.get_question("Q1")
   if question is not None:
       print(question.name, question.label)
       question.label = "Updated question label"

Question objects returned by this lookup share their fields with the datamap,
so edits are included when you export the datamap.

Export a datamap
~~~~~~~~~~~~~~~~~~~~~~~~~

Serialise the current datamap to XML and save the returned string:

.. code-block:: python

   from pathlib import Path

   xml = datamap.to_xml()
   Path("updated.sss").write_text(xml, encoding="utf-8")

``datamap.to_classic()`` returns a Triple-S Classic string. It raises
``ValueError`` for features that Classic cannot represent, such as date
questions. These exports contain the datamap, not the respondent data.


SurveyData
---------------------------

Use the datamap to interpret its matching data file, then convert the loaded
data into a `polars.DataFrame`, or a `pandas.DataFrame`:

.. code-block:: python

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

Alternatively, :func:`triple_s.read_triple_s` reads the datamap as well as the data file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   print(survey.info.name)
   print(survey.has_data())
   df = survey.data.to_polars()
   datamap = survey.datamap

Omitting the data path returns a survey with empty data. Loading data through
``survey.datamap.load_data()`` returns a separate ``SurveyData`` object and
does not replace ``survey.data``.

``survey.info`` and ``survey.datamap.info`` share metadata. For example,
assigning ``survey.title = "Updated title"`` also updates
``survey.datamap.title`` and ``survey.info.title``.

