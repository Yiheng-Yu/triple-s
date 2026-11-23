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

.. raw:: html

   <div class="separator">Iteration & indexing</div>

.. py:method:: Record.__iter__() -> Iterator[Question]

   Iterate over the questions in the record.

.. py:method:: Record.__getitem__(index: int | str | slice) -> Question | tuple[Question, ...]

   Accept a question index, question name, or slice.

.. raw:: html

   <div class="separator">Create instance</div>

.. automethod:: Record.empty() -> Record

.. automethod:: Record.from_dict(content: dict) -> Record

.. raw:: html

   <div class="separator">Content access</div>

.. automethod:: Record.get(question_name: str) -> Question | None

.. raw:: html

   <div class="separator">Mutation</div>

.. automethod:: Record.add_question(question: Question) -> None

.. automethod:: Record.remove_question(question_name: str) -> Question | None

.. raw:: html

   <div class="separator">Exporting & validation</div>

.. automethod:: Record.to_xml() -> str

.. automethod:: Record.validate_data(path: str) -> None
