# API overview

## Python

```python
from triple_s import read_datamap

survey = read_datamap("survey.sss")
```

| Entry point | Purpose |
| --- | --- |
| `read_datamap(path)` | Read a Classic or XML definition into a `Survey`. |
| `survey.title` | Get the survey title. |
| `survey.get_question(name)` | Find a question by name, ignoring case. |
| `survey.record` | Access the survey's record and questions. |
| `survey.load_data(path)` | Load a matching ASC or CSV data file. |
| `survey.to_polars()` | Export loaded data as a Polars dataframe. |
| `survey.to_pandas()` | Export loaded data as a pandas dataframe. |
| `survey.to_classic()` | Serialize the definition to a Classic string. |
| `survey.to_xml()` | Serialize the definition to an XML string. |

Load data before exporting a dataframe. Classic export does not support literal
response codes.

## Rust

The crate root exports the main types and reading functions:

| Entry point | Purpose |
| --- | --- |
| `read_datamap(path)` | Read a definition from a file. |
| `read_from_string(text)` | Read a definition from a string. |
| `read_from_reader(reader)` | Read a definition and return its parsing log. |
| `Survey` | Survey metadata and its record. |
| `Record` | Questions and data loading. |
| `Question` | A question's definition, location, and responses. |
| `DeepCopy::deepcopy()` | Create an independent copy of an object. |

Many objects share their contents when cloned. Use `DeepCopy` when changes to
the copy should leave the original untouched.

Generate the full Rust API reference locally:

```sh
cargo doc --no-deps --open
```
