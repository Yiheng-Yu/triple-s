.. triple-s documentation master file, created by
   sphinx-quickstart on Sat Oct 17 07:38:51 2026.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

triple-s documentation
======================

A parser for reading/parsing `Triple-S <https://www.triple-s.org/>`_
datamap files (.sss).

`Triple-S <https://triple-s.org/documentation/faq/>`__ is a file format
used for handling survey data.

This format has been relatively popular and is used in many different
places. Most major survey platforms use Triple-S to transfer survey data
from one platform to another.

It typically consists of two independent files:

A. A data file, which can either be a text (.asc) file containing the
   raw data or a CSV (.csv) file with question names as column headers.
   Most data files use the former format.

B. A datamap file that records how the data file should be read,
   including where the data for each question and respondent is located.

The current library provides a (very!) fast and convenient way of
reading, loading, and modifying Triple-S files.

It supports all Triple-S file formats (Triple-S Classic and XML)
up to Triple-S 2.0.

Installation
---------------

1) Install with ``pip``::

      pip install triple-s-survey -U

2) install with ``uv``::

      uv pip install triple-s-survey -U


Library Overview
----------------

Import the installed package as ``triple_s``. The API separates survey
definitions from respondent data:

* :class:`triple_s.Survey` combines ``datamap`` and ``data``.
  :func:`triple_s.read_triple_s` returns a survey, with ``info`` providing
  direct access to the datamap's metadata, with two main components combined together:

* :class:`triple_s.Datamap` contains metadata in ``info`` and question
  definitions in ``record``. :func:`triple_s.read_datamap` returns a datamap.
  
* :class:`triple_s.SurveyData` contains respondent data. Both
  ``datamap.load_data(path)`` and ``datamap.record.load_data(path)`` return
  this object; use its ``to_polars()`` or ``to_pandas()`` methods to export it.


.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   print(survey.info.name)
   df = survey.data.to_polars()
   xml = survey.datamap.to_xml()

Omit the data path to read only the definition; ``survey.has_data()`` then
returns ``False``.

See :doc:`getting_started/overview` for an overview and links to the guides and
:doc:`api/index` for the public Python API reference.

.. toctree::
   :hidden:
   :maxdepth: 3
   :caption: Getting Started

   getting_started/overview
   getting_started/survey
   getting_started/datamap
   getting_started/survey_data

.. toctree::
   :hidden:
   :maxdepth: 3
   :caption: API Reference

   api/functions
   api/objects
   api/exceptions
