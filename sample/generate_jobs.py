import requests
import uuid

WATERWHEEL_HOST = 'http://localhost:8080'

requests.post(WATERWHEEL_HOST + '/api/projects', json={
    'name': 'autogen_project',
    'description': 'a project for the autogenerated test jobs'
})

template = {
    "uuid": None,
    "project": "autogen_project",
    "name": None,
    "description": "a generated job",
    "triggers": [
        {
            "name": "daily",
            "start": "2021-02-08T12:00:00Z",
            "period": "1h",
        }
    ],
    "tasks": [
        {
            "name": "step0",
            "docker": {
                "image": "python:3-alpine",
                "args": [
                    "python",
                    "-c",
                    "import random, sys; sys.exit(random.randint(0, 1))"
                ],
            },
            "depends": [
                "trigger/daily"
            ],
        },
        {
            "name": "failure",
            "depends_failure": [
                "task/step0"
            ],
            "threshold": 1
        },
        {
            "name": "success",
            "depends": [
                "task/step0"
            ],
        }
    ]
}

for i in range(500):
    job = template.copy()
    job['name'] = f'job {i}'
    job['uuid'] = str(uuid.uuid4())

    print(f'creating job {i + 500}')
    resp = requests.put(WATERWHEEL_HOST + '/api/jobs', json=job)
    resp.raise_for_status()
