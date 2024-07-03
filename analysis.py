# Generate CSV from JSON files

import os, pathlib
import json, csv

BASE_DIR = pathlib.Path(os.path.abspath("")).resolve()

def gen_csv():
    with open(BASE_DIR / "results.jsonl") as f:
        while line := f.readline():
            data = json.loads(line)

            repo = data.get("repository").get("full_name")
            lifetime_day = data.get("lifetime")
            worktime_day = data.get("worktime")
            # TODO: Add more columns

            with open(BASE_DIR / "analyze.csv", "a") as g:
                writer = csv.writer(g)
                writer.writerow([repo,lifetime_day, worktime_day])
