triple-s documentation
======================

Welcome to ``triple-s``! This library helps you read, explore, and edit
`Triple-S <https://www.triple-s.org/>`_ survey files in Python, with Rust
handling the work behind the scenes.

`Triple-S <https://triple-s.org/documentation/faq/>`__ is a file format
used for handling survey data. Triple-s standards consists two files:

* A **datamap** (``.sss``) describes the questions and explains how to read
  each response from the data file.
* An **optional data file** (``.asc`` or ``.csv``) contains the respondents' answers.
  An ``.asc`` file stores values in fixed-width fields; a ``.csv`` file
  separates them with commas.


The current ``triple-s`` library brings these files together so you can inspect a survey,
work with its responses in Polars or pandas, and update its definitions.

It supports Triple-S Classic and XML formats up to Triple-S 2.0. 

3.0 support is incoming but should be mostly working okay.

Installation
------------

Install the package with ``pip``:

.. code-block:: shell

   pip install triple-s-survey -U

Or, if you use ``uv``:

.. code-block:: shell

   uv pip install triple-s-survey -U


The installation process may take a little while, as the source code needs to be compiled. 
One dependency in particular, `polars <https://pola.rs/>`_, 
can take quite some time to compile, so don't worry if things seem a little slow!

If you'd like to see what's happening along the way, you can add the `-v` flag to enable more detailed installation progress:

.. code-block:: shell

   pip install triple-s-survey -v -U

Or for ``uv``:

.. code-block:: shell

   uv pip install triple-s-survey -v -U

Library Overview
----------------

Once installed, import the package as ``triple_s``. To get started, load
a datamap and its matching data file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   print(survey.info.name)
   df = survey.data.to_polars()
   xml = survey.datamap.to_xml()

Replace the example paths with your own files. If you only have a datamap,
leave out the second path to explore the survey definition without loading
responses.

You will mainly work with three objects:

* :class:`triple_s.Survey` brings the definition and responses together.
  Access them through ``survey.datamap`` and ``survey.data``, or use
  ``survey.info`` to read and edit the survey's metadata.
* :class:`triple_s.Datamap` holds metadata in ``info`` and question
  definitions in ``record``. Use :func:`triple_s.read_datamap` when you
  want to work with the definition on its own.
* :class:`triple_s.SurveyData` holds the loaded responses. Use
  ``to_polars()`` or ``to_pandas()`` to turn them into a DataFrame.

For a closer look, follow the :doc:`Quick Start <getting_started/overview>`
and the guides that follow it. The :doc:`API Reference <api/index>` has
more detail on individual classes, methods, and functions.

Architecture and Implementation
-------------------------------

The library's core is written in Rust, with a Python interface built using
PyO3 and maturin. This lets you work with familiar Python objects while
Rust handles parsing and data processing.

If you prefer working in Rust or want to explore how the library fits
together, the `Rust API documentation on docs.rs
<https://docs.rs/triple_s/latest/triple_s/index.html>`_ is a good place
to start. Much of the functionality exposed in Python is also available
directly from Rust.

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
