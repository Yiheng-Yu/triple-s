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

   >>> survey[0]

   {'RESPONDENT_ID': 520001.0, 'Q1.a': datetime.date(2005, 5, 4), 'Q1.b': datetime.time(11, 20), 'Q2': 0, 'Q3': [1, 3, 5, 9], 'Q3.a': 'Nottingham Goose Fair', 'Q4': 2, 'Q5': [1, 5], 'Q6': 25.0, 'Q7': 1, 'Q8': 'A', 'WT': 1.131}

Or iterate through the responses, output as dictionary of ``{ question_name: question_response }``:

.. code-block:: python

   for response in survey:
      print(response)
   
This is equivalent to ``survey.iter_rows()``:

.. code-block:: python

   for response in survey.iter_rows():
      print(response)


To switch between values and labels:

.. code-block:: python

   print("This is response as values:")
   print(next(survey.iter_rows()))

   print("This is response as text labels:")
   survey.response_display = "label"
   print(next( survey.iter_rows()))

Note:

   Responses returned by iteration or indexing are copies of the original data. Modifying them does not change the survey's underlying data.
   
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

The view stays connected to the survey's data. It copies the current
``response_display`` and ``dataframe_format`` settings when you create it;
later changes to the survey's settings leave existing views unchanged.
``to_polars()`` returns a Polars Series:

.. code-block:: python

   values = answers.to_polars()
   print(values.dtype)

Editing the returned Series does not update the survey. You can assign new
data with ``survey.data = new_data``; existing question views will read from it.

Get cross tabs for a survey question
------------------------------------

You can use :meth:`~triple_s.Survey.crosstab` to look at one question's answers
by another question. For example, to look at ``Q2`` by ``Q4``:

.. code-block:: python

   table = survey.crosstab("Q2", ["Q4"])
   print(table)

You can also pass several questions as columns:

.. code-block:: python

   table = survey.crosstab("Q2", ["Q4", "Q5"])


Normalise counts
~~~~~~~~~~~~~~~~

You can return counts as relative frequencies normalised to 1.0:

.. code-block:: python

   table = survey.crosstab("Q2", ["Q4"], normalise=True)

Each response count is divided by its column total, so ``0.75`` represents
75%. The first row keeps the total counts. You can hide that row with
``show_column_total=False`` and the proportions stay the same.

Include response codes and question names
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

You can include the row-response codes and question name, and give the
overall count column a name:

.. code-block:: python

   table = survey.crosstab(
       "Q2", 
       ["Q4"], 
       include_response_codes=True,
       include_question_name=True, 
       colname_stats="answers",
   )

This adds ``code`` and ``question`` columns. The question name appears only
in the first displayed row; the remaining rows contain nulls.

If you want a pandas dataframe instead, simply add ``output="pandas"``.
Otherwise, the output follows ``survey.dataframe_format``. See the
:doc:`Survey Data recipes <../recipes/survey_data>` for more examples.

Next steps
----------

* :doc:`Datamap <datamap>`
  Working with survey definitions.

* :doc:`SurveyData <survey_data>`
  Interacting with survey data.