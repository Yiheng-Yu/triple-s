# triple-s

A parser for reading parsing [Triple-S](https://www.triple-s.org/) survey files (.sss), backend by rust via [PyO3](https://pyo3.rs/) and [maturin](https://maturin.rs/).

Still a work in progress. The current plan is to complete support for *reading* triple-s files first, followed by implementing support for *writing* files later.

Current progression:

| Files | Read | Write |
| --- | --- | --- |
| Triple-S 1.1 below (sss classic) | ✅ | ✅ |
| Triple-S 1.1 above (sss xml) | 🔧 | 🔧 |
| Processing `.asc` data file | 🔧 | 🔧 |

## Python

### Getting started

Install with:

```bash
pip install triple-s
```

#### Read a survey

```python
from triple_s import read_datamap

survey = read_datamap("path/to/survey.sss")

print(survey.title)
```

Accessing contents within a survey:

```python
# Lookup by question name, case insensitive
print(survey["Q1"].label)
print(survey['q1'].label)  # this will also do
```

Edit the loaded survey:

```python
survey["q1"].title = "New title"
question = survey["Q1"]
question.text = "Updated question text"
```

Survey can be iterated:

```python
for question in survey:
    print(question.name)
```

Survey responses can be converted to `pd.DataFrame`:

```python
choices_table = survey["Q1"].response.pandas
```

#### Exporting

Export instance to sss classic (sss 1.1):

```python
string = survey.to_sss_classic()
print(string)
```

Alternatively, insead of the entire survey, you can also export individual survey components (i.e., individual questions, question response, data location) as well:

```python
question_string = survey.record[3].to_sss_classic()
print(question_string)
```

## Rust

Add the crate to your project:

```toml
[dependencies]
triple_s = "0.1"
```

Read a Classic or XML Triple-S file:

```rust
use triple_s::read_datamap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let survey = read_datamap("path/to/survey.sss")?;

    println!("{:?}", survey.title);
    for question in &survey.record.content {
        println!("{}: {}", question.name, question.label.value);
    }

    Ok(())
}
```

Response codes keep their Rust types:

| Response | Code type | Empty constructor |
| --- | --- | --- |
| Numeric single | `u64` | `Response::new_single()` |
| Numeric multiple | `u64` | `Response::new_multiple()` |
| Literal single | `String` | `Response::new_literal_single()` |
| Quantity | `f64` | `Response::new_quantity()` |

```rust
use triple_s::{Code, Response};

fn main() -> anyhow::Result<()> {
    let response = Response::new_literal_single()?;
    response.try_add_code(Code::<String>::new("001", "First option")?)?;

    {
        let values = response.try_get_values::<String>()?;
        assert_eq!(values[0].code, "001");
    } // The read lock is released here.

    response.try_get_values_mut::<String>()?[0].label.value = "New label".into();
    Ok(())
}
```

`try_add_code` infers the type from its argument and rejects duplicates. Typed
getters return an error for incompatible responses. Read and write guards hold
the lock until dropped; release them before taking another lock on the same
response. Direct edits through mutable guards or public fields bypass validation.
Float codes must be finite when parsed, constructed, or added through checked methods.

`FromValues` has been replaced by ordinary constructors:
`Categorical::integer_single(codes)`, `Categorical::integer_multiple(codes)`, and
`Categorical::string_single(codes)`. Match the `Categorical` variant to work with
its `Values<T>` directly using `values()`, `values_mut()`, and `try_add_code()`.

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

Both `read_datamap` and `read_from_string` detect Classic or XML input. Numeric
categories retain integer codes, quantity values retain floating-point codes,
and literal single responses retain string codes (including leading zeroes).
Literal multiple responses return an error. Literal codes cannot be exported to
Classic format.

## AI usage

I handwrote majorities of the code with *some* support of ai, mostly on writing unit tests, documentation, and some boilerplate code realted with text parsing in `src/classic.rs`.

I'm still learning `pyo3` hence I leaned to AI a bit on part of the codes that I am not sure how to do yet, these should only be a very minor part of the codebase. I've no idea why it didn't allow me to run `cargo test`, don't know how to fix it my self, so asked ai for help.

## License

MIT license
