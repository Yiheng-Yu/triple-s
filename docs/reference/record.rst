Record
======

.. autoclass:: triple_s.Record

Attributes
----------------

.. autoattribute:: Record.data_format
   :annotation: -> DataFormat | None

.. autoattribute:: Record.content
   :annotation: -> tuple[Question, ...]

.. autoattribute:: Record.href
   :annotation: -> str | None

.. autoattribute:: Record.ident
   :annotation: -> str

.. autoattribute:: Record.skip
   :annotation: -> str | None

Properties
-----------------

.. py:property:: triple_s.Record.question_names
   :type: List[str]

   List of question names.

.. py:property:: triple_s.Record.num_questions
   :type: int

.. py:property:: triple_s.Record.data_label
   :type: Dict[int | float | str, Text]

Methods
-------------

.. automethod:: Record.get(question_name) -> Question | None
.. automethod:: Record.load_data(path_data) -> SurveyData
.. automethod:: Record.validate_data(path) -> None
.. automethod:: Record.from_dict(content) -> Record
.. automethod:: Record.to_xml() -> str
