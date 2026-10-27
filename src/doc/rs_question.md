# Creating a new question

## 1.Via [`QuestionOptions`]

To build a question:

```rust
use triple_s::{
    question::{QuestionOptions, QuestionType},
    text::Text,
};

let question = QuestionOptions::new("Q1")
    .with_index(1)
    .with_label(Text::new("Would you visit again?"))
    .with_question_type(QuestionType::Logical)
    .with_location(1, None)
    .finish()?;

assert_eq!(question.name(), "Q1");
assert_eq!(question.label().value(), "Would you visit again?");
# Ok::<(), anyhow::Error>(())
```

## 2. Read from triple-s strings

```rust
use triple_s::Question;

let fragment = r#"
<variable ident="8" type="multiple">
    <name>Q5</name>
    <label>Two favourite attractions visited</label>
    <position start="62" finish="63" />
    <spread subfields="2" />
    <values>
        <value code="1">Sherwood Forest</value>
        <value code="2">Nottingham Castle</value>
        <value code="3">&quot;Friar Tuck&quot; Restaurant</value>
        <value code="4">&quot;Maid Marion&quot; Cafe</value>
        <value code="5">Mining museum</value>
        <value code="9">Other</value>
    </values>
</variable>
"#
.trim();

let question = Question::from_string(fragment)?;
assert_eq!(question.name(), "Q5".to_string());
# Ok::<(), anyhow::Error>(())
```

# Viewing and modifying

Access question metadata through methods such as [`Question::name()`],
[`Question::label()`], and [`Question::location()`]. Setters and mutable
accessors, such as [`Question::name_mut()`] and [`Question::label_mut`],
update these values.

Cloning a question shares its lock-backed metadata, including the name, label,
location, and response contents. Changes to that shared metadata are visible
through other clones; the question index is copied independently.

# Exporting

Use [`Question::to_classic`] or [`Question::to_xml`] to serialise a question
back to either format as [`String`]:

```rust
use triple_s::{QuestionOptions, QuestionType, Text};
let question = QuestionOptions::new("Q1")
    .with_label(Text::new("Would you visit again?"))
    .with_question_type(QuestionType::Logical)
    .with_location(1, None)
    .finish()?;

let text = question.to_xml()?;
let text = question.to_classic()?;
# Ok::<(), anyhow::Error>(())
```

Note that this would ignore attributes that exists in certain version but
not in other versions (i.e., `usage` field exists in xml but not in classic).

Respondent data is stored by [`Record`]; use [`Record::to_polars`] to export it.
