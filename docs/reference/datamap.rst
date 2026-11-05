Datamap
=======

.. autoclass:: triple_s.Datamap

Attributes
----------------

.. autoattribute:: Datamap.record
   :annotation: -> Record

.. autoattribute:: Datamap.info
   :annotation: -> SurveyInfo

.. autoattribute:: Datamap.title
   :annotation: -> str | None

Properties
----------------
.. :py:property:: triple_s.Datamap.question_names
   :type: -> List[str]

.. :py:property:: triple_s.Datamap.num_questions
   :type: -> int

Methods
-------------

.. automethod:: Datamap.get(question_name) -> Question | None
.. automethod:: Datamap.from_dict(content) -> Datamap
.. automethod:: Datamap.to_classic() -> str
.. automethod:: Datamap.to_xml() -> str
.. automethod:: Datamap.load_data(path) -> SurveyData
