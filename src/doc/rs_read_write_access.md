#
 Read-write access of field values

In order to work with python's dynamic tying, all fieds in the `Question` struct
are `Arc<parking_lot::RwLock<T>>` pointers that points towards data stored on the heap.

You should be able to access these field values via function calls `{field_name}()` that
gives you a read-only copy (`Question::name()`):

# Example

```rust
use triple_s::question::QuestionOptions;

let question = QuestionOptions::new("Q1").finish()?;
let name = question.name().clone();

assert_eq!(name, "Q1".to_string());
# Ok::<(), anyhow::Error>(())
```

or `{field_name}_mut()` that gives you a read-write copy (`Question::name_mut()`):

# Example

``` rust
use triple_s::question::QuestionOptions;

let mut question = QuestionOptions::new("Q1").finish()?;
question.name_mut() =  "Q2".to_string();

assert_eq!(question.name().clone(), "Q2".to_string());
# Ok::<(), anyhow::Error>(())
```

Note that rust has its own very strict laws on these pointers. Basically you can either have:

- Multiple read-only copy of a value

Or:

- ONE SINGLE read-write copy of a value.

parking_lot::RWLock will not issue you any complier warnings when this rule gets violated,
it would just deadlock the thread thread and thus everything freezes.

For example, this will raise an error:

```compile_fail
use triple_s::question::QuestionOptions;
///
let mut question = QuestionOptions::default().with_index(67).finish()?;
let mut label = question.label_mut(); // acquires a read-only copy
*label.value_mut() = question.name().clone();
// cannot borrow `question` as immutable because it is also borrowed as mutablerustc(E0502) 
# Ok::<(), anyhow::Error>(())
```
