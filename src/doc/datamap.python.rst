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

Create new datamap
------------------------

Alternatively, you can build a datamap with ``Datamap.from_dict()``:

.. code-block:: python

   from triple_s import Datamap

   datamap = Datamap.from_dict({
       "info": {"version": "2.0", "name": "example", "title": "Example survey"},
       "record": {"ident": "A"},
   })
   datamap.info.name = "updated-example"
   datamap.title = "Updated survey"
   assert datamap.info.title == "Updated survey"


Access questions
----------------

To see out all questions in a datamap, use :py:property:`Record.questions`, which lists question names in record order.

Use ``get()`` on the datamap or record to get questions by its `name`, case insensitive; a missing
question returns ``None``:

.. code-block:: python

   datamap = read_datamap("survey.sss")
   print(datamap.questions)
   question = datamap.get("Q1")
   if question is not None:
       print(question.label)

Load and export data
--------------------

``load_data()`` uses the record's question definitions.

*Loading and validation support UTF-8 and BOM-marked UTF-16; invalid sequences become ``�`` (U+FFFD), each occupying one fixed-width position.*

.. code-block:: python

   data = datamap.load_data("survey.asc")
   df = data.to_polars()
   pandas_df = data.to_pandas()

Loading returns a :class:`triple_s.SurveyData` instance, 
it updates the datamap record's ``href`` to to the path of the data file, as well as ``data_format`` to the type of data file. 

*To read the definition and respondent data together, use :func:`triple_s.read_triple_s`:*


Export the datamap
------------------

``to_classic()`` returns Triple-S classic document

.. code-block:: python

   classic = datamap.to_classic()


``to_xml()`` returns a complete Triple-S XML document:

.. code-block:: python

   xml = datamap.to_xml()

