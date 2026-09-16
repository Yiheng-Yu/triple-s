import os
import triple_s
import pprint
import pytest


@pytest.fixture
def sss():
    path = os.path.join(os.path.dirname(__file__), "sss_classic.sss")
    survey = triple_s.read_sss(path)
    return survey


def test_sss_read(sss):
    meta = sss.meta
    assert meta.version == "1.1"
    assert meta.date == "13th September 2026"
    assert meta.time == "12:30"
    assert meta.origin == "Example Survey Exporter v4.2"
    assert meta.user == "Survey Administrator"
    assert meta.standard == "NAMES"

    record = sss.record
    assert record.id == "A"


def test_set_title(sss):
    sss.title = "hiiiiiiii"
    assert sss.title == "hiiiiiiii"


def test_meta_attrib_getter_setter(sss):
    attr_names = ["title", "date", "time", "version", "user", "origin", "style", "standard"]
    meta = sss.meta
    for name in attr_names:
        name = f"meta.{name}"
        setattr(meta, name, "hiiii")
        assert getattr(meta, name) == "hiiii"

        setattr(meta, name, None)
        assert getattr(meta, name) is None

        setattr(meta, name, "hiiii")
        assert getattr(meta, name) == "hiiii"


def test_meta_to_dict(sss):
    exported = sss.meta.to_dict()
    correct = dict(
        version="1.1",
        date="13th September 2026",
        time="12:30",
        origin="Example Survey Exporter v4.2",
        user="Survey Administrator",
        standard="NAMES"
        )
    assert exported == correct


def test_meta_to_sss_classic(sss):
    sss.meta.to_sss_classic()


def test_get_set_record_id(sss):
    sss.record.id = "hii"
    assert sss.record.id == "hii"

    with pytest.raises(TypeError):
        sss.record.id = None

def test_sss_content_length_check(sss):
    assert len(sss.record.content) == 24