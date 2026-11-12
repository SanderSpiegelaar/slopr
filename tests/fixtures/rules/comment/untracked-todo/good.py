"""TODO: docstrings are not comments."""


# TODO(#123): handle errors
def load(path):
    # FIXME ABC-12 this breaks on Windows
    # TODO see https://example.com/issues/9
    # todo in lowercase is prose, and XXXL is a size.
    return open(path).read()  # noqa: SIM115
