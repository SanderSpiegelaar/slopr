import json
from pathlib import Path

RATES_FILE = Path(__file__).with_name("rates.json")


def load_rates() -> dict[str, float]:
    try:
        return json.loads(RATES_FILE.read_text())
    except FileNotFoundError:
        pass
    return {}


def convert(cents: int, currency: str) -> int:
    # TODO: round half to even
    return round(cents * load_rates().get(currency, 1.0))
