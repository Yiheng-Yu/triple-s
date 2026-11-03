triple-s documentation
======================

Welcome to ``triple-s``! This library helps you read, explore, and edit
`Triple-S <https://www.triple-s.org/>`_ survey files in Python, with Rust
handling the work behind the scenes. 

`Triple-S <https://triple-s.org/documentation/faq/>`__ is a file format used for handling survey data. 
This format has been relatively popular and been used accross many different 
places, and most of the major survey platforms uses triple-s to transfer 
survey data from one place to another.

A triple-s survey usually comes in two files:

* A **datamap** (``.sss``) a datamap that record how the data file should be read. i.e., where's the data for each question for each respondent etc.,.
* A **data file** (``.asc`` or ``.csv``)  this can either be a text .asc file that contains the raw data, or a csv that with question name as column headers. Most of the data files are of the former kind.

You can explore questions, analyse responses with pandas or Polars,
and edit and save the datamap.


Currently, the library supports Triple-S Classic and XML formats up to Triple-S 2.0.
Support for 3.0 is still in progress, though much of it should already work.

Installation
------------

Install the package with ``pip``:

.. code-block:: shell

   pip install triple-s-survey -U

Or, if you use ``uv``:

.. code-block:: shell

   uv pip install triple-s-survey -U


If no wheel matches your system, installation compiles the source. This can
take a while, especially for `Polars <https://pola.rs/>`_. Add ``-v`` to
either command to see progress.

Library Overview
----------------

Most work starts with a ``Survey``. It keeps the question definitions in
``survey.datamap`` and the respondent data in ``survey.data``, so you can
move between the questions and answers as you work.

Once installed, import the package as ``triple_s``. To get started, load
a datamap and its matching data file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   print(survey)


Finding Your Way Around
-----------------------

This documentation consists mainly three parts:

* **Contents** introduces the library. Start with
  :doc:`Survey <guides/survey>` to load a survey and look through its responses,
  then explore :doc:`SurveyData <guides/survey_data>` or
  :doc:`Datamap <guides/datamap>`.

* **Task Recipes** offers practical examples.
  See :doc:`Datamap Management <recipes/datamap>` for working with question
  definitions, or :doc:`Survey Data <recipes/survey_data>` for working with responses.

* **API Reference** describes each class and function.
  Browse :doc:`Functions <reference/functions>`, :doc:`Objects <reference/objects>`,
  or :doc:`Exceptions <reference/exceptions>`.

Use the sidebar search to find a class, method, or topic.

Architecture and Implementation
-------------------------------

The Rust core handles parsing and data processing. PyO3 and maturin provide
the Python interface.

For Rust usage, see the `Rust API documentation
<https://docs.rs/triple_s/latest/triple_s/index.html>`_.

.. toctree::
   :hidden:
   :maxdepth: 2
   :caption: Contents

   guides/survey
   guides/survey_data
   guides/datamap

.. toctree::
   :hidden:
   :maxdepth: 2
   :caption: Task Recipes

   recipes/datamap
   recipes/survey_data

.. toctree::
   :hidden:
   :maxdepth: 4
   :caption: API Reference

   reference/functions
   reference/objects
   reference/exceptions
