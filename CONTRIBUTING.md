# Contributing to Noctra

Thank you for your interest in contributing to Noctra!

## 📚 Before You Start

Please read these documents first:

- **[README.md](README.md)** - Project overview and features
- **[DESIGN.md](docs/DESIGN.md)** - Technical architecture
- **[ROADMAP.md](docs/ROADMAP.md)** - Development timeline and priorities
- **[GETTING_STARTED.md](docs/GETTING_STARTED.md)** - Setup and usage

---

## Development Setup

```bash
# Clone repository
git clone https://github.com/noctra/noctra.git
cd noctra

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

---

## Contribution Areas

### High Priority (Milestone 1)
See [ROADMAP.md Milestone 1](docs/ROADMAP.md#milestone-1-core-mvp)

- [ ] Core executor implementation
- [ ] SQLite backend
- [ ] RQL parser (see [RQL-EXTENSIONS.md](docs/RQL-EXTENSIONS.md))
- [ ] Basic REPL

### Medium Priority (Milestone 2-3)
- [ ] Form library (see [FDL2-SPEC.md](docs/FDL2-SPEC.md))
- [ ] TUI/NWM (see [DESIGN.md Section 6](docs/DESIGN.md#6-noctra-window-manager-nwm))
- [ ] Running aggregates
- [ ] PostgreSQL backend

### Documentation
- [ ] API examples (add to [API-REFERENCE.md](docs/API-REFERENCE.md))
- [ ] Tutorial improvements
- [ ] Example forms

---

## Code Standards

### Formatting
```bash
cargo fmt --all
```

### Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing
- Minimum 70% coverage for new code
- See [DESIGN.md Section 9](docs/DESIGN.md#9-testing-strategy)

---

## Pull Request Process

1. Create feature branch: `git checkout -b feature/your-feature`
2. Make changes
3. Add tests
4. Update documentation (see [Documentation Updates](#documentation-updates))
5. Run `cargo test --workspace`
6. Run `cargo clippy`
7. Run `cargo fmt --all`
8. Commit: `git commit -m "feat: your feature"`
9. Push: `git push origin feature/your-feature`
10. Create PR on GitHub

---

## Documentation Updates

When adding features, update relevant documentation:

- **New RQL features** → Update [RQL-EXTENSIONS.md](docs/RQL-EXTENSIONS.md)
- **New form features** → Update [FDL2-SPEC.md](docs/FDL2-SPEC.md)
- **API changes** → Update [API-REFERENCE.md](docs/API-REFERENCE.md)
- **Architecture changes** → Update [DESIGN.md](docs/DESIGN.md)
- **New milestones** → Update [ROADMAP.md](docs/ROADMAP.md)

---

## Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Tests only
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `chore:` - Maintenance

---

## Questions?

- Open an issue: https://github.com/noctra/noctra/issues
- Discussions: https://github.com/noctra/noctra/discussions

---

**Current Focus:** [Milestone 1 - Core MVP](docs/ROADMAP.md#milestone-1-core-mvp)
