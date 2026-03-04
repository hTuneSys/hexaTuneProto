# AGENTS.md

> **This file defines mandatory rules for all AI agents working in this repository.**
> Violations will result in rejected PRs and reverted changes.

---

## 🚨 ABSOLUTE REQUIREMENT: English Language Only

**This is the most important rule and must NEVER be violated.**

Everything in the project directory MUST be in English — without exception:

- Code, comments, documentation, commit messages
- Variable names, function names, type names, file names
- Error messages, log messages, test names, configuration files
- Pull request titles, descriptions, and review comments
- TODO comments, FIXME notes, and inline annotations

**Only exceptions:**

- Chat/conversation with users (any language is acceptable)

**Any PR containing non-English content in code, comments, or technical documentation will be immediately rejected.**

---

## ⚠️ CRITICAL: Commit and Push Approval

AI Agents **MUST** ask for explicit user approval before:

- Creating any git commit (even if the user requested changes)
- Pushing to any remote branch

**Required process:**

1. Complete all changes
2. Show a clear summary of what was changed
3. Ask **"May I commit?"** → Wait for explicit **yes**
4. Commit with a proper conventional commit message
5. Ask **"May I push?"** → Wait for explicit **yes**
6. Push to the remote

**Never assume permission. Always ask. No exceptions.**

---
