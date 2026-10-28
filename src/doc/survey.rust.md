Represents the root of a Triple-S datamap. It contains survey metadata
([`SurveyInfo`]) and the survey's questions and data ([`Record`]).

See the [Triple-S documentation](https://triple-s.org/documentation/manuals/)
for the format specifications.

# Create a survey

```rust
use triple_s::{Survey, SurveyInfo, Version};

let mut info = SurveyInfo::new(Version::V2_0);
info.name = Some("example".into());
info.title = Some("Example Survey".into());

let survey = Survey::new(info);

assert_eq!(survey.info().name.as_deref(), Some("example"));
assert_eq!(survey.info().title.as_deref(), Some("Example Survey"));
```

Use [`Survey::empty`] for a default survey with Triple-S version 1.2.
Use [`Survey::info_mut`] and [`Survey::record_mut`] to edit the metadata
and record.

# Read a survey

```no_run
use triple_s::read_triple_s;

let survey = read_triple_s("survey.sss", None::<&str>, false)?;
if let Some(question) = survey.get_question("Q1") {
    println!("Title of question 'Q1': '{:?}'", question.label());
}
# Ok::<(), anyhow::Error>(())
```

Question names are case insensitive. [`Survey::get_question`] returns
`None` when the question does not exist.

# Load and export

Use [`Survey::load_data`] to load fixed-width or CSV respondent data,
then [`Record::to_polars`] on [`Survey::record`] to create a DataFrame.
[`Survey::to_classic`] and [`Survey::to_xml`] return the datamap as a string.

Data loading supports UTF-8 and BOM-marked UTF-16. 