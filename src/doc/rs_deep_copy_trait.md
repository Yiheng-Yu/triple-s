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

All structs in this library comes with `DeepCopy` trait. This trait provides
`deepcopy()` method call that actually clones the data that the `Arc`
pointers points to. This would crete a stand-alone clone of the object, and
mutating the item returned by `deepcopy()` will not update the original
data:

For an independent copy, import `DeepCopy` and call `deepcopy()`:

```rust
use triple_s::{DeepCopy, Text};
// Or:
// use triple_s::prelude::*

let original = Text::new("A");
let mut copy = original.deepcopy();

*copy.value_mut() = "B".to_string(); // will not update the original
*copy.value_mut() = "B".to_string();

assert_ne!(original.value(), copy.value());
assert_eq!(original.value(), "A".to_string());
```