Datamap
=======

:class:`triple_s.Datamap` contains survey metadata in ``info`` and question
definitions in ``record``. It describes how to interpret the data file.

Create New Datamap
------------------------

From triple-s file
*********************

Read a datamap with :func:`triple_s.read_datamap`:

.. code-block:: python

   from triple_s import read_datamap
   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.version)


****

Build one from scartch
**************************

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


You can either pass your list of questions to the `record` child dictionary, 
or ues :func:`Datamap.record.add_question` and :func:`Datamap.record.remove_question` later too add/ 
remove survey questions:

.. code-block:: python

   datamap.add_question(q1)
   datamap.remove_question(q2)


Components
------------------------

Metadata
*********************

Access survey metadata through :py:attr:`Datamap.info`:

.. code-block:: python

   info = datamap.info
   print(info)

Returns:

.. code-block:: python
   :class: code-output

   SurveyInfo(
      version=Version.V2_0,
      name='SP5201-1',
      title='Historic House Exit Survey<br/>First Wave',
      date='14 April 2005',
      time='16:00',
      origin='SurveyProg v1.3.05',
      user='User Site',
      style=None,
      language_options=(),
      mode_options=(),
      standard=None,
   )

See :class:`triple_s.SurveyInfo` for the available attributes.

****

Records
*********************

The :py:attr:`Datamap.record` attribute holds the questions:

.. code-block:: python

   record = datamap.record
   print(record)

****

Check questions in a survey
----------------------------------

Iterate through :class:`Datamap` or :py:attr:`Datamap.record` to read each question:

.. code-block:: python
   
   for question in datamap:
      print(question.name, question.label)

Look up a question by name:

.. code-block:: python

   question = datamap["Q1"]  # or datamap.record["Q1"]
   print(question.name, question.label)

Use ``get()`` to return ``None`` when the name is missing:

.. code-block:: python

   question = datamap.get("Q1")
   if question is not None:
      print(question.name, question.label)


You can also use a zero-based index, as with a Python list:

.. code-block:: python

   question = datamap[0]  # or datamap.record[0]
   print(question.name, question.label)


Update Datamap
---------------------

Edit a question, then save the updated datamap:

.. code-block:: python

   from triple_s import read_datamap
   datamap = read_datamap("survey.sss")
   datamap["Q1"].label = "Updated question label"

   with open("updated survey.sss", "w", encoding="utf-8") as file:
      file.write(datamap.to_xml())


Export the datamap
------------------

``to_classic()`` returns Triple-S classic document:

.. code-block:: python

   classic = datamap.to_classic()


``to_xml()`` returns a complete Triple-S XML document:

.. code-block:: python

   xml = datamap.to_xml()

