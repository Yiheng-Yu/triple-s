SurveyData
==========

.. currentmodule:: triple_s

.. autoclass:: triple_s.SurveyData

Obtain data with :func:`read_data`, :meth:`SurveyData.empty`, or
:meth:`SurveyData.from_polars`. Direct construction is unsupported.

Properties
----------

.. autoattribute:: SurveyData.question_names
   :annotation: : list[str]

   List of question names in a SurveyData instance.

.. autoattribute:: SurveyData.num_response
   :annotation: : int

   Total number of respondents in a SurveyData instance.


Methods
-------

.. raw:: html

   <div class="separator">Iteration & indexing</div>


.. py:method:: SurveyData.__iter__() -> Dict[str, Any]

   Iterate data row by row, returns dictionary of {question_name: question_data}


.. py:method:: SurveyData.__getitem__(index: str | int) -> polars.Series | Dict[str, Any]
   
   Indexing accepts case-insensitive question names or row index. 
   
   For example:

   .. code-block:: python

      import pandas as pd
      from triple_s import SurveyData

      # create a `SurveyData` instance
      df = pd.DataFrame(
         {
            "num_legs": [2, 4, 8, 0],
            "num_wings": [2, 0, 0, 0],
            "num_specimen_seen": [10, 2, 1, 8],
         },
         index=["falcon", "dog", "spider", "fish"],
      )
      df.index.name = "animal"
      data = SurveyData.from_pandas(df, include_index=True)

      # get data for specific question:
      question = data["animal"]

      # ..or get data for specific row:
      first_row = data[0]


.. raw:: html

   <div class="separator">Content access</div>

.. automethod:: SurveyData.has_data() -> bool

.. raw:: html

   <div class="separator">Create instance</div>

.. automethod:: SurveyData.empty() -> SurveyData

.. automethod:: SurveyData.from_polars(frame: polars.DataFrame) -> SurveyData

.. automethod:: SurveyData.from_pandas(frame: pandas.DataFrame, include_index: bool=False) -> SurveyData

.. raw:: html

   <div class="separator">Exporting</div>

.. automethod:: SurveyData.to_polars() -> polars.DataFrame

.. automethod:: SurveyData.to_pandas() -> pandas.DataFrame

.. raw:: html

   <div class="separator">Summarise</div>

.. automethod:: SurveyData.describe() -> polars.DataFrame
