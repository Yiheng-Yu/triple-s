A Triple-S datamap containing survey metadata ([`SurveyInfo`]) and question
definitions ([`Record`]). Use [`crate::Survey`] to work with a datamap and
respondent data together.

See the [Triple-S documentation](https://triple-s.org/documentation/manuals/)
for the format specifications.

# Create and edit a datamap

```rust
use triple_s::{Datamap, SurveyInfo, Version};

let mut info = SurveyInfo::new(Version::V2_0);
*info.name_mut() = Some("example".into());
*info.title_mut() = Some("Example datamap".into());

let mut datamap = Datamap::new(info);
assert_eq!(datamap.name().as_deref(), Some("example"));
assert_eq!(datamap.title().as_deref(), Some("Example datamap"));

*datamap.info_mut().title_mut() = Some("Updated datamap".into());
assert_eq!(datamap.title().as_deref(), Some("Updated datamap"));
```

[`Datamap::new`] creates an empty record with the supplied metadata.
[`Datamap::empty`] creates an empty datamap with Triple-S version 1.2.
Use [`Datamap::info_mut`] and [`Datamap::record_mut`] to edit metadata and
record definitions. Metadata accessors return lock guards; release read
guards before requesting write access to the same metadata.

# Read a datamap and access questions

[`crate::read_datamap`] reads Triple-S Classic or XML definitions without
loading respondent data, even when the record references a data file.

```no_run
use triple_s::read_datamap;

let datamap = read_datamap("survey.sss")?;
if let Some(question) = datamap.get_question("Q1") {
    println!("Question label: {:?}", question.label());
}
# Ok::<(), anyhow::Error>(())
```

Question names are case insensitive. [`Datamap::get_question`] returns a
read guard for the matching question, or `None` if it does not exist.
Use [`Datamap::record`] to access the full record.

# Load and export respondent data

Call [`Record::load_data`] through [`Datamap::record_mut`]. The file extension
selects fixed-width (`.asc`) or CSV (`.csv`) parsing. Both use the record's
question definitions and `skip` setting. CSV fields are selected by question
locations, not column names; set `skip` to skip a header line if necessary.
Data loading supports UTF-8 and BOM-marked UTF-16.

```no_run
use triple_s::read_datamap;

let mut datamap = read_datamap("survey.sss")?;
let data = datamap.record_mut().load_data("survey.asc")?;
let frame = data.to_polars()?;
# Ok::<(), anyhow::Error>(())
```

Loading returns a [`crate::SurveyData`] backed by a temporary Parquet cache
and updates the record's `href` and `data_format`. Keep the returned data
owner alive while using the cache. The returned data is not stored in the
record or attached to an existing [`crate::Survey`].

To read both files into a survey, use [`crate::read_triple_s`]:

```no_run
use triple_s::read_triple_s;

let survey = read_triple_s("survey.sss", Some("survey.asc"))?;
let frame = survey.data().to_polars()?;
let datamap = survey.datamap();
# Ok::<(), anyhow::Error>(())
```

Passing `None::<&str>` as the data path reads only the definition and creates
empty respondent data; `survey.has_data()` then returns `false`.

# Export the datamap

[`Datamap::to_classic`] and [`Datamap::to_xml`] return `Result<String>`
containing the definition, not respondent data. They do not write files.
The XML output is a complete Triple-S `<sss>` document. Classic export
returns an error for unsupported question types, including date and time
questions.
