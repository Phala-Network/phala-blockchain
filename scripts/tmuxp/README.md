# Quick dev env with tmuxp

Install **tmuxp**:

- Ubuntu: `sudo apt install tmuxp`
- Other: `pip install tmuxp`

Spawn dev processes:

- local 3 nodes:

	```bash
	tmuxp load ./scripts/tmuxp/three-nodes.yaml
	```

- poc2 3 nodes:

	```bash
	CHAIN=poc2 tmuxp load ./scripts/tmuxp/three-nodes.yaml
	```

- poc2 4 nodes:

	```bash
	CHAIN=poc2 tmuxp load ./scripts/tmuxp/four-nodes.yaml
	```

Kill session in tmux by `:kill-session`
