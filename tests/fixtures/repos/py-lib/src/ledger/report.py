from ledger.accounts import Account


def summary(accounts: list[Account]) -> str:
    lines = [f"{account.owner}: {account.balance()}" for account in accounts]
    print(lines)
    return "\n".join(lines)
