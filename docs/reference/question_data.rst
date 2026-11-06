QuestionData
============

.. autoclass:: triple_s.QuestionData

Attributes
----------

.. autoattribute:: QuestionData.name
   :annotation: -> str

.. autoattribute:: QuestionData.type
   :annotation: -> str | None

.. autoattribute:: QuestionData.label
   :annotation: -> triple_s.Text

.. autoattribute:: QuestionData.range
   :annotation: -> triple_s.DataRange | None

.. autoattribute:: QuestionData.data
   :annotation: -> triple_s.Question

.. autoattribute:: QuestionData.data
   :annotation: -> tuple

   A copy of the answers as a tuple, with ``None`` for missing values.

.. autoattribute:: QuestionData.response_display
   :annotation: -> str

Properties
-------------------
.. :py:property:: triple_s.QuestionData.response_label
   :annotation: Dict[str | int | float, Text]

Iteration and indexing
----------------------

Use ``view[index]`` to read an answer (negative indices are supported), or
``list(view)`` to iterate over answers. Each iterator reads a snapshot of the
column taken when iteration starts.

Methods
-------

.. automethod:: QuestionData.to_polars() -> polars.Series
.. automethod:: QuestionData.to_pandas() -> pandas.Series
.. automethod:: QuestionData.to_list() -> List[Any]
