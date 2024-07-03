# Collect data and save to JSON

import os, time, random, datetime
import pathlib, requests, json

from dotenv import load_dotenv

# Load environment variables
load_dotenv()

BASE_DIR = pathlib.Path(os.path.abspath("")).resolve()
GITHUB_API_URL = "https://api.github.com"
GITHUB_TOKEN = os.getenv("GITHUB_TOKEN")
GITHUB_API_KEY = os.getenv("GITHUB_API_KEY")

repo_checked = set();

headers = {
    "Authorization": f"token {GITHUB_TOKEN}",
    "Accept": "application/vnd.github.v3+json"
}

def get_random_repository(query):
    """
    Get random repository object from GitHub.

    """

    # Backoff setting to prevent rate limit
    # backoff = (backoff * coeff_m) + (random * coeff_r)
    backoff = 1.0  # Initial backoff time
    coeff_m = 1.5  # Multiplyer coefficient
    coeff_r = 1.0  # Randomness coeffieient

    # Set up the GitHub API endpoint for searching repositories
    api_url = "https://api.github.com/search/repositories"
    headers = {
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {GITHUB_API_KEY}",
    }

    # Set up the parameters for the search query
    while True:
        params = {
            "q": query,
            "page": random.randint(1, 30),
        }

        response = requests.get(api_url, params=params, headers=headers)

        if response.status_code == 200:
            data = response.json()

            if data["items"]:
                random_repo = random.choice(data["items"])
                return random_repo

            else:
                continue

        backoff = backoff * coeff_m + random.random() * coeff_r
        time.sleep(backoff)

def get_data(repo):
    keys = [
        "full_name",
        "html_url",
        "size",
        "stargazers_count",
        "language",
        "forks_count",
        "topics",
    ]

    repo_data = {key: repo.get(key) for key in keys}
    issue_url = repo.get("issues_url").rstrip("{/number}")

    page = 1

    while True:
        response = requests.get(
            issue_url,
            params={"state": "closed", "page": page},
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {GITHUB_API_KEY}",
            },
        )

        if response.status_code == 200:
            issues = response.json()

        else:
            continue

        if len(issues) == 0:
            break

        for issue in issues:
            issue_type = "pull_request" if "pull_request" in issue else "issue"

            created_at = datetime.fromisoformat(issue.get("created_at").rstrip("Z"))
            closed_at = datetime.fromisoformat(issue.get("closed_at").rstrip("Z"))
            commented_at = closed_at

            response = requests.get(
                issue.get("timeline_url"),
                params={"per_page": 100, "page": 1},
                headers={
                    "Accept": "application/vnd.github+json",
                    "Authorization": f"Bearer {GITHUB_API_KEY}",
                },
            )

            if response.status_code == 200 and (timelines := response.json()):
                for timeline in timelines:
                    match timeline.get("event"):
                        case "reviewed":
                            time = timeline.get("submitted_at").rstrip("Z")

                        case "commented":
                            time = timeline.get("created_at").rstrip("Z")

                        case _:
                            continue

                    commented_at = datetime.fromisoformat(time)
                    break

            lifetime = closed_at - created_at
            worktime = commented_at - created_at

            #                   sec  min  hour
            SECONDS_IN_DAYS = ( 60 * 60 * 24 )

            lifetime_day: float = lifetime.total_seconds() / SECONDS_IN_DAYS
            worktime_day: float = worktime.total_seconds() / SECONDS_IN_DAYS

            with open(BASE_DIR / "results.jsonl", "a") as f:
                json.dump(
                    {
                        "repository": repo_data,
                        "issue_type": issue_type,
                        "closed_at": str(closed_at),
                        "created_at": str(created_at),

                        "lifetime": str(lifetime),
                        "worktime": str(worktime),

                        "lifetime_day": str(lifetime_day),
                        "worktime_day": str(worktime_day),
                    },
                    f,
                )
                f.write("\n")

        page += 1

def collect():
    while True:
        try:
            repo = get_random_repository(query="is:public")

            if (repo_name := repo.get("full_name")) in repo_checked:
                continue

            repo_checked.add(repo_name)

            print(f"Looking for {repo_name}...")
            get_data(repo)

        except Exception as e:
            print(e)
