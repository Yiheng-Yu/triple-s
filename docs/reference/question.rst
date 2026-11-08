Questions and responses
=======================

.. currentmodule:: triple_s

Question
--------


Attributes
^^^^^^^^^^

.. autoattribute:: Question.type
   :annotation: -> str | None

.. autoattribute:: Question.location
   :annotation: -> Location

.. autoattribute:: Question.label
   :annotation: -> Text

.. autoattribute:: Question.spread
   :annotation: -> Spread | None

.. autoattribute:: Question.format
   :annotation: -> str | None

.. autoattribute:: Question.usage
   :annotation: -> str | None

.. autoattribute:: Question.filter
   :annotation: -> str | None

.. autoattribute:: Question.response
   :annotation: -> Response

.. autoattribute:: Question.name
   :annotation: -> str


Properties
^^^^^^^^^^^
.. :py:property:: triple_s.Question.response_label
      :type: -> Dict[str | int | float, Text]

Methods
^^^^^^^

.. automethod:: Question.copy() -> Question
.. automethod:: Question.set_location(start=None, finish=None) -> None
.. automethod:: Question.from_string(question_fragment) -> Question
.. automethod:: Question.from_dict(content) -> Question
.. automethod:: Question.to_classic() -> str
.. automethod:: Question.to_xml() -> str

****

Location
--------

.. autoclass:: triple_s.Location

Attributes
^^^^^^^^^^

.. autoattribute:: Location.start
   :annotation: -> int

.. autoattribute:: Location.finish
   :annotation: -> int

Methods
^^^^^^^
.. automethod:: Location.from_dict(value) -> Location

****

Spread
------

.. autoclass:: triple_s.Spread

Attributes
^^^^^^^^^^

.. autoattribute:: Spread.subfield_width
   :annotation: -> int | None

.. autoattribute:: Spread.num_subfields
   :annotation: -> int

Methods
^^^^^^^
.. automethod:: Spread.from_dict(value) -> Spread

****

Response
--------

.. autoclass:: triple_s.Response

Attributes
^^^^^^^^^^

.. autoattribute:: Response.range
   :annotation: -> DataRange | None

.. autoattribute:: Response.values
   :annotation: -> tuple[Code, ...]

Methods
^^^^^^^

.. automethod:: Response.get_value(value) -> Code | None
.. automethod:: Response.add_value(value) -> None
.. automethod:: Response.set_value(value) -> None
.. automethod:: Response.from_dict(content) -> Response

****

DataRange
---------

.. autoclass:: triple_s.DataRange

Attributes
^^^^^^^^^^

.. autoattribute:: DataRange.end
   :annotation: -> int | float

.. autoattribute:: DataRange.start
   :annotation: -> int | float

Methods
^^^^^^^

.. automethod:: DataRange.unsigned(start, end) -> DataRange
.. automethod:: DataRange.signed(start, end) -> DataRange
.. automethod:: DataRange.float(start, end) -> DataRange
.. automethod:: DataRange.from_dict(content) -> DataRange

****

Code
----

.. autoclass:: triple_s.Code

Attributes
^^^^^^^^^^

.. autoattribute:: Code.label
   :annotation: -> Text

.. autoattribute:: Code.value
   :annotation: -> int | float | str

.. autoattribute:: Code.score
   :annotation: -> float | None

.. autoattribute:: Code.filter
   :annotation: -> str | None

Methods
^^^^^^^

Creating new instance with various data types:
*********************************************************

.. automethod:: Code.unsigned(value, label=None, filter=None, score=None) -> Code
.. automethod:: Code.string(value, label=None, filter=None, score=None) -> Code
.. automethod:: Code.float(value, label=None, filter=None, score=None) -> Code
.. automethod:: Code.signed(value, label=None, filter=None, score=None) -> Code

Creating new instance from dictionary:
*********************************************************
.. automethod:: Code.from_dict(content) -> Code
