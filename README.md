# Ledger Auth

Ledger Auth is the **custom authentication server** for the Ledger project.
It provides secure token issuance and access control for Ledger.

⚠️ **Note:** This project is currently under construction. The roadmap and features are still being finalized.

## Features for MVP.
- [x] Upload
- [x] Download
- [x] Files as CDN
- [x] Better error handling.
- [x] Standard response type.
- [ ] File delete
- [ ] User create, update, and delete
- [ ] Lock files ops behind auth
- [ ] Team based auth for file access (even if solo)
- [ ] Team based admin controls, add/remove users (even if solo)
- [ ] Ability to safely share files (team member, password, or public)
- [ ] Bucket folder structure per team; team names must be unique
- [ ] File encryption at rest (SSE-C AES-256? probably SSE-C and "workspace" specific decryption)
