//! Collections of output configuration options
#[cfg(not(feature = "datamap-only"))]
mod analysis;
mod survey;
#[cfg(not(feature = "datamap-only"))]
pub use analysis::{AnalysisOptions, LabelOptions, SkipQType};
pub use survey::{DataFrameFormat, ResponseDisplay, SurveyOptions};
