Cloning metadata objects such as `Text` shares their values:

```rust
use triple_s::Text;

let original = Text::new("A");
let mut copy = original.clone();

// Both objects share the text value.
*copy.value_mut() = "B".to_string();
assert_eq!(original.value(), copy.value());
assert_eq!(original.value(), "B".to_string());
```

For an independent copy, import `DeepCopy` and call `deepcopy()`:

```rust
use triple_s::{DeepCopy, Text};

let original = Text::new("A");
let mut copy = original.deepcopy();

*copy.value_mut() = "B".to_string();

assert_ne!(original.value(), copy.value());
assert_eq!(original.value(), "A".to_string());
```
