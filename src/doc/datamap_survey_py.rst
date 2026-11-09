Datamap describes how to interpret the data file, a datamap consists of two parts:
- survey metadata in ``info`` and,
- question definitions in ``record``.

Examples:
    Read a datamap

    Load an existing datamap with :func:`triple_s.read_datamap`. Both Triple-S
    Classic and XML datamaps are supported. This reads the definition only;
    it does not load the data file referenced by the record.

    .. code-block:: python

       from triple_s import read_datamap

       datamap = read_datamap("survey.sss")
       print(datamap.title)
       print(datamap.info.name)

    Create new datamap

    Alternatively, you can build a datamap with ``Datamap.from_dict()``:

    .. code-block:: python

       from triple_s import Datamap

       datamap = Datamap.from_dict({
           "info": {"version": "2.0", "name": "example", "title": "Example survey"},
           "record": {"ident": "A"},
       })
       datamap.info.name = "updated-example"
       datamap.title = "Updated survey"
       assert datamap.info.title == "Updated survey"

    You can either pass your list of questions to the `record` child dictionary,
    or ues :func:`Datamap.record.add_question` and :func:`Datamap.record.remove_question` later too add/
    remove survey questions:

    .. code-block:: python

       datamap.add_question(q1)
       datamap.remove_question(q2)


    Access questions

    To see out all questions in a datamap, use :py:attr:`Record.questions`, which lists question names in record order.


    Load and export survey data

    :func:`triple_s.read_data`:

    *Loading and validation support UTF-8 and BOM-marked UTF-16; invalid sequences become ``�`` (U+FFFD), each occupying one fixed-width position.*

    .. code-block:: python

       from triple_s import read_data

       data = read_data(datamap, "survey.asc")
       df = data.to_polars()
       pandas_df = data.to_pandas()

    Loading returns a :class:`triple_s.SurveyData` instance.

    *To read the definition and respondent data together, use :func:`triple_s.read_triple_s`:*


    Export the datamap

    ``to_classic()`` returns Triple-S classic document

    .. code-block:: python

       classic = datamap.to_classic()


    ``to_xml()`` returns a complete Triple-S XML document:

    .. code-block:: python

       xml = datamap.to_xml()

