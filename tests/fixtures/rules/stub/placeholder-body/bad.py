def parse(text):
    pass  # expect: stub/placeholder-body


def load(path):
    """Read the file at path."""
    raise NotImplementedError  # expect: stub/placeholder-body


class Store:
    def save(self, key):
        raise NotImplementedError("save")  # expect: stub/placeholder-body

    def remove(self, key):
        # later
        ...  # expect: stub/placeholder-body
