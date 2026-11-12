import pdb  # expect: debug/leftover
import os, pdb as debugger  # expect: debug/leftover


def total(items):
    print(items)  # expect: debug/leftover
    pdb.set_trace()  # expect: debug/leftover
    breakpoint()  # expect: debug/leftover
    return sum(items)
