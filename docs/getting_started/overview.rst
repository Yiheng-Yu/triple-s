Quick Start
===========

A Triple-S survey usually consists of a datamap (``.sss``) describing the
questions and a matching data file (``.asc`` or ``.csv``) containing responses.
Import the installed package as ``triple_s``.


Read a survey
-------------

:func:`triple_s.read_triple_s` reads the datamap and its matching data file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   print(survey.info.name)
   print(survey.has_data())
   df = survey.data.to_polars()
   datamap = survey.datamap


Next steps
----------

Continue getting started with these guides:

* :doc:`Survey <survey>`
  Load a survey, access its metadata, and explore individual responses through indexing and iteration.

* :doc:`Datamap <datamap>` 
  Inspect question definitions, edit survey metadata and labels, and export the updated datamap.

* :doc:`SurveyData <survey_data>`
  Load respondent data using a datamap and convert it to Polars or pandas DataFrames.
