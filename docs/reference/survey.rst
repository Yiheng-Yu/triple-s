Survey
======

.. autoclass:: triple_s.Survey

Use :func:`triple_s.read_triple_s` to read a datamap and optional respondent
file together. ``info`` shares metadata with ``datamap.info``; changing either
view updates the survey. ``title`` is also writable.

Attributes
----------

.. autoattribute:: Survey.datamap
   :annotation: -> Datamap

.. autoattribute:: Survey.info
   :annotation: -> SurveyInfo

.. autoattribute:: Survey.title
   :annotation: -> str | None

.. autoattribute:: Survey.data
   :annotation: -> SurveyData

Methods
-------

.. automethod:: Survey.has_data() -> bool
