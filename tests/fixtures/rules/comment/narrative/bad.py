# This function adds two numbers  # expect: comment/narrative
def add(a, b):
    # It returns the sum  # expect: comment/narrative
    return a + b


# First, we load the config  # expect: comment/narrative
CONFIG = {}
# Handles retries for the client  # expect: comment/narrative
RETRIES = 3
