# Pull Request

## Summary

> Describe what this PR changes and why. Link to related issue(s).

---

## Affected Crate(s)

- [ ] hexa-tune-proto (core)
- [ ] hexa-tune-proto-embedded
- [ ] hexa-tune-proto-ffi
- [ ] Dart bindings
- [ ] Documentation
- [ ] CI / Infra

---

## PR Title Format

> PR title must follow [Conventional Commits](https://www.conventionalcommits.org/) format.
> **Allowed types:**

```text
feat, fix, chore, refactor, test, docs, ci, perf, build, release, hotfix, style
```

---

## Checklist

- [ ] Branch name follows format: `<type>/<short-description>` (e.g. `feat/streaming-parser`)
- [ ] PR title starts with an approved type
- [ ] Code passes formatting (`cargo fmt --all -- --check`)
- [ ] Clippy passes (`cargo clippy --workspace -- -D warnings`)
- [ ] Tests pass (`cargo test --workspace`)
- [ ] no_std check passes (if core/embedded changed)
- [ ] Related issues linked with `Closes #N`
- [ ] No unrelated changes included

---

## Related Issues

```text
Closes #
```

---

## Notes (Optional)

> Test instructions, protocol examples, or context useful for reviewers.
