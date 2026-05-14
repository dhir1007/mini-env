import requests
import json

BASE = "http://localhost:8080"

def reset():
    r = requests.post(f"{BASE}/reset", json={"task": "buggy"})
    r.raise_for_status()
    return r.json()

def step(episode_id, tool, **args):
    r = requests.post(f"{BASE}/step", json={
        "episode_id": episode_id,
        "tool": tool,
        "args": args
    })
    r.raise_for_status()
    return r.json()

def verify(episode_id):
    r = requests.post(f"{BASE}/verify", json={"episode_id": episode_id})
    r.raise_for_status()
    return r.json()

def run():
    print("=== Resetting environment ===")
    episode = reset()
    eid = episode["episode_id"]
    print(f"Episode: {eid}")
    print(f"Observation: {episode['observation']}\n")

    print("=== Step 1: Run tests to see what's failing ===")
    result = step(eid, "run_tests")
    print(result["observation"][:800])

    print("\n=== Step 2: Read the solution file ===")
    result = step(eid, "read_file", path="solution.py")
    print(result["observation"])

    print("\n=== Step 3: Fix the bug (a - b → a + b) ===")
    fixed_code = '''def add(a, b):
    return a + b  # fixed

def multiply(a, b):
    return a * b

def is_palindrome(s):
    return s == s[::-1]

def fizzbuzz(n):
    result = []
    for i in range(1, n + 1):
        if i % 15 == 0:
            result.append("FizzBuzz")
        elif i % 3 == 0:
            result.append("Fizz")
        elif i % 5 == 0:
            result.append("Buzz")
        else:
            result.append(str(i))
    return result
'''
    result = step(eid, "write_file", path="solution.py", content=fixed_code)
    print(result["observation"])

    print("\n=== Step 4: Run tests again ===")
    result = step(eid, "run_tests")
    print(result["observation"][:800])

    print("\n=== Step 5: Verify (isolated verifier) ===")
    verdict = verify(eid)
    print(json.dumps(verdict, indent=2))

if __name__ == "__main__":
    run()