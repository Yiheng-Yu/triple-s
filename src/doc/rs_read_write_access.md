# Read and edit fields

Use field accessors to read metadata and their `_mut()` counterparts to edit it.
For example, `Question::name()` returns an owned string, while
`Question::name_mut()` returns a write guard:

```rust
use triple_s::question::QuestionOptions;

let mut question = QuestionOptions::new("Q1").finish()?;
let name = question.name();
*question.name_mut() = "Q2".to_string();

assert_eq!(name, "Q1");
assert_eq!(question.name(), "Q2");
# Ok::<(), anyhow::Error>(())
```

Some accessors return lock guards. A field can have several read guards or
one write guard at a time. Release its read guards before requesting write
access; holding both can deadlock the thread.
