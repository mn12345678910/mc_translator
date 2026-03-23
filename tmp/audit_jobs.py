import urllib.request
import json
import sys

# 修正 Windows 控制台編碼
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

run_id = "23423768259"  # 剛才失敗的 Rust CI & Release
url = f"https://api.github.com/repos/mn12345678910/mc_translator/actions/runs/{run_id}/jobs"

try:
    with urllib.request.urlopen(url) as response:
        data = json.loads(response.read().decode('utf-8'))
        
        for job in data.get('jobs', []):
            print(f"Job Name: {job['name']}")
            print(f"Status: {job['status']}")
            print(f"Conclusion: {job.get('conclusion', 'None')}")
            print("Steps:")
            for step in job.get('steps', []):
                print(f"  - {step['name']}: {step['status']} / {step.get('conclusion', 'None')}")
            print("-" * 40)
except Exception as e:
    print("Error fetching jobs:", e)
