Library overview
================

A :class:`triple_s.Survey` is the root of a loaded datamap. It combines
:class:`triple_s.SurveyInfo` metadata with a :class:`triple_s.Record` of
questions. Each :class:`triple_s.Question` describes how to interpret a
variable in the data file.

.. list-table:: Main objects
   :header-rows: 1
   :widths: 30 70

   * - Object
     - Purpose
   * - :class:`triple_s.Survey`
     - Look up questions, load data, and export the datamap or DataFrames.
   * - :class:`triple_s.SurveyInfo`
     - Store the survey's title, name, version, and other metadata.
   * - :class:`triple_s.Record`
     - Hold questions and record settings such as data format and skipped rows.
   * - :class:`triple_s.Question`
     - Describe a variable's name, type, label, location, and allowed responses.
   * - :class:`triple_s.Response`, :class:`triple_s.Code`, :class:`triple_s.DataRange`
     - Describe response codes, their labels, and allowed value ranges.
   * - :class:`triple_s.Location`, :class:`triple_s.Spread`
     - Describe data positions and multiple-response subfield layouts.
   * - :class:`triple_s.Text`, :class:`triple_s.Variant`
     - Store default labels and specialised text for languages or usage modes.

Enums describe supported settings: :class:`triple_s.Version` for the datamap
version, :class:`triple_s.QuestionType` for variable types,
:class:`triple_s.DataSource` for data formats, :class:`triple_s.CodeFormat`
and :class:`triple_s.Value` for response values, :class:`triple_s.Usage` for
variable roles, and :class:`triple_s.Mode` for specialised text modes.

Use the sidebar for the full API reference, including methods and properties
for each object.
