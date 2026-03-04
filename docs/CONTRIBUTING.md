<!--
SPDX-FileCopyrightText: 2025 hexaTune LLC
SPDX-License-Identifier: MIT
-->

# Contributing to hexaTuneProto

Thank you for your interest in contributing to **hexaTuneProto**!  
This document outlines how to get involved, contribute code or ideas, and follow our development process.

---

## Before You Start

Please review the following core documents:

- [Architecture](ARCHITECTURE.md)
- [Branch Strategy](BRANCH_STRATEGY.md)
- [Commit Strategy](COMMIT_STRATEGY.md)
- [PR Strategy](PR_STRATEGY.md)
- [README](../README.md)

---

## Contribution Types

- **Code:** New features, bugfixes, protocol enhancements
- **Docs:** Fixing typos, improving structure, or writing new guides
- **Tests:** Adding unit tests, golden test fixtures, integration tests
- **Issues & Feedback:** Filing issues, proposing ideas or improvements

---

## Branch Strategy

Please follow our [Branching Guide](BRANCH_STRATEGY.md)

- Always branch from `develop`
- Use one of the allowed types as prefix:  
  `feat/`, `fix/`, `chore/`, `refactor/`, `test/`, `docs/`, `ci/`, `perf/`, `build/`, `release/`, `hotfix/`, `style/`
- Example: `feat/auth-handler`, `fix/login-bug`, `docs/contributing`
- Never branch from `main`
- Only maintainers may merge to `main` or `release/*`

---

## Commit & PR Formatting

All commits and pull requests must follow [Conventional Commits](https://www.conventionalcommits.org/):

### Allowed Types

`feat`, `fix`, `chore`, `refactor`, `test`, `docs`, `ci`, `perf`, `build`, `release`, `hotfix`, `style`

### 📝 Examples

```bash
feat: add user authentication module
fix: resolve panic on empty payload
chore: remove unused dependencies
refactor: simplify scheduler logic
test: add unit tests for HexaStore
docs: improve contributing guide
ci: update GitHub Actions for linting
perf: optimize event matching engine
build: update pubspec.yaml deps and app version
release: prepare v0.2.0 release
hotfix: patch critical runtime bug
style: reformat codebase with dart format
```

PR titles must follow the same format. Title linting is enforced.

---

## PR Flow

All contributions are tracked via GitHub Issues and PRs:

1. Choose or create an issue
2. Fork the repo and branch from `develop`
3. Submit a PR with a descriptive title
4. PR flows through review and CI checks

Refer to [Labelling Strategy](LABELLING_STRATEGY.md) for issue/PR labels.

---

## CI/CD & Releases

- All PRs must pass checks (build, test, format, lint)
- PRs are merged into `develop`, then promoted to `release/*`
- Only merges into `main` trigger semantic-release automation

---

## Support & Communication

- Questions? Use [GitHub Discussions](https://github.com/hTuneSys/hexaTuneProto/discussions)
- For sensitive topics, contact [info@hexatune.com](mailto:info@hexatune.com)
- Please follow our [Code of Conduct](CODE_OF_CONDUCT.md)

We’re excited to build hexaTuneProto with your help 🚀
