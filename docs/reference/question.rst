Questions and responses
=======================

Question
--------


Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Question.type
   :annotation: -> str | None

.. autoattribute:: triple_s.Question.location
   :annotation: -> Location

.. autoattribute:: triple_s.Question.label
   :annotation: -> Text

.. autoattribute:: triple_s.Question.spread
   :annotation: -> Spread | None

.. autoattribute:: triple_s.Question.format
   :annotation: -> str | None

.. autoattribute:: triple_s.Question.usage
   :annotation: -> str | None

.. autoattribute:: triple_s.Question.filter
   :annotation: -> str | None

.. autoattribute:: triple_s.Question.response
   :annotation: -> Response

.. autoattribute:: triple_s.Question.name
   :annotation: -> str

Methods
^^^^^^^

.. automethod:: triple_s.Question.copy() -> Question
.. automethod:: triple_s.Question.set_location(start=None, finish=None) -> None
.. automethod:: triple_s.Question.from_string(question_fragment) -> Question
.. automethod:: triple_s.Question.from_dict(content) -> Question
.. automethod:: triple_s.Question.to_classic() -> str
.. automethod:: triple_s.Question.to_xml() -> str

****

Location
--------

.. autoclass:: triple_s.Location

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Location.start
   :annotation: -> int

.. autoattribute:: triple_s.Location.finish
   :annotation: -> int

Methods
^^^^^^^
.. automethod:: triple_s.Location.from_dict(value) -> Location

****

Spread
------

.. autoclass:: triple_s.Spread

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Spread.subfield_width
   :annotation: -> int | None

.. autoattribute:: triple_s.Spread.num_subfields
   :annotation: -> int

Methods
^^^^^^^
.. automethod:: triple_s.Spread.from_dict(value) -> Spread

****

Response
--------

.. autoclass:: triple_s.Response

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Response.range
   :annotation: -> DataRange | None

.. autoattribute:: triple_s.Response.values
   :annotation: -> tuple[Code, ...]

Methods
^^^^^^^

.. automethod:: triple_s.Response.get_code(code) -> Code | None
.. automethod:: triple_s.Response.add_value(value) -> None
.. automethod:: triple_s.Response.set_value(value) -> None
.. automethod:: triple_s.Response.from_dict(content) -> Response

****

DataRange
---------

.. autoclass:: triple_s.DataRange

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.DataRange.end
   :annotation: -> int | float

.. autoattribute:: triple_s.DataRange.start
   :annotation: -> int | float

Methods
^^^^^^^

.. automethod:: triple_s.DataRange.unsigned(start, end) -> DataRange
.. automethod:: triple_s.DataRange.signed(start, end) -> DataRange
.. automethod:: triple_s.DataRange.float(start, end) -> DataRange
.. automethod:: triple_s.DataRange.from_dict(content) -> DataRange

****

Code
----

.. autoclass:: triple_s.Code

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Code.label
   :annotation: -> Text

.. autoattribute:: triple_s.Code.code
   :annotation: -> int | float | str

.. autoattribute:: triple_s.Code.score
   :annotation: -> float | None

.. autoattribute:: triple_s.Code.filter
   :annotation: -> str | None

Methods
^^^^^^^

Creating new instance with various data types:
*********************************************************

.. automethod:: triple_s.Code.unsigned(value, label=None, filter=None, score=None) -> Code
.. automethod:: triple_s.Code.string(value, label=None, filter=None, score=None) -> Code
.. automethod:: triple_s.Code.float(value, label=None, filter=None, score=None) -> Code
.. automethod:: triple_s.Code.signed(value, label=None, filter=None, score=None) -> Code

Creating new instance from dictionary:
*********************************************************
.. automethod:: triple_s.Code.from_dict(content) -> Code
