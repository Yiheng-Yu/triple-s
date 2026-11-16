Record
======

.. currentmodule:: triple_s

.. autoclass:: triple_s.Record

Attributes
----------------

.. autoattribute:: Record.data_format
   :annotation: : DataFormat | None

.. autoattribute:: Record.content
   :annotation: : tuple[Question, ...]

.. autoattribute:: Record.href
   :annotation: : str | None

.. autoattribute:: Record.ident
   :annotation: : str

.. autoattribute:: Record.skip
   :annotation: : str | None

.. autoattribute:: Record.question_names
   :annotation: : list[str]

.. autoattribute:: Record.num_questions
   :annotation: : int

Methods
--------

.. automethod:: Record.empty() -> Record

.. automethod:: Record.get(question_name: str) -> Question | None

.. automethod:: Record.add_question(question: Question) -> None

.. automethod:: Record.remove_question(question_name: str) -> Question | None

.. automethod:: Record.validate_data(path: str) -> None

.. automethod:: Record.from_dict(content: dict) -> Record

.. automethod:: Record.to_xml() -> str

