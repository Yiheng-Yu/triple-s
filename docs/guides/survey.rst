Survey
======

A Triple-S survey usually consists of a datamap (``.sss``) describing the
questions and a matching data file (``.asc`` or ``.csv``) containing responses.
Import the installed package as ``triple_s``.


Read triple-s files
----------------------

Load a survey with :func:`read_triple_s`:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s(
      "path/to/survey.sss",
      path_data="path/to/data.asc"  # optional
      )

   print(survey)

This outputs:

.. code-block:: python
   :class: code-output

   Survey({
      name: "SP5201-1"
      title: "Historic House Exit Survey<br/>First Wave"
      questions: ["RESPONDENT_ID", "Q1.a", "Q1.b", ...]
      num_response: 3
   })

View Survey Data
---------------------

Use a zero-based index to read one respondent's answers:

.. code-block:: python

   respondent_1 = survey[0]

Example output:

.. code-block:: python
   :class: code-output

   {'RESPONDENT_ID': 520001.0, 'Q1.a': datetime.date(2005, 5, 4), 'Q1.b': datetime.time(11, 20), 'Q2': 0, 'Q3': [1, 3, 5, 9], 'Q3.a': 'Nottingham Goose Fair         ', 'Q4': 2, 'Q5': [1, 5], 'Q6': 25.0, 'Q7': 1, 'Q8': 'A', 'WT': 1.131}

Or iterate through the responses:

.. code-block:: python

   for response in survey:
      print(response)
   
Responses returned by iteration or indexing are copies, and 
modifying them does not change the survey's underlying data:

.. code-block:: python

   response = survey[0]

   print("Original value:", response["Q2"])
   response["Q2"] = 2

   print("Modified copy:", response["Q2"])
   print("Survey value:", survey[0]["Q2"])

The survey still holds the original value:

.. code-block:: text
   :class: code-output

   Original value: 0
   Modified copy: 2
   Survey value: 0


Work with one question's answers
--------------------------------

Use a question name to get a shared :class:`triple_s.QuestionData` view:

.. code-block:: python

   answers = survey["Q2"]
   print(survey.datamap["Q2"].label)
   print(answers.to_polars())

The view stays connected to the survey's data. ``to_polars()`` returns a
Polars Series:

.. code-block:: python

   values = answers.to_polars()
   print(values.dtype)

Editing the returned Series does not update the survey. You can assign new
data with ``survey.data = new_data``; existing question views will read from it.

Next steps
----------

* :doc:`Datamap <datamap>`
  Interacting with survey datamp.

* :doc:`SurveyData <survey_data>`
  Interacting with survey data.
