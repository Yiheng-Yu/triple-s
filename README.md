# triple-s

A Rust parser for reading [Triple-S](https://www.triple-s.org/) survey files (.sss).

Still a work in progress. The current plan is to complete support for *reading* triple-s files first, followed by implementing support for *writing* files later.

Current progression:

| SSS version | Read | Write |
| --- | --- | --- |
| <1.2 (sss classic) | ✅ | ✅ |
| >=1.2 | 🔧 | 🔧 |

Python bindings are intentionally not included yet. The `python/triple_s` package
is reserved for a future binding layer over the Rust model.

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
