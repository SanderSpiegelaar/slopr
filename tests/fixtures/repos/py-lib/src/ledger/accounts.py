from dataclasses import dataclass, field


@dataclass
class Account:
    owner: str
    entries: list[int] = field(default_factory=list)

    def deposit(self, cents: int) -> None:
        if cents <= 0:
            raise ValueError(f"deposit must be positive, got {cents}")
        self.entries.append(cents)

    def balance(self) -> int:
        return sum(self.entries)
