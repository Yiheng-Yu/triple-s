The root of a Triple-S datamap, containing survey metadata and a record of
questions.

Read a survey
-------------

Load an existing datamap with :func:`triple_s.read_datamap`. Both Triple-S
Classic and XML datamaps are supported.

.. code-block:: python

   from triple_s import read_datamap

   survey = read_datamap("survey.sss")
   print(survey.title)
   print(survey.survey_info.name)

Access questions
----------------

Use ``get_question()`` to look up a question by name. Names are case
insensitive; a missing question returns ``None``.

.. code-block:: python

   question = survey.get_question("Q1")
   if question is not None:
       print(question.label)

The ``record`` property exposes the survey's :class:`triple_s.Record`;
``survey_info`` exposes its :class:`triple_s.SurveyInfo` metadata.

Load and export data
--------------------

Load respondent data using the record's format and skip settings, then
export it to a Polars or pandas DataFrame.

.. code-block:: python

   survey.load_data("survey.asc")
   df = survey.to_polars()
   pandas_df = survey.to_pandas()

For a datamap whose record uses CSV format, pass the corresponding CSV
file to ``load_data()`` instead.

Export the datamap
------------------

.. code-block:: python

   classic = survey.to_classic()
   xml = survey.to_xml()

These methods return datamap strings; they do not write files or export
respondent data.
``to_classic()`` raises ``ValueError`` if the survey uses features that
Triple-S Classic cannot represent, such as date questions.
