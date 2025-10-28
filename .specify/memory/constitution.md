<!--
═══════════════════════════════════════════════════════════════════════════════
SYNC IMPACT REPORT: Constitution Version 0.0.1 → 0.1.0
Date: 2025-10-27
═══════════════════════════════════════════════════════════════════════════════

VERSION CHANGE: 0.0.1 → 0.1.0 (MINOR)
RATIONALE: Added new principle section "Continuous Integration & Delivery" that
elevates CI/CD from a workflow step to a core non-negotiable principle.

PRINCIPLES MODIFIED:
  None - all existing principles preserved

PRINCIPLES ADDED:
  + Continuous Integration & Delivery (new section after Simple Implementation Philosophy)

SECTIONS REMOVED:
  None

TEMPLATES REQUIRING UPDATES:
  ✅ .specify/templates/plan-template.md - reviewed, no changes needed (already has Constitution Check gate)
  ✅ .specify/templates/spec-template.md - reviewed, no changes needed (focus is on user requirements)
  ✅ .specify/templates/tasks-template.md - reviewed, no changes needed (already references CI/CD validation)

FOLLOW-UP TODOS:
  None - all dependent artifacts are aligned with new principle

═══════════════════════════════════════════════════════════════════════════════
-->

# Spec Constitution

## Core Principles

### Security & Secrets Management

- **No Secret Access**: You MUST NOT read, write, or access any files containing secrets, credentials, or API keys
- **Environment Files**: You MUST NOT open or manipulate `.env`, `.env.local`, `.env.production` or similar environment configuration files
- **Credential Management**: All API key configuration, credential storage, and secret management is handled exclusively by humans
- **Code References Only**: You may reference the existence of environment variables in code (e.g., `process.env.API_KEY`) but MUST NOT access the actual values
- **Security by Design**: When designing features requiring secrets, You MUST document where secrets should be stored and how humans should configure them, without accessing the secrets themselves

**Rationale**: Maintaining strict boundaries around secrets prevents accidental exposure, ensures security best practices, and establishes clear responsibility for sensitive information management. You operate on code structure and logic.

### Memory Bank Management

You MUST read the memory bank (/memory-bank at repo root) before starting any work and MUST update it upon completion. The memory bank contains critical project context, decisions, patterns, and learnings that ensure continuity and prevent knowledge loss. No work begins without understanding the current project state, and no work is complete without documenting changes and insights.

- **Pre-Work Reading**: MUST read all memory bank files before starting any development task
- **Post-Work Updates**: MUST update relevant memory bank files after completing work
- **Context Preservation**: All significant decisions, patterns, and learnings MUST be documented
- **Knowledge Continuity**: Memory bank serves as the single source of truth for project evolution

### Test-Driven Development

Establish a local test harness that lets you rapidly iterate and validate code in the same environment you’ll develop in.

Every feature MUST follow Test-Driven Development (TDD).
Write tests before any implementation code — always. Tests should be fast to write, easy to read, and brutally effective at catching meaningful issues.

Focus on critical paths: authentication, API calls, data persistence, and user flows. These are the tests that save you from production disasters.
Do not waste time over-testing trivial or purely mechanical operations (e.g., simple math or getters).

Remember: tests are not just checks — they are living documentation and executable specifications that define what “correct” means in your system.

### Resource Constraints Reality

- **Finite Time & Compute**: All development operates under finite time and compute constraints
- **Fast Test Feedback**: Tests MUST provide feedback in seconds, not minutes - unit tests under 10 seconds, integration tests under 30 seconds
- **Pivot on Test Failures**: If a test fails repeatedly due to setup issues after 5+ development cycles, MUST pivot to different implementation/infrastructure
- **Test Framework Flexibility**: If tests are consistently flaky or complex, MUST switch to simpler testing approach/framework
- **Resource Efficiency**: Choose testing tools and approaches that maximize feedback speed per development hour invested

### Critical Test Focus

- **End-to-End UI Tests**: High-level user journeys MUST be tested using simulators/emulators - these are the MOST critical tests
- **Authentication & API Tests**: Login failures, Firebase errors, and API integration failures MUST be tested
- **Performance Tests**: Core performance metrics MUST be continuously monitored
- **Backend Critical Tests**: Non-visible critical code (data processing, business logic) MUST be tested
- **KISS Testing**: Keep tests simple and focused - avoid complex test setups and over-engineering

### Simple Implementation Philosophy

- **KISS Principle**: Keep implementations as simple as possible while meeting requirements
- **Avoid Over-Engineering**: Resist the urge to add complexity "just in case"
- **Clear Code**: Code should be self-documenting and easy to understand
- **Minimal Dependencies**: Only add dependencies when absolutely necessary
- **Progressive Enhancement**: Start simple, add complexity only when needed

### Continuous Integration & Delivery

- **Passing CI/CD from Day One**: Repository MUST have a working CI/CD pipeline configured from the very first commit - not added later
- **Local Validation MANDATORY**: Before every commit, ALL CI/CD pipeline steps MUST pass locally on your development machine
- **Development Iteration Standard**: On each development iteration, You MUST run the complete local CI/CD validation sequence:
  - Linting (e.g., `npm run lint`, `pylint`, `cargo clippy`)
  - Formatting checks (e.g., `npm run format:check`, `black --check`, `rustfmt --check`)
  - Type checking (e.g., `npm run type-check`, `mypy`, type systems where applicable)
  - Full test suite (unit, integration, contract tests)
  - Build verification (e.g., `npm run build`, `cargo build --release`, production build)
- **No Bypassing**: NEVER commit or push code without local CI/CD validation passing
- **Pipeline Equivalence**: Local validation steps MUST mirror remote CI/CD pipeline exactly
- **Fast Feedback Loop**: CI/CD steps should provide feedback quickly (see Resource Constraints Reality)

**Rationale**: A passing CI/CD pipeline from the start establishes quality gates early, prevents technical debt accumulation, and ensures every commit meets minimum quality standards. Local validation before push prevents broken builds, reduces CI/CD queue time, and catches issues before they impact the team. This discipline transforms quality from an afterthought into a non-negotiable development habit.

## Development Workflow

### Pre-Development Requirements

- **Memory Bank Review**: MUST read all memory bank files to understand current project state
- **Design Review**: UI/UX designs MUST be reviewed before implementation
- **Technical Planning**: Architecture decisions MUST be documented
- **Test Planning**: Test strategy MUST be defined before coding begins
- **Dependency Analysis**: Impact on existing code MUST be assessed

### Development Process

1. **Read Memory Bank**: Review all memory bank files to understand current project state
2. **Write Critical Tests First**: Implement simple, valuable tests before any production code
3. **Implement Feature**: Write minimal code to pass tests
4. **Refactor**: Improve code while keeping tests green
5. **End-to-End Verification**: Test critical user journeys using simulators
6. **Local CI/CD Validation**: Run ALL CI/CD pipeline steps locally and ensure they pass
   - Run linting: `npm run lint`
   - Run formatting check: `npm run format:check`
   - Run type checking: `npm run type-check`
   - Run test suite: `npm test`
   - Run build: `npm run build`
7. **Commit & Push**: Only after local validation succeeds, commit code and push to remote
8. **Code Review**: Submit PR - remote CI/CD pipelines will validate again
9. **Update Memory Bank**: Document changes, decisions, and learnings in memory bank

### Quality Gates

- **Automated Checks**: All CI/CD checks MUST pass before merge
  - Linting and formatting
  - Type checking
  - Unit test suite
  - Integration tests
  - UI tests (where applicable)
  - Security audit
  - Performance benchmarks
- **Manual Review**: Code review MUST be completed by qualified reviewer
- **Platform Testing**: Feature MUST be tested on at least 2 platforms
- **Documentation**: README and API docs MUST be updated if needed

## Governance

### Amendment Procedure

1. **Proposal**: Any team member may propose amendments via pull request to `.specify/memory/constitution.md`
2. **Impact Analysis**: Proposer MUST include Sync Impact Report documenting affected templates and dependent artifacts
3. **Review**: Team reviews proposal against core principles and project needs
4. **Consensus**: Amendments require team consensus before adoption
5. **Version Update**: Constitution version MUST be incremented per semantic versioning:
   - **MAJOR** (X.0.0): Backward-incompatible changes, principle removals, or fundamental redefinitions
   - **MINOR** (0.X.0): New principles added, material expansions to guidance, new sections
   - **PATCH** (0.0.X): Clarifications, wording improvements, typo fixes, non-semantic refinements
6. **Propagation**: Update ALL dependent templates, documentation, and command files to maintain consistency
7. **Documentation**: Update `LAST_AMENDED_DATE` and prepend Sync Impact Report to constitution file

**Version**: 0.1.0 | **Ratified**: 2025-10-27 | **Last Amended**: 2025-10-27
