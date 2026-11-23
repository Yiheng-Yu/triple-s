Datamap
=======

.. currentmodule:: triple_s

.. autoclass:: triple_s.Datamap

Attributes
----------------

.. autoattribute:: Datamap.record
   :annotation: : Record

.. autoattribute:: Datamap.info
   :annotation: : SurveyInfo

.. autoattribute:: Datamap.title
   :annotation: : str | None

.. autoattribute:: Datamap.question_names
   :annotation: : list[str]

   Returns a copy of question names in the current survey datamap.

.. autoattribute:: Datamap.num_questions
   :annotation: : int

   Read-only question count.

Methods
-------

.. raw:: html

   <div class="separator">Iteration & indexing</div>

.. py:method:: Datamap.__iter__() -> Iterator[Question]

   Iterate over the questions in the record.

.. py:method:: Datamap.__getitem__(index: int | str | slice) -> Question | tuple[Question, ...]

   Accept a question index, case-sensitive question name, or slice.

.. raw:: html

   <div class="separator">Content access</div>

.. automethod:: Datamap.get(question_name: str) -> Question | None

.. raw:: html

   <div class="separator">Create instance</div>

.. automethod:: Datamap.from_dict(content: dict) -> Datamap

.. raw:: html

   <div class="separator">Exporting</div>

.. automethod:: Datamap.to_classic() -> str

.. automethod:: Datamap.to_xml() -> str
