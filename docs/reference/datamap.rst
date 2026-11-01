Datamap
=======

.. autoclass:: triple_s.Datamap

Attributes
----------------

.. autoattribute:: triple_s.Datamap.record
	:annotation: -> Record
.. autoattribute:: triple_s.Datamap.info
	:annotation: -> SurveyInfo
.. autoattribute:: triple_s.Datamap.title
	:annotation: -> str | None

Methods
-------------

.. automethod:: triple_s.Datamap.get(question_name) -> Question | None
.. automethod:: triple_s.Datamap.from_dict(content) -> Datamap
.. automethod:: triple_s.Datamap.to_classic() -> str
.. automethod:: triple_s.Datamap.to_xml() -> str
.. automethod:: triple_s.Datamap.load_data(path) -> SurveyData
