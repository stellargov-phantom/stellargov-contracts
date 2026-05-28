# Contributing to StellarGov

Thank you for your interest in contributing to StellarGov — Modular On-Chain Governance, DAO Tooling & Treasury Management for Soroban. This guide covers everything you need to get started.

---

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](./CODE_OF_CONDUCT.md). We are committed to a welcoming, respectful, and harassment-free environment for everyone.

---

## Ways to Contribute

- 🐛 **Report a bug** — Open an issue in the relevant repository using the Bug Report template.
- 💡 **Suggest a feature** — Open a Feature Request issue with a clear description and use case.
- 🛠️ **Implement a task** — Pick up a point-weighted issue to implement a core capability.
- 📖 **Improve documentation** — Fix typos, expand integration guides, or translate articles.
- ✅ **Review pull requests** — Help review open PRs from other contributors.

---

## Drips Wave — Earn Rewards

StellarGov participates in the **Stellar Drips Wave** program. Funded issues are tagged with the `Stellar Wave` label on GitHub and detailed in the project's issue list.

**How it works:**
1. Browse funded issues at [drips.network/wave](https://www.drips.network/wave).
2. Apply to an issue on the Drips platform (log in with your GitHub account).
3. A maintainer will review your application and assign you within 24–48 hours.
4. Implement the feature, submit your Pull Request, and get it merged.
5. Earn your point rewards once the Wave cycle closes!

**Point values:**
- 🟢 **Trivial** — 100 points (docs fixes, translations, simple bugs)
- 🟡 **Medium** — 150 points (standard REST APIs, CI pipelines, UI layouts)
- 🔴 **High** — 200 points (core contract features, complete integrations, SDK architecture)

---

## Repository Setup

### Tech Stack Context:
* **Core Technologies:** TypeScript for SDK, React/Vite for app, Soroban/Rust for contracts

### Setup Commands:
```bash
# Clone the repository
git clone https://github.com/stellargov-phantom/stellargov-sdk
cd stellargov-sdk

# Run tests
cargo test (contracts) or npm test (sdk)
```

---

## Branching & Workflow

We follow a standard **feature branch** workflow:
* `main` — always stable, production-ready.
* `feat/your-feature-name` — feature branch for specific issues.
* `fix/bug-fix-name` — bug fix branch.

1. Fork the repository to your GitHub account.
2. Create a branch from `main` named after your issue: `git checkout -b feat/my-feature`
3. Commit with clear, present-tense messages: `git commit -m "add support for X"`
4. Push your branch and open a Pull Request against `main`.

---

## Pull Request Guidelines

Every Pull Request must:
* Reference the issue it resolves: `Closes #XX`
* Include comprehensive tests covering happy path, error paths, and boundary conditions.
* Pass all CI checks.
* For smart contracts: compile cleanly and pass SoroGuard static analysis scans.
* Update documentation to match any interface or behavior changes.

---

## Getting Help

* 💬 **Discussions** — Open a discussion thread on GitHub for questions.
* 🐛 **Issues** — Use GitHub Issues for bug reporting and feature requests.
* 🔒 **Security** — See [`SECURITY.md`](./SECURITY.md) for vulnerability reports.

Thank you for contributing to StellarGov! 🚀
