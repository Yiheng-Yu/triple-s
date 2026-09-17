# from .survey import Survey, Question, Response
from .triple_s import (
    Survey,
    Meta,
    Record,
    Kind,
    Choice,
    Location,
    Response,
    read_sss_file,
)

# def read_sss_file(sss_path: str) -> Survey:
#     """this reads sss"""
#     inner = triple_s.read_sss_file(sss_path)
#     return Survey(inner)

__all__ = [
    "Survey",
    "Meta",
    "Record",
    "Question",
    "Kind",
    "Location"
    "Choice",
    "Response",
    "read_sss_file",
]