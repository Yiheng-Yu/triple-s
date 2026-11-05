Text and variants
=================

Text
----

.. autoclass:: triple_s.Text

Attributes
^^^^^^^^^^

.. autoattribute:: Text.value
   :annotation: -> str

.. autoattribute:: Text.variants
   :annotation: -> tuple[Variant, ...]

Methods
^^^^^^^

.. automethod:: Text.add_variant(variant, mode=None, language=None) -> None
.. automethod:: Text.remove_variant(index) -> Variant
.. automethod:: Text.from_dict(content) -> Text
.. automethod:: Text.to_dict() -> Dict[str, str|List[Dict[str, str]]]

Variant
-------

.. autoclass:: triple_s.Variant

Attributes
^^^^^^^^^^

.. autoattribute:: Variant.language
   :annotation: -> str | None

.. autoattribute:: Variant.value
   :annotation: -> str

.. autoattribute:: Variant.mode
   :annotation: -> str | None

Methods
^^^^^^^

.. automethod:: Variant.copy() -> Variant
.. automethod:: Variant.from_dict(content) -> Variant
.. automethod:: Variant.to_dict() -> Dict[str, str] 
