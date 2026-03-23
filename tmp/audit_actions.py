import urllib.request
import json
import sys

# 修正 Windows 控制台編碼
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

url = "https://api.github.com/repos/mn12345678910/mc_translator/actions/runs?per_page=5"
try:
    with urllib.request.urlopen(url) as response:
        data = json.loads(response.read().decode('utf-8'))
        
        for run in data.get('workflow_runs', []):
            print(f"ID: {run['id']}")
            print(f"Workflow: {run['name']}")
            print(f"Status: {run['status']}")
            print(f"Conclusion: {run.get('conclusion', 'None')}")
            print(f"URL: {run['html_url']}")
            print(f"Jobs URL: {run['jobs_url']}")
            print("-" * 40)
except Exception as e:
    print("Error fetching actions:", e)
