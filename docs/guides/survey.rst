Survey
======

A Triple-S survey usually consists of a datamap (``.sss``) describing the
questions and a matching data file (``.asc`` or ``.csv``) containing responses.
Import the installed package as ``triple_s``.


Read triple-s files
----------------------

Load triple-s survey via :func:`read_triple_s`:

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

Viewing data from a respondent via indexing:

.. code-block:: python

   respondent_1 = survey[0]

This would give you:

.. code-block:: python
   :class: code-output

   {'RESPONDENT_ID': 520001.0, 'Q1.a': datetime.date(2005, 5, 4), 'Q1.b': datetime.time(11, 20), 'Q2': 0, 'Q3': [1, 3, 5, 9], 'Q3.a': 'Nottingham Goose Fair         ', 'Q4': 2, 'Q5': [1, 5], 'Q6': 25.0, 'Q7': 1, 'Q8': 'A', 'WT': 1.131}

Alternatively, you can iterate each individual response like this:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("path/to/survey.sss", "path/to/data.asc")
   for response in survey:
      print(response)
   
*Note: Responses returned by iteration or indexing are copies.
Modifying them does not change the survey's underlying data.
For example:*

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("path/to/survey.sss", "path/to/data.asc")
   resp_1 = survey[1]

   print("Original value:", resp_1["Q2"])
   resp_1["Q2"] = 2

   print("Modified copy:", resp_1["Q2"])
   print("Survey value:", survey[1]["Q2"])

*You will notice the value of 'Q2' hasn't changed:*

.. code-block:: text
   :class: code-output

   Original value: 1
   Modified copy: 2
   Survey value: 1


See :doc:`SurveyData <survey_data>` for more details about `SurveyData` class.


Work with one question's answers
--------------------------------

Use a question name to get a shared :class:`triple_s.QuestionData` view:

.. code-block:: python

   answers = survey["Q2"]
   print(answers.question.label)
   print(answers.to_polars())

The view stays connected to the survey's data. ``to_polars()`` returns the
answers as a Polars Series, ready to use directly:

.. code-block:: python

   values = answers.to_polars()
   print(values.dtype)

Editing the returned Series does not update the survey. You can assign new
data with ``survey.data = new_data``; existing question views will read from it.

View Data map
---------------------

See :doc:`Datamap <datamap>` for more details about `Datamap` class.


Next steps
----------

* :doc:`Datamap <datamap>`
  Inspect question definitions, edit survey metadata and labels, and export the updated datamap.

* :doc:`SurveyData <survey_data>`
  Load respondent data using a datamap and convert it to Polars or pandas DataFrames.
