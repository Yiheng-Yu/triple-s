Survey object

`Survey` combines a datamap with optional respondent data.

## `datamap-only` feature

With the `datamap-only` feature enabled, `Survey` contains only the datamap
and its configuration.

In this build:

```ignore
use triple_s::Datamap;

let survey = triple_s::read_triple_s("survey.sss")?;
let survey = triple_s::Survey::new(datamap);
let survey = triple_s::Survey::empty();
```

With the default feature set, the corresponding forms accept respondent data:

```ignore
let survey = triple_s::read_triple_s("survey.sss", Some("survey.asc"))?;
let survey = triple_s::Survey::new(datamap, survey_data);
let survey = triple_s::Survey::empty();
```