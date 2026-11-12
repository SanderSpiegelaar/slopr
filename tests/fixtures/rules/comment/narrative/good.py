"""This module adds numbers."""


def add(a, b):
    """This function returns the sum."""
    # Ints past 2**63 overflow on the C side, so callers clamp.
    return a + b  # noqa: E501


# Note: this function is hot.
# Functionality here is frozen until the next release.
