pub mod question;
pub mod read;
pub mod record;
pub mod response;
pub mod survey_info;
pub mod text;

use anyhow::Result;
use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyDict};
pub use question::{CodeFormat, Location, Question, QuestionType, Spread, Usage};
use quick_xml::{Writer, events::BytesText};
pub use record::{DataFormat, Record};
pub use response::{Code, Response, Value};
pub use survey_info::{SurveyInfo, Version};
pub use text::{Text, Variant};

#[cfg(feature = "pyo3")]
use crate::{
    data::SurveyData,
    datamap::record::RecordIterator,
    utils::{get_type_name, py_value_error},
};
use crate::{
    prelude::Flag,
    utils::{ArcLock, lock_it_up},
};

// Rustdoc always uses the Rust guide, including builds with Python bindings.
#[cfg_attr(any(not(feature = "pyo3"), doc), doc = include_str!("../doc/datamap.rust.md"))]
#[cfg_attr(all(feature = "pyo3", not(doc)), doc = include_str!("../doc/datamap.python.rst"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Datamap", from_py_object)
)]
#[derive(Debug, Clone)]
pub struct Datamap {
    info: ArcLock<SurveyInfo>,
    record: Record,
}

impl Datamap {
    /// Creates an empty datamap with the provided [`SurveyInfo`].
    pub fn new(info: SurveyInfo) -> Self {
        Self {
            info: lock_it_up(info),
            record: Record::default(),
        }
    }

    /// Creates an empty datamap with Triple-S version 1.2.
    pub fn empty() -> Self {
        Self {
            info: lock_it_up(SurveyInfo::new(Version::V1_2)),
            record: Record::default(),
        }
    }

    /// Collects survey meta information, i.e., survey name & title, creation
    /// date, etc.,
    #[inline]
    pub fn info(&self) -> RwLockReadGuard<'_, SurveyInfo> {
        self.info.read()
    }

    /// A read-write view of the `info` field
    #[inline]
    pub fn info_mut(&mut self) -> RwLockWriteGuard<'_, SurveyInfo> {
        self.info.write()
    }

    /// survey record. For more info please check docuentation of the [`Record`]
    /// struct.
    #[inline]
    pub fn record(&self) -> &Record {
        &self.record
    }

    #[inline]
    /// A read-write view of the [`Record`] field
    pub fn record_mut(&mut self) -> &mut Record {
        &mut self.record
    }

    /// get question by its name, case insensitive.
    pub fn get_question(
        &self,
        question_name: &str,
    ) -> Option<ArcRwLockReadGuard<RawRwLock, Question>> {
        self.record.get_question(question_name)
    }

    /// export content of datamap to triple-s classic string.
    pub fn to_classic(&self) -> Result<String> {
        let mut output = format!("SSS\nVERSION {}\n", self.info().version().to_string());
        for (name, value) in [
            ("DATE", &self.info().date().as_ref()),
            ("TIME", &self.info().time().as_ref()),
            ("ORIGIN", &self.info().origin().as_ref()),
            ("USER", &self.info().user().as_ref()),
            ("STANDARD", &self.info().standard().as_ref()),
        ] {
            if let Some(value) = value {
                output.push_str(&format!("{name} \"{value}\"\n"));
            }
        }
        output.push_str("SURVEY\n");
        for (name, value) in [
            ("NAME", self.info().name().as_ref()),
            ("TITLE", self.info().title().as_ref()),
        ] {
            if let Some(value) = value {
                output.push_str(&format!("{name} \"{value}\"\n"));
            }
        }
        output.push_str(&self.record.to_classic()?);
        output.push_str("END SURVEY\nEND SSS\n");
        Ok(output)
    }

    /// Serialises the survey as a complete Triple-S XML `<sss>` document.
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        self.write_xml(&mut writer)?;
        String::from_utf8(writer.into_inner()).map_err(anyhow::Error::from)
    }

    /// Writes survey metadata and the record, propagating serialisation and I/O
    /// errors.
    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let version = self.info().version().to_string();
        let mut element =
            writer.create_element("sss").with_attribute(("version", version.as_str()));
        if let Some(standard) = &self.info().standard().as_ref() {
            element = element.with_attribute(("options", standard.as_str()));
        }
        if !self.info().language_options().is_empty() {
            element = element.with_attribute((
                "languages",
                self.info().language_options().join(" ").as_str(),
            ));
        }
        if !self.info().mode_options().is_empty() {
            let modes = self
                .info()
                .mode_options()
                .iter()
                .map(Flag::to_string)
                .collect::<Vec<_>>()
                .join(" ");
            element = element.with_attribute(("modes", modes.as_str()));
        }
        element.write_inner_content(|children| {
            for (name, value) in [
                ("date", &self.info().date().as_ref()),
                ("time", &self.info().time().as_ref()),
                ("origin", &self.info().origin().as_ref()),
                ("user", &self.info().user().as_ref()),
                ("style", &self.info().style().as_ref()),
            ] {
                if let Some(value) = value {
                    children.create_element(name).write_text_content(BytesText::new(value))?;
                }
            }
            children.create_element("survey").write_inner_content(|survey| {
                for (name, value) in [
                    ("name", &self.info().name().as_ref()),
                    ("title", &self.info().title().as_ref()),
                ] {
                    if let Some(value) = value {
                        survey
                            .create_element(name)
                            .write_text_content(BytesText::new(value))?;
                    }
                }
                self.record.write_xml(survey).map_err(std::io::Error::other)
            })?;
            Ok(())
        })?;
        Ok(())
    }

    /// (Optional) name of the datamap
    pub fn name(&self) -> Option<String> {
        self.info().name().clone()
    }

    /// (Optional) title of the datamap
    pub fn title(&self) -> Option<String> {
        self.info().title().clone()
    }

    #[cfg(feature = "pyo3")]
    pub(crate) fn py_question_repr(&self) -> Vec<(&str, String)> {
        const MAX_NUM_COLNAME_SHOW: usize = 3;
        let colnames = self.py_get_questions();
        let mut colname_repr = colnames[0..colnames.len().min(MAX_NUM_COLNAME_SHOW)]
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<String>>()
            .join(", ");
        if colnames.len() > MAX_NUM_COLNAME_SHOW {
            colname_repr.push_str(", ...");
        }
        let colname_repr = format!("[{}]", colname_repr);
        vec![("questions", colname_repr)]
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Datamap {
    /// Create a datamap from a dictionary containing optional `survey_info`
    /// (or `info`) and `record` values. Each value may be its corresponding
    /// object or a dictionary accepted by that object's `from_dict` method.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let info = content.get_item("survey_info")?.or(content.get_item("info")?);
        let info = match info {
            None => SurveyInfo::new(Version::V1_2),
            Some(value) if value.is_none() => SurveyInfo::new(Version::V1_2),
            Some(value) => {
                if let Ok(info) = value.extract::<SurveyInfo>() {
                    info
                } else if let Ok(info) = value.extract::<Bound<'_, PyDict>>() {
                    SurveyInfo::py_from_dict(info)?
                } else {
                    let type_name = get_type_name(&value)?;
                    return Err(exceptions::PyTypeError::new_err(format!(
                        "Expect `survey_info` to be a dictionary or `SurveyInfo` instance, got: '{type_name}'"
                    )));
                }
            }
        };

        let record = match content.get_item("record")? {
            None => Record::default(),
            Some(value) if value.is_none() => Record::default(),
            Some(value) => {
                if let Ok(record) = value.extract::<PyRef<'_, Record>>() {
                    record.clone()
                } else if let Ok(record) = value.extract::<Bound<'_, PyDict>>() {
                    Record::py_from_dict(record)?
                } else {
                    let type_name = get_type_name(&value)?;
                    return Err(exceptions::PyTypeError::new_err(format!(
                        "Expect `record` to be a dictionary or `Record` instance, got: '{type_name}'"
                    )));
                }
            }
        };

        Ok(Self {
            info: lock_it_up(info),
            record,
        })
    }

    /// get question by question name, returns `None` if question name not exist.
    #[pyo3(name="get_question", signature=(question_name:"str"))]
    fn py_get_question(&self, question_name: &str) -> Option<Question> {
        self.record.py_get_question(question_name)
    }

    /// (optional) Title of the survey
    #[getter(title)]
    fn py_get_title(&self) -> Option<String> {
        self.info().title().clone()
    }

    #[setter(title)]
    fn py_set_title(&mut self, title: Option<String>) {
        *self.info_mut().title_mut() = title;
    }

    /// Datamap record. For more info please check docuentation of the `Record`
    /// class.
    #[getter(record)]
    fn py_get_record(&self) -> Record {
        self.record.clone()
    }

    /// Collects survey meta information, i.e., survey name & title, creation date, etc.,
    #[getter(info)]
    fn py_get_info(&self) -> SurveyInfo {
        self.info().clone()
    }

    #[getter(questions)]
    pub(crate) fn py_get_questions(&self) -> Vec<String> {
        self.record()
            .content()
            .iter()
            .map(|question| question.read().name())
            .collect::<Vec<String>>()
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        let name = self.info().name().clone().unwrap_or("None".to_string());
        content.push(("name", format!("{:?}", name)));

        let title = self.info().title().clone().unwrap_or("None".to_string());
        content.push(("title", format!("{:?}", title)));

        const MAX_NUM_COLNAME_SHOW: usize = 3;
        let colnames = self.py_get_questions();
        let mut colname_repr = colnames[0..colnames.len().min(MAX_NUM_COLNAME_SHOW)]
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<String>>()
            .join(", ");
        if colnames.len() > MAX_NUM_COLNAME_SHOW {
            colname_repr.push_str(", ...");
        }
        let colname_repr = format!("[{}]", colname_repr);
        content.push(("questions", colname_repr));

        let formatted = content
            .into_iter()
            .map(|(field, value)| format!("{}: {}", field, value))
            .collect::<Vec<String>>()
            .join(",\n  ");

        Ok(format!("Datamap({{\n  {}\n}})", formatted))
    }

    fn __getitem__<'py>(&self, index: &Bound<'py, PyAny>) -> PyResult<Py<PyAny>> {
        self.record().__getitem__(index)
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<RecordIterator>> {
        self.record().__iter__(py)
    }

    /// export content of datamap to triple-s classic string.
    #[pyo3(name = "to_classic")]
    fn py_to_classic(&self) -> PyResult<String> {
        self.to_classic().map_err(py_value_error)
    }

    /// Serialises the survey as a complete Triple-S XML `<sss>` document.
    #[pyo3(name = "to_xml")]
    fn py_to_xml(&self) -> PyResult<String> {
        self.to_xml().map_err(py_value_error)
    }

    /// Load an `.asc` or `.csv` file using the record's skip setting.
    /// Returns `SurveyData` and updates the record's `href` and `data_format`.
    /// The returned data is not stored on the datamap.
    #[pyo3(name="load_data", signature=(path:"str"))]
    fn py_load_data(&mut self, path: &str) -> PyResult<SurveyData> {
        self.record_mut().load_data(path).map_err(py_value_error)
    }
}

#[cfg(test)]
mod test {
    use quick_xml::{Reader, events::Event};

    use super::*;

    pub fn normalize_xml(xml: &str) -> Vec<String> {
        let mut reader = Reader::from_str(xml);
        let mut events = Vec::new();

        loop {
            match reader.read_event() {
                Ok(Event::Eof) => {
                    break;
                }
                Ok(Event::Comment(_)) => (),
                Ok(Event::Text(text)) if text.as_ref().trim().is_empty() => (),
                Ok(Event::Start(tag)) => {
                    events.push(format!("Start({})", tag.as_ref().trim()));
                }
                Ok(Event::Empty(tag)) => {
                    events.push(format!("Empty({})", tag.as_ref().trim()));
                }
                Ok(event) => events.push(format!("{event:?}")),
                Err(e) => panic!("Invalid XML: {e}"),
            }
        }
        events.sort();
        events
    }

    #[test]
    fn survey_xml_metadata_and_record_attributes() {
        let mut info = SurveyInfo::new(Version::V2_0);
        *info.name_mut() = Some("hayır olduğunu".into());
        *info.title_mut() = Some("<Title>".into());
        *info.date_mut() = Some("2026-10-03".into());
        *info.time_mut() = Some("12:30".into());
        *info.origin_mut() = Some("Exporter".into());
        *info.user_mut() = Some("User".into());
        *info.style_mut() = Some("p > b { color: red; }".into());
        *info.standard_mut() = Some("standardnames".into());
        *info.language_options_mut() = vec!["en-GB".into(), "fr".into()];
        *info.mode_options_mut() = vec![text::Mode::Interview, text::Mode::Analysis];
        let mut survey = Datamap::new(info);
        *survey.record.ident_mut() = 'V';
        *survey.record.skip_mut() = Some("1".into());
        *survey.record.href_mut() = Some("a&\"b.csv".into());
        *survey.record.data_format_mut() = Some(DataFormat::Csv);
        let xml = survey.to_xml().unwrap();
        for expected in [
            "version=\"2.0\"",
            "options=\"standardnames\"",
            "languages=\"en-GB fr\"",
            "modes=\"interview analysis\"",
            "<name>hayır olduğunu</name>",
            "<title>&lt;Title&gt;</title>",
            "<date>2026-10-03</date>",
            "<time>12:30</time>",
            "<origin>Exporter</origin>",
            "<user>User</user>",
            "<style>p &gt; b { color: red; }</style>",
            "ident=\"V\"",
            "skip=\"1\"",
            "href=\"a&amp;&quot;b.csv\"",
            "format=\"csv\"",
        ] {
            assert!(xml.contains(expected), "Missing {expected} in {xml}");
        }
    }

    #[test]
    fn survey_write_xml_err() {
        let mut survey = Datamap::empty();
        assert!(survey.to_xml().is_err());
        *survey.record.ident_mut() = 'V';
        let xml = survey.to_xml().unwrap();
        assert_eq!(
            xml,
            "<sss version=\"1.2\">\n  <survey>\n    <record ident=\"V\">\n    </record>\n  </survey>\n</sss>"
        );
        let mut buffer = [0_u8; 1];
        let mut writer = quick_xml::Writer::new(&mut buffer[..]);
        assert!(survey.write_xml(&mut writer).is_err());
    }

    /// test if error raised at lower level pops up on upper level
    #[test]
    fn survey_xml_error_travelling() {
        let survey =
            read::read_from_string(include_str!("../../tests/sss-examples/sss_xml_1.0.xml"))
                .unwrap();
        for question in survey.record.content().iter() {
            let mut question = question.write();
            if *question.question_type() == QuestionType::Character {
                question.response_mut().set_range(None);
            }
        }
        assert!(survey.record.to_xml().is_err());
        assert!(survey.to_xml().is_err());
    }

    #[test]
    fn test_read_asc() {
        const PATH_ASC: &str = "tests/sss-examples/sss-2.0/example1.asc";
        const PATH_SSS: &str = "tests/sss-examples/sss-2.0/example1.sss";

        let mut survey = crate::read_datamap(PATH_SSS).unwrap();
        let record = survey.record_mut();

        record.load_data(PATH_ASC).unwrap();
    }
}
