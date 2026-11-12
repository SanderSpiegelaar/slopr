import sys


def main(argv):
    print(" ".join(argv))
    return 0


if __name__ == "__main__":
    print("starting")
    sys.exit(main(sys.argv[1:]))
