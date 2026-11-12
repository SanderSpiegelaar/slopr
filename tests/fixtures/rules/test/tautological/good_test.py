import unittest

from math_lib import add


def test_values():
    assert add(1, 2) == 3
    assert 1 == 2
    assert 1 == 1 == add(0, 1)
    assert False is not True
    assert 1 != 1


class MathTest(unittest.TestCase):
    def test_add(self):
        self.assertTrue(add(1, 1))
        self.assertEqual(add(1, 2), 3)
        self.assertEqual(1, 1.0)
