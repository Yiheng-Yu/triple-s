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

   pip install triple-s -U

2) install with ``uv``::

   uv pip install triple-s -U

Note: 

Installing it for the first time may be a bit slow, this is because one of `triple_s`'s dependency, `polars`,
takes a while to compile. You might want to add `-v` to see a bit more information, like::

   pip install triple-s -U -v

Or if you were using `uv`::

   uv pip install triple-s -U -v




See :doc:`getting_started/quickstart` for a working Python workflow and
:doc:`api/index` documents the public Python API.

.. toctree::
   :hidden:
   :maxdepth: 3

   getting_started/quickstart
   api/index
