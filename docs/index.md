# triple-s

Read Triple-S survey definitions (Classic or XML), load survey data, and export
it to a dataframe. Available for Python and Rust.

## Python quick start

```sh
pip install triple-s
```

```python
from triple_s import read_datamap

survey = read_datamap("survey.sss")
print(survey.title)

# Load the matching data file, using the definition in survey.sss.
survey.load_data("survey.asc")
df = survey.to_polars()
```

For CSV data, use a datamap with `<record format="csv">` and pass the CSV path
to `load_data()`. Field positions are one-based; `skip="1"` skips a header row.

## Rust quick start

```toml
[dependencies]
triple_s = "0.2"
```

```rust
use triple_s::read_datamap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let survey = read_datamap("survey.sss")?;
    println!("{:?}", survey.info());
    Ok(())
}
```

See the [API overview](api.md) for the main entry points.

## Preview these docs

Run from the repository root:

```sh
python -m pip install -r docs/requirements.txt
mkdocs serve
```

Open the local URL printed by MkDocs. To generate static HTML, run
`mkdocs build --strict`; the output goes into `site/`.
