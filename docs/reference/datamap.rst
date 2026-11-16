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

.. automethod:: Datamap.get(question_name: str) -> Question | None

.. automethod:: Datamap.from_dict(content: dict) -> Datamap

.. automethod:: Datamap.to_classic() -> str

.. automethod:: Datamap.to_xml() -> str

