# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html
import sys
import re
from pathlib import Path

# -- Project information --------------------------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'triple-s'
copyright = '2026, Yiheng Yu'
author = 'Yiheng Yu'

# -- General configuration ------------------------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.doctest",
    "sphinx.ext.inheritance_diagram",
    "sphinx.ext.intersphinx",
    "sphinx.ext.todo",
    "sphinx.ext.coverage",
    "sphinx.ext.ifconfig",
    "sphinx.ext.viewcode",
    ]

napoleon_use_ivar = False

templates_path = ['_templates']
source_suffix = ".rst"
master_doc = "index"

exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']



# -- Options for HTML output -------------------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'furo'
html_static_path = ['_static']
html_css_files = ['style.css']

PYTHON_API_CLASSES = {
    "Code", "CodeFormat", "DataRange", "DataFormat", "Location", "Mode",
    "Question", "QuestionType", "Record", "Response", "Spread", "Datamap", "Survey",
    "SurveyInfo", "SurveyData", "Text", "Usage", "Value", "Variant", "Version",
}
PYTHON_API_FUNCTIONS = {"read_datamap", "read_triple_s"}


def rustdoc_link(match):
    """Turn Rust-style symbol links into Sphinx links when they name Python API."""
    target = match.group(1)
    if target in PYTHON_API_CLASSES:
        return f":class:`triple_s.{target}`"
    if target in PYTHON_API_FUNCTIONS:
        return f":func:`triple_s.{target}`"
    return f"``{target}``"


def prepare_docstring(app, what, name, obj, options, lines):
    """Render the Rust bindings' Markdown examples as reStructuredText."""
    if what == "class" and name.rsplit(".", 1)[-1] == "Datamap":
        # Read the Python guide directly so Sphinx does not need a rebuilt
        # native extension after documentation-only changes.
        source = Path(__file__).resolve().parents[1] / "src/doc/survey.python.rst"
        app.env.note_dependency(str(source))
        lines[:] = source.read_text(encoding="utf-8").splitlines()
        return
    if name == "triple_s.Question.format":
        # This inherited Rust docstring has an XML example without an opening fence.
        start = next((i for i, line in enumerate(lines) if line.startswith("<variable")), None)
        if start is not None:
            lines[start:start] = ["", "```xml"]
    if name == "triple_s.Variant.language":
        lines[:] = [line.replace("`", "") for line in lines]
    result = []
    in_code = False
    for line in lines:
        if line.strip().startswith("```"):
            if in_code:
                in_code = False
                result.append("")
            else:
                language = line.strip()[3:] or "text"
                result.extend(["", f".. code-block:: {language}", ""])
                in_code = True
        elif in_code:
            result.append("   " + line)
        else:
            # Rustdoc type links and Markdown inline code.
            line = re.sub(r"\[`([^`]+)`\]", rustdoc_link, line)
            line = re.sub(r"(?<![\w:`])`([^`]+)`(?!`)", r"``\1``", line)
            heading = re.match(r"^#+ (.+)$", line)
            if heading:
                result.extend(["", heading[1], "^" * len(heading[1]), ""])
            else:
                result.append(line)
    # The Question constructor's Args block uses unindented fields.
    if name == "triple_s.Question" and "Args:" in result:
        start = result.index("Args:") + 1
        result[start:] = ["    " + line if line else "" for line in result[start:]]
    lines[:] = result


def setup(app):
    # Run before Napoleon parses the Google-style argument/attribute sections.
    app.connect("autodoc-process-docstring", prepare_docstring, priority=100)
