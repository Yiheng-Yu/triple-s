Questions and responses
=======================

.. currentmodule:: triple_s

Question
--------


Attributes
^^^^^^^^^^

.. autoattribute:: Question.type
   :annotation: : str | None

.. autoattribute:: Question.label
   :annotation: : Text

.. autoattribute:: Question.location
   :annotation: : Location

.. autoattribute:: Question.spread
   :annotation: : Spread | None

.. autoattribute:: Question.format
   :annotation: : str | None

.. autoattribute:: Question.usage
   :annotation: : str | None

.. autoattribute:: Question.filter
   :annotation: : str | None

.. autoattribute:: Question.response
   :annotation: : Response

.. autoattribute:: Question.name
   :annotation: -> str


Properties
^^^^^^^^^^^
.. :py:property:: triple_s.Question.response_label
      :type: -> Dict[str | int | float, Text]

Methods
^^^^^^^

.. automethod:: Question.copy() -> Question

.. automethod:: Question.set_location(start: int | None = None, finish: int | None = None) -> None

.. automethod:: Question.from_string(question_fragment: str) -> Question

.. automethod:: Question.from_dict(content: dict) -> Question

.. automethod:: Question.to_classic() -> str

.. automethod:: Question.to_xml() -> str

.. raw:: html

    <hr>

Location
--------

.. autoclass:: triple_s.Location

Positions are one-based and inclusive. ``finish=None`` uses ``start``. Fixed-width positions count Unicode scalar values; CSV uses ``start`` as a field number.

Properties
----------

.. autoattribute:: Location.start
   :annotation: : int

   Writable.

.. autoattribute:: Location.finish
   :annotation: : int

   Writable.

Methods
-------

.. automethod:: Location.from_dict(value: dict) -> Location

.. raw:: html

    <hr>

Spread
------

.. autoclass:: triple_s.Spread

Attributes
^^^^^^^^^^

.. autoattribute:: Spread.subfield_width
   :annotation: : int | None

Methods
^^^^^^^
.. automethod:: Spread.from_dict(value) -> Spread

.. raw:: html

    <hr>

Response
--------

.. autoclass:: triple_s.Response

Attributes
^^^^^^^^^^

.. autoattribute:: Response.range
   :annotation: : DataRange | None

.. autoattribute:: Response.values
   :annotation: : tuple[Code, ...]

.. autoattribute:: Response.response_label
   :annotation: : dict[int | float | str, Text]


Methods
^^^^^^^

.. automethod:: Response.get_value(value: int | float | str) -> Code | None

.. automethod:: Response.add_value(value: Code) -> None

.. automethod:: Response.set_value(value: Code) -> None

.. automethod:: Response.from_dict(content: dict) -> Response

.. raw:: html

    <hr>

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

.. automethod:: DataRange.unsigned(start: int, end: int) -> DataRange

.. automethod:: DataRange.signed(start: int, end: int) -> DataRange

.. automethod:: DataRange.float(start: float, end: float) -> DataRange

.. automethod:: DataRange.from_dict(content: dict) -> DataRange

.. raw:: html

    <hr>

Code
----

.. autoclass:: triple_s.Code

Attributes
^^^^^^^^^^

.. autoattribute:: Code.label
   :annotation: : Text

.. autoattribute:: Code.value
   :annotation: : int | float | str

.. autoattribute:: Code.score
   :annotation: : float | None

.. autoattribute:: Code.filter
   :annotation: : str | None

Methods
^^^^^^^

Creating new instance with various data types:
*********************************************************

.. automethod:: Code.unsigned(value: int, label: str | None = None, filter: str | None = None, score: float | None = None) -> Code

.. automethod:: Code.signed(value: int, label: str | None = None, filter: str | None = None, score: float | None = None) -> Code

.. automethod:: Code.float(value: float, label: str | None = None, filter: str | None = None, score: float | None = None) -> Code

.. automethod:: Code.string(value: str, label: str | None = None, filter: str | None = None, score: float | None = None) -> Code

Creating new instance from dictionary:
*********************************************************
.. automethod:: Code.from_dict(content: dict) -> Code
