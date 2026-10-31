Represents the root of a Triple-S datamap. It contains survey metadata
([`SurveyInfo`]) and the survey's question definitions ([`Record`]).

See the [Triple-S documentation](https://triple-s.org/documentation/manuals/)
for the format specifications.

# Create a datamap

```rust
use triple_s::{Datamap, SurveyInfo, Version};

let mut info = SurveyInfo::new(Version::V2_0);
*info.name_mut() = Some("example".into());
*info.title_mut() = Some("Example Datamap".into());

let survey = Datamap::new(info);

assert_eq!(survey.info().name().as_deref(), Some("example"));
assert_eq!(survey.info().title().as_deref(), Some("Example Datamap"));
```

Use [`Datamap::empty`] for a default datamap with Triple-S version 1.2.
Use [`Datamap::info_mut`] and [`Datamap::record_mut`] to edit the metadata
and record.

# Read a survey

```no_run
use triple_s::read_triple_s;

let survey = read_triple_s("survey.sss", Some("survey-data.asc"))?;
if let Some(question) = survey.datamap().get_question("Q1") {
    println!("Title of question 'Q1': '{:?}'", question.label());
}
# Ok::<(), anyhow::Error>(())
```

Question names are case insensitive. [`Datamap::get_question`] returns
`None` when the question does not exist.

# Load and export

Use [`Record::load_data`] through [`Datamap::record_mut`] to load respondent
data. It returns a [`crate::SurveyData`] and updates the record's source metadata.

```no_run
use triple_s::read_datamap;

let mut datamap = read_datamap("survey.sss")?;
let data = datamap.record_mut().load_data("survey.asc")?;
let frame = data.to_polars()?;
# Ok::<(), anyhow::Error>(())
```

[`crate::read_triple_s`] returns a [`crate::Survey`] containing both the datamap
and data. Access responses with `survey.data().to_polars()?`.
[`Datamap::to_classic`] and [`Datamap::to_xml`] return the datamap as a string.

Data loading supports UTF-8 and BOM-marked UTF-16. 