def noisy(text):
    try:
        return int(text)
    except ValueError:  # expect: error/redundant-rethrow
        raise
    try:
        return int(text)
    except ValueError as err:  # expect: error/redundant-rethrow
        raise err
