uv venv
source .venv/bin/activate
uv sync

# Modify server_config.json with your REALM session token and domain



read -s TAVERN_AUTH_SESSION
GX2u....
export TAVERN_AUTH_SESSION

read -s ANTHROPIC_API_KEY
sk-...5pgA
export ANTHROPIC_API_KEY

export TAVERN_URL="https://tavern.example.com"

mcp-cli chat --server tavern --provider anthropic --model claude-haiku-4-5 --disable-filesystem
