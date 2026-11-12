from abc import ABC, abstractmethod
from typing import Protocol, overload
import abc


class Shape(ABC):
    def area(self):
        raise NotImplementedError


class Base:
    @abstractmethod
    def run(self):
        pass

    @abc.abstractmethod
    def stop(self):
        ...


class Reader(Protocol):
    def read(self, n: int) -> bytes:
        ...


@overload
def scale(x: int) -> int:
    ...


@overload
def scale(x: float) -> float:
    ...


def scale(x):
    return x * 2


def describe():
    """Only a docstring."""


def check(value):
    if not value:
        raise NotImplementedError
    return value
