Datamap
=======

:class:`triple_s.Datamap` contains survey metadata in ``info`` and question
definitions in ``record``. It describes how to interpret the data file.

To read just the datamap, use :func:`triple_s.read_datamap` function:

.. code-block:: python

   from triple_s import read_datamap
   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.version)


Conceptually, a survey datamap consists of two parts: a :class:`SurveyInfo` that contains survey 
metadata (name, title, creation date, etc.,) and a :class:`Record` that collects survey questions.

Overview
----------------------------

Survey metadata
*********************

Survey metadata can be accessed by :py:attr:`Datamap.info`:

.. code-block:: python

   info = datamap.info
   print(info)


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

Survey records
*********************

Survey questions are collated under :py:attr:`Datamap.record` attribute:

.. code-block:: python

   record = datamap.record
   print(record)


Check questions in a survey
----------------------------------

Check each question by iterate the :py:attr:`Datamap.record`:

.. code-block:: python
   
   for question in datamap.record:
      # do someting..

For getting specific questions in the datamap instance, you can do so by:

A. via question name:

.. code-block:: python

   question = datamap["Q1"]
   print(question.name, question.label)

Use ``get_question()`` to get datamap of a specific question:

.. code-block:: python

   question = datamap.get_question("Q1")
   print(question.name, question.label)


B. via question index:

Just like indexing of any python list, you can do:

.. code-block:: python

   question = datamap[0]
   print(question.name, question.label)


Update Datamap
---------------------

You can simply update contents of any question like this:

.. code-block:: python

   datamap["Q1"].label = "Updated question label"

      


Convert between Triple-s Classic and XML
-----------------------------------------------

You can easily convert between triple-s classic/ triple-s xml via 
:func:`Datamap.to_classic` and :func:`Datamap.to_xml` function call:

.. code-block:: python

   from triple_s import read_datamap
   datamap = read_datamap("triple-s_xml.sss")

   with open("converted.sss") as file:
      file.write(datamap.to_classic())


The :func:`Datamap.to_xml` function returns a string that can be read by any xml parser, 
such as the `lxml <https://lxml.de/>`_ library:

.. code-block:: python

   from lxml import etree
   from triple_s import read_datamap

   datamap = read_datamap("triple-s_xml.sss")
   xml_string = datamap.to_xml()

   root = etree.fromstring(xml_string.encode("utf-8"))
