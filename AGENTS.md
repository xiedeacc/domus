# Project Rules

## Deployment Target

- Deploy Domus only to the NAS target. Do not treat the current development machine as the deployment host.
- Before any deploy, explicitly identify the NAS host, connection method, target paths, service names, and backup/rollback plan.
- Local systemd, local `/opt`, local `/usr/local/immich`, and local database changes are only for development smoke tests when the user explicitly asks for local testing.
- If a request says "deploy" without a target, assume NAS deployment is intended and verify the NAS details from existing project configuration or ask before changing any machine state.
