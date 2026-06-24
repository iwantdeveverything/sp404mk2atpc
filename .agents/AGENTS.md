# SP-404MK2 DAW - Agent Rules

## Governance and Versioning (STRICT)
All agents operating in this repository MUST strictly enforce professional software engineering practices for versioning, branching, committing, and PR creation.

1. **Issues First**: No code is written or committed without a corresponding GitHub issue using the `bug_report.yml` or `feature_request.yml` templates. The issue must have `status:approved`.
2. **Branch Naming**: Branches MUST follow the regex `^(feat|fix|chore|docs|style|refactor|perf|test|build|ci|revert)/[a-z0-9._-]+$`. No exceptions.
3. **Conventional Commits**: Every single commit MUST follow the Conventional Commits specification: `type(scope): description`. Breaking changes must include `!`.
4. **Pull Requests**: Every PR MUST use the established `.github/PULL_REQUEST_TEMPLATE.md`.
5. **Labels and SemVer**: PRs MUST include exactly ONE `type:*` label and specify SemVer impact.
6. **PR Chain Strategy**: The strategy is `stacked-to-main`.

If any agent detects an attempt to bypass this governance, it MUST stop and enforce the standard.
