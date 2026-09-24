use pyo3::prelude::*;

use crate::survey::{PyText, PyVariant};

pub fn html_print_pytext(pytext: &PyText) -> PyResult<String> {
    let value = pytext.value.read();
    let variants = pytext.variants.read();

    let variants_html = variants
        .iter()
        .map(html_print_pyvariant)
        .collect::<PyResult<Vec<_>>>()?
        .join("\n");

    Ok(format!(
        r#"
    <div>
        <span style="vertical-align: top;">
            <b></b>{value}
        </span>
        <details style="
            display: inline-block;
            vertical-align: top;
            margin-left: 0.5em;
        ">
            <summary><i>variants ({count})</i></summary>
            <pre>{variants_html}</pre>
        </details>
    </div>
    "#,
        count = variants.len(),
    ))
}

pub fn html_print_pyvariant(variant: &PyVariant) -> PyResult<String> {
    let variant = variant.0.read();
    let mut meta = Vec::new();

    let mode = variant.mode.try_as_str().unwrap_or("None".into());
    meta.push(format!(
        r#"<span style="font-style: italic; opacity: 0.5;">Mode: </span><span>{mode}</span>"#
    ));
    if let Some(language) = &variant.get_language() {
        meta.push(format!(
            r#"<span style="font-family: monospace; opacity: 0.4;">Language: </span><span>{language}</span>"#
        ));
    }

    let meta = if meta.is_empty() {
        String::new()
    } else {
        format!(" | {}", meta.join(" | "))
    };

    Ok(format!(
        r#"<code>{}</code><small>{meta}</small>"#,
        variant.value,
    ))
}