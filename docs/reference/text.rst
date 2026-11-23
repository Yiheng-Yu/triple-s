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

.. raw:: html

   <div class="separator">Mutation</div>

.. automethod:: Text.add_variant(variant: str | Variant, mode: str | None = None, language: str | None = None) -> None

.. automethod:: Text.remove_variant(index: int) -> Variant

.. raw:: html

   <div class="separator">Create & export</div>

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

.. raw:: html

   <div class="separator">Mutation</div>

.. automethod:: Variant.copy() -> Variant

.. raw:: html

   <div class="separator">Create & export</div>

.. automethod:: Variant.from_dict(content: dict) -> Variant

.. automethod:: Variant.to_dict() -> dict[str, str]
