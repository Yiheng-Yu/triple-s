A Triple-S datamap containing survey metadata ([`SurveyInfo`]) and question
definitions ([`Record`]). Use [`Survey`] to work with a datamap and
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

[`read_datamap`] reads Triple-S Classic or XML definitions without
loading respondent data, even when the record references a data file.

```no_run
use triple_s::read_datamap;

let datamap = read_datamap("survey.sss")?;
if let Some(question) = datamap.get("Q1") {
    println!("Question label: {:?}", question.label());
}
# Ok::<(), anyhow::Error>(())
```

Question names are case insensitive. [`Datamap::get`] and [`Record::get`] return a
read guard for the matching question, or `None` if it does not exist.

Use [`Datamap::record`] to access the full record.

# Load and export respondent data

Call [`Record::load_data`] through [`Datamap::record_mut`]. The extension
selects fixed-width (`.asc`) or CSV (`.csv`) parsing, using the record's
question definitions. CSV fields use question locations.

Data loading and validation support UTF-8 and BOM-marked UTF-16. Invalid
sequences become `�` (U+FFFD), each occupying one fixed-width position.

```no_run
use triple_s::read_datamap;

let mut datamap = read_datamap("survey.sss")?;
let data = datamap.record_mut().load_data("survey.asc")?;
let frame = data.to_polars()?;
# Ok::<(), anyhow::Error>(())
```

Loading returns a [`SurveyData`] backed by a temporary Parquet cache
and updates the record's `href` and `data_format`.

To read both files into a survey, use [`read_triple_s`] instead.

# Export the datamap

[`Datamap::to_classic`] and [`Datamap::to_xml`] converts datamap to triple-s classic/ xml string respectively.