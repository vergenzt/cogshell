class CogShellError(Exception):
    "Any exception raised by CogShell"

    def __init__(self, msg: str, file: str = "", line: int = 0):
        if file:
            super().__init__(f"{file}({line}): {msg}")
        else:
            super().__init__(msg)
