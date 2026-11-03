QuestionData
============

.. autoclass:: triple_s.QuestionData

Attributes
----------

.. autoattribute:: triple_s.QuestionData.name
   :annotation: -> str

.. autoattribute:: triple_s.QuestionData.type
   :annotation: -> str | None

.. autoattribute:: triple_s.QuestionData.label
   :annotation: -> triple_s.Text

.. autoattribute:: triple_s.QuestionData.range
   :annotation: -> triple_s.DataRange | None

.. autoattribute:: triple_s.QuestionData.data
   :annotation: -> tuple

   A copy of the answers as a tuple, with ``None`` for missing values.

Iteration and indexing
----------------------

Use ``view[index]`` to read an answer (negative indices are supported), or
``list(view)`` to iterate over answers. Each iterator reads a snapshot of the
column taken when iteration starts.

Methods
-------

.. automethod:: triple_s.QuestionData.to_polars() -> polars.Series
.. automethod:: triple_s.QuestionData.to_pandas() -> pandas.Series
.. automethod:: triple_s.QuestionData.to_list() -> List[Any]