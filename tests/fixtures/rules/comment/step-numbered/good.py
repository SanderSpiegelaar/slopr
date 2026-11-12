def run(data):
    """Step 1: load the data."""
    # Steps 1 and 2 share one buffer.
    rows = list(data)
    # stage1 is the warm cache, see the docs.
    return [r for r in rows if r]  # noqa: C416
