<!-- dossier-format: 1 -->
# FND-001 · scp -p from a Windows PC sends the file mode 0666, so a file returned with -p be

- kind: finding
- status: approved
- created: 2026-11-25T07:34:44+01:00
- author: reyna
- via: reyna - claude
- approved: 2026-11-25T07:34:44+01:00 by reyna
- valid-from: 2026-11-25T07:34:44+01:00
- task: T-410
- software: OpenSSH
- code: src/remote/transfer.rs
- topic: files, remote
- related: LES-002, INV-001
- tags: scp, windows, file-mode, return
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: created
- history: 2026-11-25T07:34:44+01:00 reyna via reyna - claude: approved
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related LES-002
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related INV-001
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Observation

scp -p from a Windows PC sends the file mode 0666, so a file returned with -p became writable for everyone on the server

## Evidence

take and return from Windows OpenSSH 9.5 against an OpenSSH 10.2 sshd in WSL, 2026-11-23
