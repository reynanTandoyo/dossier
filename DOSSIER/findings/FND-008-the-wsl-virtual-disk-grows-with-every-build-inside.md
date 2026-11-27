<!-- dossier-format: 1 -->
# FND-008 · The WSL virtual disk grows with every build inside WSL and does not shrink by it

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- topic: release
- related: TOOL-004
- tags: wsl, disk, cargo
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related TOOL-004
- history: 2026-11-27T19:12:39+01:00 reyna via reyna - claude: changed — field tags

## Observation

The WSL virtual disk grows with every build inside WSL and does not shrink by itself

## Evidence

ext4.vhdx at 9.4 GB with 4.3 GB used in FedoraLinux-44, 2026-11-23
