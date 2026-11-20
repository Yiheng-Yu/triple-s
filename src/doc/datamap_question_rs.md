# Create a question

### Build with [`QuestionOptions`]

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

### Read a Triple-S fragment

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

# Read and edit metadata

Access question metadata through methods such as [`Question::name()`],
[`Question::label()`], and [`Question::location()`]. Setters and mutable
accessors, such as [`Question::name_mut()`] and [`Question::label_mut`],
update these values.

Cloning a question shares its lock-backed metadata, including the name, label,
location, and response contents. Changes to that shared metadata are visible
through other clones; the question index is copied independently.

# Export a question

Use [`Question::to_classic`] or [`Question::to_xml`] to return the definition
as a [`String`]:

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

Classic output omits XML-only attributes such as `usage`.

Respondent data is returned by [`read_data`](crate::read_data); use
[`SurveyData::to_polars`](crate::data::SurveyData) to export it.


# Fields

For compatibility reasons (pyo3 & python), all fields are private and cannot be accessed directly.

To access a field, use it's call method to get a read-only view (i.e., `Question::index()`), 
and use the `field_name_mut()` (i.e., `Question::index_mut()`) to get a mutable view of the field.

Here's a list of fields in a `Question` instance, more details can be found in their relevant calling methods (`field_name()`/ `field_name_mut()`):

### `index` (u32)
Index of a question in `<record>`, this corresponds to the `ident` attribute of a `<variable>` node in trile-s xml.

#### Example:

```no_run
use triple_s::question::QuestionOptions;

// Create a new question instance
let mut question = QuestionOptions::default().with_index(67).finish()?;
assert_eq!(question.index(), &67);

// To update index of an question,,
*question.index_mut() = 32;
assert_eq!(question.index(), &32);
# Ok::<(), anyhow::Error>(())
```

### `name` (String)
Name of the question.

### `label` ([`Text`])
Question wording (i.e., 'what is your age?')

#### Example: update question wording

```no_run
use triple_s::question::QuestionOptions;

let mut question = QuestionOptions::new("Q1").finish()?;

*question.label_mut().value_mut() = "What is your age?".to_string();
assert_eq!(question.label().value(), "What is your age?");
# Ok::<(), anyhow::Error>(())
```

### `location` ([`Location`])
The data location described by `<position>` child node. For more information about data location, please refer to the relevant documentations for [`Location`] struct for more info.

IMPORTANT NOTE: Data location is 1-indexed (start counting from 1 instead of 0)

### `spread` (Option<[`Spread`]>)
Optional `<spread>` layout for a multiple-response variable. Basically, `spread` field controls how to parse response data of questions with multi-choice reponses. Please refer to the relevant documentations for [`Spread`] struct for more info.

### `filter` (`Option<String>`)
Question filter, this corresponds to the `<filter>` node of a `<variable>` element.

This field is left unused anywhere else in the crate.

### `format` (Option<[`CodeFormat`]>)
Corresponds to the `use` attribute of a `<variable>` node, also used for parsing 
data for single-choice questions.

Please refer to the documentation of [`CodeFormat`] for more details.

### `usage` (Option<[`Usage`]>)
Role declared by the `use` attribute.

Please refer to the documentation of [`Usage`] for more details.

### `question_type` ([`QuestionType`])
Variable type that determines how its data and response definition are interpreted. Like `Single`/ `Multiple`/ `Character` etc.,

Direct mutable access to question_type is disabled because it would mess up data location. (technically you shouldn't convert any question type into any other quesiton types anyways).

Conversion between _some_ question types are allowed, please refer to the implementation details in [`Question::set_question_type()`] for more details. To update question type, call [`Question::set_question_type()`] instead.

Also note that `question_type` field *controls how question data gets read/ parsed*. Updating this field AFTER data been loaded will leads to very undefined/ unexpected behaviours.

### `response` ([`Response`])
Response definition, basically each of the selectable items in a single/ multi-choice question, as well as 
the data range/ maximum allowed values for quantity/ date/ time kind of questions.

