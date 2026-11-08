SurveyInfo
==========

.. currentmodule:: triple_s

.. autoclass:: triple_s.SurveyInfo

Attributes
----------------

.. autoattribute:: SurveyInfo.time
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.style
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.name
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.user
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.origin
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.title
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.standard
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.date
   :annotation: -> str | None

.. autoattribute:: SurveyInfo.version
   :annotation: -> Version

.. autoattribute:: SurveyInfo.language_options
   :annotation: -> tuple[str, ...]

.. autoattribute:: SurveyInfo.mode_options
   :annotation: -> tuple[Mode, ...]

Methods
-------------

.. automethod:: SurveyInfo.from_dict(content) -> SurveyInfo
.. automethod:: SurveyInfo.to_dict() -> dict[str, Version | str | list[str] | list[Mode]]
