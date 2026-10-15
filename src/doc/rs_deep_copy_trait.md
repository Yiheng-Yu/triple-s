
```rust
use triple_s::Text;

let original = Text::new("A");
let mut copy = original.clone();

// this mutates the original data!
*copy.value_mut() = "B".to_string();
assert_eq!(original.value(), copy.value());
assert_eq!(original.value(), "B".to_string());
```

All structs in this library comes with `DeepCopy` trait. This trait provides
`deepcopy()` method call that actually clones the data that the `Arc`
pointers points to. This would crete a stand-alone clone of the object, and
mutating the item returned by `deepcopy()` will not update the original
data:

## Example

Import `DeepCopy` trait, or call `use triple_s::prelude::*` that does the
same job:

```rust
use triple_s::{DeepCopy, Text};
// Or:
// use triple_s::prelude::*

let original = Text::new("A");
let mut copy = original.deepcopy();

*copy.value_mut() = "B".to_string(); // will not update the original

assert_ne!(original.value(), copy.value());
assert_eq!(original.value(), "A".to_string());
```