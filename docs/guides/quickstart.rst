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

    from triple_s import read_triple_s
    
    path_datamap = "stackoverflow_2026.sss"
    path_data = "stackoverflow_2026.asc"
    survey = read_triple_s(path_datamap, path_data)