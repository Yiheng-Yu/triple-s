Quick start
=====================

------------------------
Read triple-s survey
------------------------

    Datamap & csv data file (``stackoverflow_2026.sss``, ``stackoverflow_2026.csv``) are converted from 
    `the 2026 stack overflow survey <https://github.com/StackExchange/Survey>`_. 
    You can download these two files from `here <https://github.com/Yiheng-Yu/triple-s/tree/release/tests/sss-examples/sss-2.0>`_.

Read a triple-s file:

.. code-block:: python

    >>> from triple_s import read_triple_s
    >>> path_datamap = "stackoverflow_2026.sss"
    >>> path_data = "stackoverflow_2026.asc"
    >>> survey = read_triple_s(path_datamap, path_data)
    Reading datamap: '"stackoverflow_2026.sss"' raised these warnings (3):
    # | Kind         | Location            | Field      | Message
    --+--------------+---------------------+------------+-----------------------------------------------------------------------------
    1 | DataLocation | JobSat              | value/size | Field 'size' (2) exceeds the width defined in question's data location (1).
    2 | DataLocation | Compensation        | value/size | Field 'size' (19) exceeds the width defined in question's data location (1).
    3 | DataLocation | ConvertedCompYearly | value/size | Field 'size' (8) exceeds the width defined in question's data location (1).
    Reading: 30903 responses 

`read_triple_s` will load the datamap and carry out some basic checks for the datamap itself, and prints a warning message in case of inconsistencies.


--------------------------------
The `Survey` class
--------------------------------

.. code-block:: python

    >>> survey
    Survey({
        name: "SO2026"
        title: "Stack Overflow Developer Survey 2026 (converted public responses)"
        questions: ["ResponseId", "Age", "CodeMain", ...]
        num_response: 30903
        dataframe_format: "polars"
        response_display: "value"
    })


A Triple-S survey usually consists of a datamap (``.sss``) describing the
questions and a matching data file (``.asc`` or ``.csv``) containing responses.

You can access both datamap and survey data separately:

Datamap
*************

.. code-block:: python

    >>> datamap = survey.datamap
    >>> datamap


Survey data
***************

.. code-block:: python

    >>> data = survey.data
    >>> data