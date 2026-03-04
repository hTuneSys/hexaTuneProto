<!--
SPDX-FileCopyrightText: 2025 hexaTune LLC
SPDX-License-Identifier: MIT
-->

# Pull Request Strategy

This document outlines the strategy, conventions, and rules for managing Pull Requests (PRs) in the hexaTuneProto project.

---

## PR Title Convention

All PR titles must follow one of the 12 supported types:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only changes
- `style`: Code style, formatting (no logic change)
- `refactor`: Code refactoring (no feature change)
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `ci`: Changes to CI/CD configuration
- `build`: Build system or dependencies
- `chore`: Maintenance or tooling
- `release`: Versioning, changelog updates
- `hotfix`: Emergency fix for critical issues

**Example**:

```bash
feat: add user authentication module
```

---

## PR Checklist

Every PR must fulfill the following requirements:

- [ ] Clear and descriptive title using a valid PR type
- [ ] Related issue linked (if applicable)
- [ ] Description explains the context and purpose
- [ ] No unrelated changes or mixed concerns
- [ ] All checks pass (CI, lint, test)
- [ ] Reviewers assigned
- [ ] Labels applied

---

## PR Workflow

1. **Create a feature/fix branch**  
   Use a semantic branch name like:  
   `feat/login-form`, `fix/null-crash`, `docs/api-guide`

2. **Open a Draft PR**  
   Start as draft if it's a work-in-progress

3. **Update the PR as work progresses**  
   Keep commits clean and meaningful

4. **Mark as Ready for Review**  
   Request review from maintainers

5. **Address Review Comments**  
   Resolve all conversations before merge

---

## Protection Rules

- PRs targeting protected branches (e.g., `main`, `release`) must:
  - Pass all status checks
  - Be reviewed by at least 1 maintainer
  - Use a valid PR title type

---

## What to Avoid

- PRs with vague titles like “update” or “fix stuff”
- Mixing unrelated concerns in a single PR
- Skipping the review process or checks

---

Adhering to this PR strategy ensures clean history, traceable changes, and predictable releases.
