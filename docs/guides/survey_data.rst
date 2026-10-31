SurveyData
==========

:class:`triple_s.SurveyData` holds the respondent data loaded using a datamap.
When working with a :doc:`Survey <survey>`, access it through ``survey.data``.

Overview
---------------

Use `describe` to get a quick overview of the data:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("path/to/survey.sss", "path/to/data.asc")
   survey_data = survey.data
   survey_data.describe()


Getting data of a question
----------------------------

Getting data of a quastion via indexing with name of the question:

.. code-block:: python

   question_data = survey_data["Q1.a"]
   print(question_data)

An example output:

.. code-block:: python

   QuestionData({
      name: "Q1.b"
      type: "time"
      data: [11:20:00, 13:43:00, 18:05:00]
   })

Export to a dataframe
-------------------------------

Polars
~~~~~~~~~

Convert the loaded data to a ``polars.DataFrame`` via `to_polars()`

.. code-block:: python

   df = data.to_polars()

Pandas
~~~~~~~~~~

Convert to pandas dataframe:

.. code-block:: python

   pandas_df = data.to_pandas()


Caching
-------

When you load a data file, ``triple-s`` saves the parsed responses in a
temporary Parquet file behind the scenes.
