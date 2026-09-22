# from .survey import Survey, Question, Response
from .triple_s import (
    Categorical,
    Character,
    Choice,
    Date,
    Kind,
    Location,
    Logical,
    Meta,
    Note,
    Quantity,
    Question,
    Record,
    Response,
    Format,
    SingleLiteral,
    Survey,
    Time,
    read_file,
)

# def read_sss_file(sss_path: str) -> Survey:
#     """this reads sss"""
#     inner = triple_s.read_sss_file(sss_path)
#     return Survey(inner)

__all__ = [
    "Format",
    "Categorical",
    "Character",
    "Choice",
    "Date",
    "Kind",
    "Location",
    "Logical",
    "Meta",
    "Note",
    "Quantity",
    "Question",
    "Record",
    "Response",
    "SingleLiteral",
    "Survey",
    "Time",
    "read_file",
]