triple-s documentation
======================

Welcome to ``triple-s``! This library helps you read, explore, and edit
`Triple-S <https://www.triple-s.org/>`_ survey files in Python, with Rust
handling the work behind the scenes. 

`Triple-S <https://triple-s.org/documentation/faq/>`__ is a format for sharing
survey data along with the information needed to interpret it. 
A triple-s survey usually comes in two files:

* A **datamap** (``.sss``) describes the questions and explains how to read
  each response from the data file.
* A **data file** (``.asc`` or ``.csv``) contains the respondents' answers.
  An ``.asc`` file stores values in fixed-width fields; a ``.csv`` file
  separates them with commas.

The ``triple-s`` provides a (very!) fast access to these files. You can look through its questions, load 
the survey data to do some analysis via `pandas`/ `polars` or `huggingface datasets`, 
or make changes to the datamap and save it for later.


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

The sidebar groups the documentation into three sections, depending on what
you need:

* **Contents** introduces the main parts of the library. Start with
  :doc:`Survey <guides/survey>` to load a survey and look through its responses,
  then head to :doc:`SurveyData <guides/survey_data>` or
  :doc:`Datamap <guides/datamap>` when you want to work with either part in more detail.
* **Task Recipes** has practical examples you can follow for a particular job.
  See :doc:`Datamap Management <recipes/datamap>` for working with question
  definitions, or :doc:`Survey Data <recipes/survey_data>` for working with responses.
* **API Reference** is there when you need the details of a class or function.
  Browse :doc:`Functions <reference/functions>`, :doc:`Objects <reference/objects>`,
  or :doc:`Exceptions <reference/exceptions>`. Under Objects, the SurveyData and
  Datamap groups keep related classes together.

Within a page, use the table of contents to jump to a particular section.
If you already know the name of a class, method, or topic, try the search box
in the sidebar. On a smaller screen, open the navigation menu to find these links.

Architecture and Implementation
-------------------------------

The library's core is written in Rust, with a Python interface built using
PyO3 and maturin. This lets you work with familiar Python objects while
Rust handles parsing and data processing.

If you prefer working in Rust or want to explore how the library fits
together, the `Rust API documentation on docs.rs
<https://docs.rs/triple_s/latest/triple_s/index.html>`_ is a good place
to start. Much of the functionality exposed in Python is also available directly from Rust.

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
