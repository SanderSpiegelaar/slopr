import unittest


def test_values():
    assert True  # expect: test/tautological
    assert True, "always"  # expect: test/tautological
    assert 1 == 1  # expect: test/tautological
    assert "a" == "a"  # expect: test/tautological
    assert None == None  # expect: test/tautological


class MathTest(unittest.TestCase):
    def test_add(self):
        self.assertTrue(True)  # expect: test/tautological
        self.assertEqual(3, 3)  # expect: test/tautological
