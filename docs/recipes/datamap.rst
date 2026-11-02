Datamap Management
=======================


Convert between Triple-s Classic and XML
-------------------------------------------------

Convert between Triple-S Classic and XML with
:meth:`triple_s.Datamap.to_classic` and :meth:`triple_s.Datamap.to_xml`:

.. code-block:: python

    from triple_s import read_datamap
    datamap = read_datamap("triple-s_xml.sss")

    content = datamap.to_xml()  # or datamap.to_classic()

Save the result:

.. code-block:: python

    with open("converted.sss", "w", encoding="utf-8") as file:
        file.write(content)

Update Datamap
---------------------

Edit a question and save the datamap:

.. code-block:: python

    from triple_s import read_datamap
    datamap = read_datamap("survey.sss")

    datamap["Q1"].label = "Updated question label"

    with open("updated survey.sss", "w", encoding="utf-8") as file:
       file.write(datamap.to_xml())


Save survey data to a csv file
-------------------------------------------------

Convert the responses to pandas and call ``to_csv()``:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   df = survey.data.to_pandas()
   df.to_csv("survey.csv", index=False)

For Polars, use ``survey.data.to_polars().write_csv("survey.csv")``.


