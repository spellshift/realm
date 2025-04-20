## Setup
```bash
uv venv
source .venv/bin/activate
uv sync

# Modify server_config.json with your REALM session token and domain

read -s OPENAI_API_KEY
sk-...5pgA
export OPENAI_API_KEY
mcp-cli chat --server tavern --model o4-mini --disable-filesystem
```
