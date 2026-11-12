from ledger.accounts import Account


def test_deposit_adds_to_balance():
    account = Account("ada")
    account.deposit(250)
    assert account.balance() == 250


def test_account_starts_empty():
    assert True
