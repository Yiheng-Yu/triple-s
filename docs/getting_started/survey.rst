Survey
======

:class:`triple_s.Survey` combines a datamap and respondent data in one object.
Use ``survey.datamap`` for the definition, ``survey.info`` for metadata,
and ``survey.data`` for the loaded responses.

Loading a triple-s Survey
---------------------------

Load triple-s survey via :func:`triple_s.read_triple_s`:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s(
      "path/to/survey.sss", 
      path_data="path/to/data.asc"  # optional
      )

   print(survey)


.. code-block:: python
   :class: code-output

   Survey({
      name: "SP5201-1"
      title: "Historic House Exit Survey<br/>First Wave"
      questions: ["RESPONDENT_ID", "Q1.a", "Q1.b", ...]
      num_response: 3
   })


Getting Survey Data
***********************************

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

Omitting the data path returns a survey with empty data;
``survey.has_data()`` then returns ``False``.

``survey.info`` and ``survey.datamap.info`` share metadata. For example,
assigning ``survey.title = "Updated title"`` also updates
``survey.datamap.title`` and ``survey.info.title``.

See :doc:`datamap` to edit the definition and :doc:`survey_data` to load
and export respondent data.
