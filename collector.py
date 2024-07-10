# Collect data and save to JSON
print("importing libs")

import os, time, random, datetime
import pathlib, requests, json

from dateutil import parser
from dotenv import load_dotenv

print("loading dotenv")
# Load environment variables
load_dotenv()

print("define consts")
BASE_DIR = pathlib.Path(os.path.abspath("")).resolve()
GITHUB_API_URL = "https://api.github.com"
GITHUB_API_KEY = os.getenv("GITHUB_API_KEY")
GITHUB_API_KEY2 = os.getenv("GITHUB_API_KEY2")

keys = [GITHUB_API_KEY,GITHUB_API_KEY2]
key_idx = 0

repo_checked = set();

for file in os.listdir(BASE_DIR / "data" ):
    repo_checked.add(file[:-6])

def get_random_repository(query):
    """
    Get random repository object from GitHub.

    """
    global key_idx

    # Backoff setting to prevent rate limit
    # backoff = (backoff * coeff_m) + (random * coeff_r)
    backoff = 1.0  # Initial backoff time
    coeff_m = 1.5  # Multiplyer coefficient
    coeff_r = 1.0  # Randomness coeffieient

    # Set up the GitHub API endpoint for searching repositories
    api_url = "https://api.github.com/search/repositories"
    header = {
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {keys[key_idx]}",
    }

    print("header: ",header)

    # Set up the parameters for the search query
    while True:
        params = {
            "q": query,
            "page": random.randint(1, 30),
        }

        response = requests.get(api_url, params=params, headers=header)

        if response.status_code == 200:
            data = response.json()

            if data["items"]:
                random_repo = random.choice(data["items"])
                return random_repo

            else:
                key_idx = 0 if key_idx == 1 else 1
                backoff = backoff * coeff_m + random.random() * coeff_r
                time.sleep(backoff)


def get_data(repo):
    global key_idx
    issue_url = repo.get("issues_url").rstrip("{/number}")
    name= repo.get("name")

    page = 1

    while True:
        response = requests.get(
            issue_url,
            params={"state": "closed", "page": page},
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {keys[key_idx]}",
            },
        )

        if response.status_code == 200:
            issues = response.json()
        else:
            print("\nResponse code is not 200")
            print("Response: ",response.text)
            time.sleep(60)
            key_idx = 0 if key_idx == 1 else 1
            continue

        if len(issues) == 0:
            print("There is no issue\n")
            break

        for issue in issues:
            title = issue.get("title")
            print(f"{name} <{page}> title:{title}\n")
            while True:
                timeline_response = requests.get(
                    issue.get("timeline_url"),
                    # params={"per_page": 100, "page": 1},
                    headers={
                        "Accept": "application/vnd.github+json",
                        "Authorization": f"Bearer {keys[key_idx]}",
                    },
                )
                if timeline_response.status_code == 200:
                    with open(BASE_DIR / f"data/{name}.jsonl", "a") as f:
                        issue.update({"time_line": timeline_response.json()})
                        json.dump(issue,f)
                        f.write("\n")
                        break
                else:
                    print("\nResponse code is not 200 (timeline)")
                    print("Response: ",response.text)
                    time.sleep(60)
                    key_idx = 0 if key_idx == 1 else 1

        if page > 300:
            break
        page += 1

def collect():
    while True:
        try:
            repo = get_random_repository(query="is:public")

            if (repo_name := repo.get("name")) in repo_checked:
                continue

            repo_checked.add(repo_name)

            print(f"Looking for {repo_name}...")
            get_data(repo)

        except Exception as e:
            print(e)

def main():
    print("main called")
    collect()

if __name__ == '__main__':
    main()
