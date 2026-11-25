Response codes and an optional range for a datamap question.
This corresponds to the `<values>` child-node inside a `<variable>` tag.
The question's [`QuestionType`] determines how these values are interpreted:
categorical questions list response codes, quantity questions describe
allowed values and special codes, and character questions describe the
response size.

The Polars integration described below is turned off by the
`datamap-only` crate feature. It is available only in the default build
where the Polars dependencies are enabled.

Code values and range bounds use Polars `AnyValue<'static>`: `UInt32`,
`Int32`, `Float64`, and `StringOwned(PlSmallStr)` replace the former `Value` variants.
Import `AnyValue` and `DataType` from `polars::prelude`. To convert a
`Response`, `Code`, or `DataRange`, call `cast(&DataType::UInt32)` (or another
Polars data type). Casting follows `AnyValue::try_strict_cast`, including
truncating fractional numbers when converting to integers.
