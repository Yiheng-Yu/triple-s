# triple-s

A parser for reading parsing [Triple-S](https://www.triple-s.org/) datamap files (.sss), backend by rust via [PyO3](https://pyo3.rs/) and [maturin](https://maturin.rs/).

Most of the core functions are completed, there are some imporvement coming in the future.

Documentations:

Rust: [docs.rs](https://docs.rs/triple_s/latest/triple_s/index.html)

Python: [readthedocs.org](https://triple-s.readthedocs.io/) (*under construction, more informative documentation coming soon*)


## Rust

Add the crate to your project:

```bash
cargo add triple_s
```

More detailed documentations can be foud in [docs.rs](https://docs.rs/triple_s/latest/triple_s/index.html)

## Python

Python API mirrors that of Rusts', with some minor tweaks to accomdate python's typing system.

### Getting started

Install with:

```bash
pip install triple-s-survey
```

#### Read a triple-s survey file

Read a triple-s survey file via `read_triple_s` function:

```python
from triple_s import read_triple_s

survey = read_triple_s("path/to/datamap.sss", "path/to/data.asc")
```

More usage examples can be found in the [survey data task recipe page](https://triple-s.readthedocs.io/en/release/recipes/survey_data.html).

#### Read just the datamap

```python
from triple_s import read_datamap

datamap = read_datamap("path/to/datamap.sss")

print(datamap.title)
```

Accessing contents within a datamap:

```python
# Lookup by question name, case insensitive
print(datamap["Q1"].label)
print(datamap["q1"].label)  # this will also do
```

Edit the loaded datamap:

```python
datamap["q1"].title = "New title"
question = datamap["Q1"]
question.text = "Updated question text"
```

More usage examples can be found in the [datamap task recipe page](https://triple-s.readthedocs.io/en/release/recipes/datamap.html).


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

I handwrote majorities of the code with some very *minor* support of ai, 
mostly on writing biolerplate getter & setters, the basic html of python's documentation page, 
and some routine code realted with text parsing in `src/classic.rs`. Also, I used AI to evaluate some of the edge cases
that I missed.

Everything else, including the source code, the documentation, api interface, example usage etc., are written by myself. If any error/ typo feel free to pull request.

## License

MIT license
