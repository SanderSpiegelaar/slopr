import logging

logger = logging.getLogger(__name__)


def loud(text):
    try:
        return int(text)
    except ValueError:
        logger.exception("bad number")
    try:
        return int(text)
    except ValueError:
        logging.error("bad number", exc_info=True)
    try:
        return int(text)
    except ValueError as err:
        logger.warning("bad number: %s", err)
    try:
        return int(text)
    except ValueError:
        return 0
