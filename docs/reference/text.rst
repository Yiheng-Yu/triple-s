Text and variants
=================

.. currentmodule:: triple_s

Text
----

.. autoclass:: triple_s.Text

Attributes
^^^^^^^^^^

.. autoattribute:: Text.value
   :annotation: : str

.. autoattribute:: Text.variants
   :annotation: : tuple[Variant, ...]

Methods
^^^^^^^

.. automethod:: Text.add_variant(variant: str | Variant, mode: str | None = None, language: str | None = None) -> None

.. automethod:: Text.remove_variant(index: int) -> Variant

.. automethod:: Text.from_dict(content: dict) -> Text

.. automethod:: Text.to_dict() -> dict[str, str | list[dict[str, str]]]

Variant
-------

.. autoclass:: triple_s.Variant

Attributes
^^^^^^^^^^

.. autoattribute:: Variant.language
   :annotation: : str | None

.. autoattribute:: Variant.value
   :annotation: : str


.. autoattribute:: Variant.mode
   :annotation: : str | None


Methods
^^^^^^^

.. automethod:: Variant.copy() -> Variant

.. automethod:: Variant.from_dict(content: dict) -> Variant

.. automethod:: Variant.to_dict() -> dict[str, str]

