from typing import final

@final
class ToplineTable:
    """
    
    Represents one topline table
    
    Attributes:
        side: name of question to be used as the rows of the table
        top: list of question names used as columns of the table
    
    
    Note:
        This constructor is not implemented and does not return an instance.
    """
    def __new__(cls, /) -> ToplineTable:
        """
        
        Creates a new `ToplineTable` instance.
        
        `top` accepts a QuestionData, None, or an iterable
        of QuestionData objects.
        
        Raises an error if questions belong to different surveys.
        
        Note:
            This constructor is not implemented and does not return an instance.
        """
