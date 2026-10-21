# triple-s

A parser for reading parsing [Triple-S](https://www.triple-s.org/) datamap files (.sss), backend by rust via [PyO3](https://pyo3.rs/) and [maturin](https://maturin.rs/).

Most of the core functions are completed, there are some imporvement coming in the future.

Documentations:

Rust: [docs.rs](https://docs.rs/triple_s/latest/triple_s/index.html)
Python: *under construction* [readthedocs.org](https://app.readthedocs.org/projects/triple-s/)

## Python

### Getting started

Install with:

```bash
pip install triple-s
```

#### Read a datamap

```python
from triple_s import read_datamap

datamap = read_datamap("path/to/datamap.sss")

print(datamap.title)
```

Accessing contents within a datamap:

```python
# Lookup by question name, case insensitive
print(datamap["Q1"].label)
print(datamap['q1'].label)  # this will also do
```

Edit the loaded datamap:

```python
datamap["q1"].title = "New title"
question = datamap["Q1"]
question.text = "Updated question text"
```

Datamap can be iterated:

```python
for question in datamap:
    print(question.name)
```

#### Load triple-s data file

Read `.asc` data file:

```python
from triple_s import read_datamap

datamap = read_datamap("path/to/datamap.sss")
datamap.load_data("path/to/data.asc")
```

For CSV data, use a datamap with `<record format="csv">` and call
`datamap.load_data("path/to/data.csv")`. Each question's `position start`
is its one-based CSV field number. Set `skip="1"` on the record to skip a
header, or another non-negative number to skip that many initial records.
Quoted commas, escaped quotes, and multiline text fields are supported.

...And export to polars dataframe

```python
df = datamap.to_polars()
```

#### Exporting

Export instance to sss classic (sss 1.1):

```python
string = datamap.to_classic()
print(string)
```

Alternatively, insead of the entire datamap, you can also export individual datamap components (i.e., individual questions, question response, data location) as well:

```python
question_string = datamap.record[3].to_classic()
print(question_string)
```

## Rust

Add the crate to your project, and browse relevant documentations from [docs.rs](https://docs.rs/triple_s/latest/triple_s/index.html)

```toml
[dependencies]
triple_s = "0.1"
```

## Development

Install the Python development dependencies and build the extension with:

```bash
maturin develop
```

Run the Rust tests with:

```bash
cargo test
```

## AI usage

I handwrote majorities of the code with some very *minor* support of ai, mostly on writing biolerplate getter & setters, 
and some boilerplate code realted with text parsing in `src/classic.rs`.

Otherwise most of the stuffs are writtem by myself.

## License

MIT license
