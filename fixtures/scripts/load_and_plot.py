"""Load a CSV into the running plot-twist window and plot it (standard library only).

Usage:
    python load_and_plot.py <file.csv> <x-column> <y-column> [<y-column> ...] [--scatter]
"""

import json
import os
import sys
import urllib.error
import urllib.request


def api_base() -> str:
    path = os.path.expandvars(r"%LOCALAPPDATA%\plot-twist\api.json")
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)["base_url"]
    except FileNotFoundError:
        sys.exit("plot-twist is not running (no api.json found). Start the app first.")


def call(base: str, method: str, path: str, body: dict | None = None) -> dict:
    request = urllib.request.Request(
        base + path,
        method=method,
        data=json.dumps(body).encode() if body is not None else None,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request) as response:
            return json.load(response)
    except urllib.error.HTTPError as error:
        report = json.load(error)
        print(f"{report['code']}: {report['message']}", file=sys.stderr)
        print(f"  hint: {report['hint']}", file=sys.stderr)
        sys.exit(1)


def main() -> None:
    args = [a for a in sys.argv[1:] if a != "--scatter"]
    if len(args) < 3:
        sys.exit(__doc__)
    file, x, ys = os.path.abspath(args[0]), args[1], args[2:]
    style = "scatter" if "--scatter" in sys.argv else "line"

    base = api_base()
    loaded = call(base, "POST", "/load", {"path": file})
    dataset = loaded["dataset"]
    print(f"Loaded {dataset['file_name']} as {dataset['id']}: {dataset['row_count']:,} rows")
    for column in dataset["columns"]:
        print(f"  {column['name']} ({column['kind']}, {column['missing_count']} missing)")
    if dataset["bad_cell_total"]:
        print(f"  {dataset['bad_cell_total']:,} bad cells stored as missing")

    call(base, "PUT", "/plot", {"dataset_id": dataset["id"], "x": x, "y": ys, "style": style})
    print(f"Plotted {', '.join(ys)} against {x} ({style})")


if __name__ == "__main__":
    main()
