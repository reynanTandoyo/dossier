<!-- dossier-format: 1 -->
# LES-002 · Test code that builds ssh or scp command lines against a real sshd, not only aga

- kind: lesson
- status: draft
- created: 2026-11-25T07:34:45+01:00
- author: reyna
- via: reyna - claude
- confidence: measured
- task: T-410
- code: src/remote/
- topic: remote
- related: INV-001, FND-001
- tags: ssh, scp, test, server
- history: 2026-11-25T07:34:45+01:00 reyna via reyna - claude: created
- history: 2026-11-25T20:40:57+01:00 reyna via reyna - claude: changed — field code
- history: 2026-11-25T20:45:14+01:00 reyna via reyna - claude: changed — field topic
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related INV-001
- history: 2026-11-25T21:45:11+01:00 reyna via reyna - claude: changed — related FND-001
- history: 2026-11-27T19:12:38+01:00 reyna via reyna - claude: changed — field tags

## Statement

Test code that builds ssh or scp command lines against a real sshd, not only against stand-ins

## Valid for

Remote mode code in src/remote

## Evidence

Stand-in tests passed while the quoted remote path failed against OpenSSH 10.2 (INV-001), and only a real run showed the 0666 file mode
