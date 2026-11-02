Enum flags
==========

Version
-------

.. autoclass:: triple_s.Version

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Version.V1_0
   :annotation: -> Version

.. autoattribute:: triple_s.Version.V1_1
   :annotation: -> Version

.. autoattribute:: triple_s.Version.V1_2
   :annotation: -> Version

.. autoattribute:: triple_s.Version.V2_0
   :annotation: -> Version

.. autoattribute:: triple_s.Version.V3_0
   :annotation: -> Version

Methods
^^^^^^^
.. automethod:: triple_s.Version.available_options() -> list[str]

****

QuestionType
------------

.. autoclass:: triple_s.QuestionType

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.QuestionType.None
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Single
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Multiple
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Logical
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Quantity
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Character
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Date
   :annotation: -> QuestionType

.. autoattribute:: triple_s.QuestionType.Time
   :annotation: -> QuestionType

Methods
^^^^^^^
.. automethod:: triple_s.QuestionType.available_options() -> list[str]

****

CodeFormat
----------

.. autoclass:: triple_s.CodeFormat

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.CodeFormat.UnSigned
   :annotation: -> CodeFormat

.. autoattribute:: triple_s.CodeFormat.Literal
   :annotation: -> CodeFormat

****

DataFormat
----------

.. autoclass:: triple_s.DataFormat

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.DataFormat.Csv
   :annotation: -> DataFormat

.. autoattribute:: triple_s.DataFormat.Fixed
   :annotation: -> DataFormat

****

Usage
-----

.. autoclass:: triple_s.Usage

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Usage.Null
   :annotation: -> Usage

.. autoattribute:: triple_s.Usage.Serial
   :annotation: -> Usage

.. autoattribute:: triple_s.Usage.Weight
   :annotation: -> Usage

Methods
^^^^^^^
.. automethod:: triple_s.Usage.available_options() -> list[str]

****

Mode
----

.. autoclass:: triple_s.Mode

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Mode.Analysis
   :annotation: -> Mode

.. autoattribute:: triple_s.Mode.Interview
   :annotation: -> Mode

****

Value
-----

.. autoclass:: triple_s.Value

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Value.UnSigned
   :annotation: -> type[Value.UnSigned]

.. autoattribute:: triple_s.Value.Signed
   :annotation: -> type[Value.Signed]

.. autoattribute:: triple_s.Value.Float
   :annotation: -> type[Value.Float]

.. autoattribute:: triple_s.Value.String
   :annotation: -> type[Value.String]
