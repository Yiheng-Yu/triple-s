Question
========

Question
--------


Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Question.type
.. autoattribute:: triple_s.Question.location
.. autoattribute:: triple_s.Question.label
.. autoattribute:: triple_s.Question.spread
.. autoattribute:: triple_s.Question.format
.. autoattribute:: triple_s.Question.usage
.. autoattribute:: triple_s.Question.filter
.. autoattribute:: triple_s.Question.response
.. autoattribute:: triple_s.Question.name

Methods
^^^^^^^

.. automethod:: triple_s.Question.copy
.. automethod:: triple_s.Question.set_location
.. automethod:: triple_s.Question.from_string
.. automethod:: triple_s.Question.from_dict
.. automethod:: triple_s.Question.to_classic
.. automethod:: triple_s.Question.to_xml

Location
--------

.. autoclass:: triple_s.Location

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Location.start
.. autoattribute:: triple_s.Location.finish

Methods
^^^^^^^
.. automethod:: triple_s.Location.from_dict

Spread
------

.. autoclass:: triple_s.Spread

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Spread.subfield_width
.. autoattribute:: triple_s.Spread.num_subfields

Methods
^^^^^^^
.. automethod:: triple_s.Spread.from_dict

Response
--------

.. autoclass:: triple_s.Response

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Response.range
.. autoattribute:: triple_s.Response.values

Methods
^^^^^^^

.. automethod:: triple_s.Response.get_code
.. automethod:: triple_s.Response.add_value
.. automethod:: triple_s.Response.set_value
.. automethod:: triple_s.Response.from_dict

DataRange
---------

.. autoclass:: triple_s.DataRange

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.DataRange.end
.. autoattribute:: triple_s.DataRange.start

Methods
^^^^^^^

.. automethod:: triple_s.DataRange.unsigned
.. automethod:: triple_s.DataRange.signed
.. automethod:: triple_s.DataRange.float
.. automethod:: triple_s.DataRange.from_dict

Code
----

.. autoclass:: triple_s.Code

Attributes
^^^^^^^^^^

.. autoattribute:: triple_s.Code.label
.. autoattribute:: triple_s.Code.code
.. autoattribute:: triple_s.Code.score
.. autoattribute:: triple_s.Code.filter

Methods
^^^^^^^

.. automethod:: triple_s.Code.unsigned
.. automethod:: triple_s.Code.string
.. automethod:: triple_s.Code.float
.. automethod:: triple_s.Code.signed
.. automethod:: triple_s.Code.from_dict
