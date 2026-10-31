Datamap
=======

:class:`triple_s.Datamap` contains survey metadata in ``info`` and question
definitions in ``record``. It describes how to interpret the data file.

Getting datamap of a survey
***************************

1) From :attr:`triple_s.Survey.datamap`
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Check survey datamap via attribute :attr:`triple_s.Survey.datamap`:

.. code-block:: python

   from triple_s import read_triple_s
   survey = read_triple_s(
      "path/to/survey.sss",
      path_data = None
      )
   datamap = survey.datamap
   print(datamap)


2) Use :func:`triple_s.read_datamap`
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Alternatively, to read just the datamap, use :func:`triple_s.read_datamap` function:

.. code-block:: python

   from triple_s import read_datamap
   datamap = read_datamap("survey.sss")
   print(datamap.title)
   print(datamap.info.version)

Inspect and edit a question
~~~~~~~~~~~~~~~~~~~~~~~~~~~

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


See :doc:`survey_data` to load respondent data using this definition.
