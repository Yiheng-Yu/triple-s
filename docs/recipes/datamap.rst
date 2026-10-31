Datamap Management
=======================


Convert between Triple-s Classic and XML
-------------------------------------------------

You can easily convert between triple-s classic/ triple-s xml via 
:func:`Datamap.to_classic` and :func:`Datamap.to_xml`:

.. code-block:: python

    from triple_s import read_datamap
    datamap = read_datamap("triple-s_xml.sss")

    # or .to_classic() to conver to triple-s classic
    content = datamap.to_xml()  

And write to a file:

.. code-block:: python

    with open("converted.sss") as file:
        file.write(content)

Update Datamap
---------------------

You can simply update contents of any question like this:

.. code-block:: python

    from triple_s import read_datamap
    datamap = read_datamap("survey.sss")

    # Example edits
    datamap["Q1"].label = "Updated question label"

    # And some other edits..

    with open("updated survey.sss") as file:
       updated = datamap.to_xml()
       file.write(updated)


Save survey data to a csv file
-------------------------------------------------

You can convert survey data to a `pandas.DataFrame` or a `polars.DataFrame` instance,
and then call `to_csv` to export to a csv file:

.. code-block:: python

   from triple_s import read_triple_s

   survey = read_triple_s("survey.sss", "survey.asc")
   df = survey.data.to_pandas()  # or to_polars() to get polars dataframe
   df.to_csv("csv_file.csv")  # or df.write_csv() for polars dataframe



