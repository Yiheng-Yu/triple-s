# from .survey import Survey, Question, Response
from .triple_s import (
    read_sss,
    Survey,
    Meta,
    # Record,
    # Kind,
    # Choice,
)

# def read_sss(sss_path: str) -> Survey:
#     """this reads sss"""
#     inner = triple_s.read_sss(sss_path)
#     return Survey(inner)

# __all__ = [
#     "Survey",
#     "Meta",
#     "Record",
#     "Question",
#     "Kind",
#     "Choice",
#     "Response",
#     "read_sss",
# ]