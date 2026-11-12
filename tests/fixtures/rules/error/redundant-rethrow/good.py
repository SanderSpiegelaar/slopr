import logging

logger = logging.getLogger(__name__)


def wrapped(text):
    try:
        return int(text)
    except ValueError as err:
        logger.error("bad number: %s", err)
        raise
    try:
        return int(text)
    except ValueError as err:
        raise TypeError(text) from err
