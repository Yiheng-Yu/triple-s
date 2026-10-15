# triple-s

A parser for reading parsing [Triple-S](https://www.triple-s.org/) datamap files (.sss), backend by rust via [PyO3](https://pyo3.rs/) and [maturin](https://maturin.rs/).

Still a work in progress. The current plan is to complete support for *reading* triple-s files first, followed by implementing support for *writing* files later.

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

Add the crate to your project:

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

## CodeFormat support

Both `read_datamap` and `read_from_string` detect Classic or XML input. UnSigned
categories retain integer codes, quantity values retain floating-point codes,
and literal single responses retain string codes (including leading zeroes).
Literal multiple responses return an error. Literal codes cannot be exported to
Classic format.

## AI usage

I handwrote majorities of the code with *some* support of ai, mostly on writing unit tests, documentation, and some boilerplate code realted with text parsing in `src/classic.rs`.

I'm still learning `pyo3` hence I leaned to AI a bit on part of the codes that I am not sure how to do yet, these should only be a very minor part of the codebase. I've no idea why it didn't allow me to run `cargo test`, don't know how to fix it my self, so asked ai for help.

## License

MIT license
