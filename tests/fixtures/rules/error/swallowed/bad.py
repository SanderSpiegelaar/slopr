import logging

logger = logging.getLogger(__name__)


def quiet(text):
    try:
        return int(text)
    except:  # expect: error/swallowed
        pass
    try:
        return int(text)
    except ValueError:  # expect: error/swallowed
        ...
    try:
        return int(text)
    except ValueError as err:  # expect: error/swallowed
        logger.warning("bad number")
    try:
        return int(text)
    except ValueError:  # expect: error/swallowed
        print("bad number")
    return 0
