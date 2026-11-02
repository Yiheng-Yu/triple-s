Record
======

.. autoclass:: triple_s.Record

Attributes
----------------

.. autoattribute:: triple_s.Record.data_format
   :annotation: -> DataFormat | None

.. autoattribute:: triple_s.Record.content
   :annotation: -> tuple[Question, ...]

.. autoattribute:: triple_s.Record.href
   :annotation: -> str | None

.. autoattribute:: triple_s.Record.ident
   :annotation: -> str

.. autoattribute:: triple_s.Record.skip
   :annotation: -> str | None

Properties
-----------------

.. py:property:: triple_s.Record.questions
   :type: List[str]

   List of question names.

Methods
-------------

.. automethod:: triple_s.Record.get(question_name) -> Question | None
.. automethod:: triple_s.Record.load_data(path_data) -> SurveyData
.. automethod:: triple_s.Record.validate_data(path) -> None
.. automethod:: triple_s.Record.from_dict(content) -> Record
.. automethod:: triple_s.Record.to_xml() -> str
