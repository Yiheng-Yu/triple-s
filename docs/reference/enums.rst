Enum flags
==========

.. currentmodule:: triple_s

Version
-------

.. autoclass:: triple_s.Version

Attributes
^^^^^^^^^^

.. autoattribute:: Version.V1_0
   :annotation: -> Version

.. autoattribute:: Version.V1_1
   :annotation: -> Version

.. autoattribute:: Version.V1_2
   :annotation: -> Version

.. autoattribute:: Version.V2_0
   :annotation: -> Version

.. autoattribute:: Version.V3_0
   :annotation: -> Version

Methods
^^^^^^^
.. automethod:: Version.available_options() -> list[str]

****

QuestionType
------------

.. autoclass:: triple_s.QuestionType

Attributes
^^^^^^^^^^

.. autoattribute:: QuestionType.None
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Single
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Multiple
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Logical
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Quantity
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Character
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Date
   :annotation: -> QuestionType

.. autoattribute:: QuestionType.Time
   :annotation: -> QuestionType

Methods
^^^^^^^
.. automethod:: QuestionType.available_options() -> list[str]

****

CodeFormat
----------

.. autoclass:: triple_s.CodeFormat

Attributes
^^^^^^^^^^

.. autoattribute:: CodeFormat.UnSigned
   :annotation: -> CodeFormat

.. autoattribute:: CodeFormat.Literal
   :annotation: -> CodeFormat

****

DataFormat
----------

.. autoclass:: triple_s.DataFormat

Attributes
^^^^^^^^^^

.. autoattribute:: DataFormat.Csv
   :annotation: -> DataFormat

.. autoattribute:: DataFormat.Fixed
   :annotation: -> DataFormat

****

Usage
-----

.. autoclass:: triple_s.Usage

Attributes
^^^^^^^^^^

.. autoattribute:: Usage.Null
   :annotation: -> Usage

.. autoattribute:: Usage.Serial
   :annotation: -> Usage

.. autoattribute:: Usage.Weight
   :annotation: -> Usage

Methods
^^^^^^^
.. automethod:: Usage.available_options() -> list[str]

****

Mode
----

.. autoclass:: triple_s.Mode

Attributes
^^^^^^^^^^

.. autoattribute:: Mode.Analysis
   :annotation: -> Mode

.. autoattribute:: Mode.Interview
   :annotation: -> Mode

****

ResponseDisplay
---------------

.. autoclass:: triple_s.config.ResponseDisplay

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.config.ResponseDisplay.Value
   :annotation: -> ResponseDisplay

.. autoattribute:: triple_s.config.ResponseDisplay.Label
   :annotation: -> ResponseDisplay
