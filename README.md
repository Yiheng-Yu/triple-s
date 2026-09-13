# triple-s

A parser for reading parsing [Triple-S](https://www.triple-s.org/) survey files (.sss), backend by rust via [PyO3](https://pyo3.rs/) and [maturin](https://maturin.rs/).

Still a work in progress. The current plan is to complete support for *reading* triple-s files first, followed by implementing support for *writing* files later.

Current progression:

| SSS version | Read | Write |
| --- | --- | --- |
| <1.2 (sss classic) | ✅ | 🔧 |
| >=1.2 | 🔧 | 🔧 |

## AI usage desclaimer

I handwrote majorities of the code with *some* support of ai, mostly on writing unit tests and documentations.

I'm still learning `pyo3` hence I leaned to AI a bit on part of the codes that I am not sure how to do yet, namely `QuestionProxy`, `ResponseProxy`, and `ChoiceProxy` on the rust side (`survey.rs`). These three structs are mainly used for providing a temporary view/write access to python's `__getattr__`, `__setattr__`, `__getitem__` and `__setitem__`, they aren't really getting used in the main stuff. Also, `build.rs` is generated, the entire file is there just so that i can run `cargo test`. I've no idea why it didn't allow me to run `cargo test`, don't know how to fix it my self, so asked ai for help.

## Python

### Getting started

Install with:

```bash
pip install triple-s
```

#### Read a survey

```python
from triple_s import read_sss

survey = read_sss("path/to/survey.sss")

print(survey.title)
```

Accessing contents within a survey:

```python
print(len(survey))          # Number of variables
print(survey["Q1"].text)    # Lookup by question name, case insensitive
print(survey['q1'].text)    # this will also do
print(survey[0].name)       # Or by zero-based position
```

Edit the loaded survey:

```python
survey["q1"].title = "New title"
question = survey["Q1"]
question.text = "Updated question text"
question.response
question.responses
question.codes
```

Survey responses can be iterated or converted to a pandas `DataFrame`:

```python
choices = list(survey["Q1"].response)
choices_table = survey["Q1"].response.pandas
```

## Rust

Add the crate to your project:

```toml
[dependencies]
triple_s = "0.1"
```

Read a classic Triple-S file:

```rust
use triple_s::read_sss;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let survey = read_sss("path/to/survey.sss")?;

    println!("{}", survey.title);
    for question in &survey.record.content {
        println!("{}: {}", question.name, question.text);
    }

    Ok(())
}
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

## Format support

The public `read_sss` function currently parses the classic text Triple-S
format. XML fixtures are included in the repository, but XML parsing is not
currently exposed by `read_sss`.

## License

MIT license
