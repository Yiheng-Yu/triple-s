"""Sphinx configuration for the Python documentation."""

import re
from pathlib import Path
from urllib.parse import urljoin, urlsplit

from docutils import nodes
from sphinx.environment.adapters.toctree import TocTree

project = "triple-s"
copyright = "2026, Yiheng Yu"
author = "Yiheng Yu"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
]

napoleon_use_ivar = False

templates_path = ["_templates"]

exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "furo"
html_static_path = ["_static"]
html_css_files = ["style.css"]
html_js_files = [
    ("code-blocks.js", {"defer": "defer"}),
    ("sidebar.js", {"defer": "defer"}),
]

PYTHON_API_CLASSES = {
    "Code",
    "CodeFormat",
    "DataRange",
    "DataFormat",
    "Location",
    "Mode",
    "Question",
    "QuestionData",
    "QuestionType",
    "Record",
    "Response",
    "Spread",
    "Datamap",
    "Survey",
    "SurveyInfo",
    "SurveyData",
    "Text",
    "Usage",
    "Value",
    "Variant",
    "Version",
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
        source = Path(__file__).resolve().parents[1] / "src/doc/datamap.python.rst"
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


def rebuild_navigation(app, env):
    """Refresh every sidebar when titles, sections, or toctrees change."""
    navigation = tuple((name, toc.pformat()) for name, toc in sorted(env.tocs.items()))
    if navigation != getattr(env, "global_navigation", None):
        env.global_navigation = navigation
        return sorted(env.found_docs)
    return []


def prepare_sidebar(app, pagename, templatename, context, doctree):
    """Keep API navigation at class/function level without changing the page TOC."""
    def sidebar_toctree(**kwargs):
        tree = TocTree(app.env).get_toctree_for(pagename, app.builder, **kwargs)
        if tree is None:
            return ""
        page_uri = app.builder.get_target_uri(pagename)
        # Work from the leaves upward so removed headings can promote their
        # retained children (for example, the functions under descriptive titles).
        for item in reversed(list(tree.findall(nodes.list_item))):
            reference = next(item.findall(nodes.reference), None)
            if reference is None:
                continue
            uri = urlsplit(urljoin(page_uri, reference.get("refuri", "")))
            if not uri.path.startswith("reference/") or not uri.fragment:
                continue
            title = reference.astext()
            symbol = uri.fragment.removeprefix("triple_s.")
            keep = (
                title in PYTHON_API_CLASSES
                or symbol in PYTHON_API_FUNCTIONS
                or symbol in {"MissingKeyError", "NoDataError"}
            )
            # A standalone class page already supplies its class-level link.
            parent_item = item.parent.parent
            parent_reference = next(parent_item.findall(nodes.reference), None)
            if parent_reference is not None and parent_reference.astext() == title:
                keep = False
            if not keep:
                children = [
                    child
                    for nested in item.children
                    if isinstance(nested, nodes.bullet_list)
                    for child in nested.children
                ]
                position = item.parent.index(item)
                item.parent[position:position + 1] = children
        return app.builder.render_partial(tree)["fragment"]

    context["sidebar_toctree"] = sidebar_toctree


def setup(app):
    # Run before Napoleon parses the Google-style argument/attribute sections.
    app.connect("autodoc-process-docstring", prepare_docstring, priority=100)
    app.connect("env-updated", rebuild_navigation)
    app.connect("html-page-context", prepare_sidebar)
