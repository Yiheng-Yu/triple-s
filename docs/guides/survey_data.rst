SurveyData
==========

:class:`triple_s.SurveyData` holds the respondent data loaded using a datamap.
When working with a :doc:`Survey <survey>`, access it through ``survey.data``.

Overview
---------------

Use ``describe()`` for a summary of each column:

.. code-block:: python

   from triple_s import read_triple_s
   survey = read_triple_s("path/to/survey.sss", "path/to/data.asc")
   survey_data = survey.data
   survey_data.describe()


Getting data of a question
----------------------------

Use the survey to get a question's answers by name:

.. code-block:: python

   >>> question_data = survey["Q1.b"]
   >>> print(question_data)

   QuestionData({
      name: "Q1.b"
      type: "time"
      data: [11:20:00, 13:43:00, 18:05:00]
   })

Export to a dataframe
-------------------------------

Polars
~~~~~~~~~

Convert the loaded data to a ``polars.DataFrame``:

.. code-block:: python

   df = survey_data.to_polars()

Pandas
~~~~~~~~~~

Or to a ``pandas.DataFrame``:

.. code-block:: python

   pandas_df = survey_data.to_pandas()


Iteration and data replacement
-------------------------------

Iterate raw respondent as dictionaries with ``iter(survey_data)``, this is the same as ``SurvreyData.iter_rows()``:

.. code-block:: python

   for response in survey_data:
       print(response)



Caching
---------

Loading a response file creates a temporary Parquet cache, hence it's pretty fast and memory efficient (i think)!
