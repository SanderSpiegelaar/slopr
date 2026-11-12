# TODO: handle errors  # expect: comment/untracked-todo
def load(path):
    # FIXME this breaks on Windows  # expect: comment/untracked-todo
    return open(path).read()  # HACK  # expect: comment/untracked-todo
