Survey
======

.. autoclass:: triple_s.Survey

Use :func:`triple_s.read_triple_s` to read a datamap and optional respondent
file together. ``info`` shares metadata with ``datamap.info``; changing either
view updates the survey. ``title`` is also writable.

Attributes
----------

.. autoattribute:: triple_s.Survey.datamap
   :annotation: -> Datamap

.. autoattribute:: triple_s.Survey.info
   :annotation: -> SurveyInfo

.. autoattribute:: triple_s.Survey.title
   :annotation: -> str | None

.. autoattribute:: triple_s.Survey.data
   :annotation: -> SurveyData

Methods
-------

.. automethod:: triple_s.Survey.has_data() -> bool
