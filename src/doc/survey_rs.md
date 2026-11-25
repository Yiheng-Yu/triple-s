Survey object

`Survey` combines a datamap with optional respondent data.

## `datamap-only` feature

With the `datamap-only` feature enabled, `Survey` contains only the datamap
and its configuration. All respondent-data fields and methods are compiled
out: `Survey::data`, `Survey::data_mut`, `Survey::num_response`,
`Survey::has_data`, `Survey::get_question_data`, `Survey::get_row`,
`Survey::lazy`, `Survey::to_polars`, `Survey::response_label_map`,
crosstab/summary analysis, and the separate `SurveyData`/`QuestionData`
types.

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