Text and variants
=================

Text
----

.. autoclass:: triple_s.Text

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Text.value
	:annotation: -> str
.. autoattribute:: triple_s.Text.variants
	:annotation: -> tuple

Methods
^^^^^^^

.. automethod:: triple_s.Text.add_variant(variant, mode=None, language=None) -> None
.. automethod:: triple_s.Text.remove_variant(index) -> Variant
.. automethod:: triple_s.Text.from_dict(content) -> Text

Variant
-------

.. autoclass:: triple_s.Variant

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Variant.language
	:annotation: -> str | None
.. autoattribute:: triple_s.Variant.value
	:annotation: -> str
.. autoattribute:: triple_s.Variant.mode
	:annotation: -> str | None

Methods
^^^^^^^

.. automethod:: triple_s.Variant.copy() -> Variant
.. automethod:: triple_s.Variant.from_dict(content) -> Variant
