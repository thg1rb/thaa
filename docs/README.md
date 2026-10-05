# Thaa Documentation

`THAA-DEVELOPMENT-PROMPT.md` is the authoritative product and development direction. Supporting documents turn that direction into approved requirements, architecture, and verifiable work; they should not duplicate the prompt wholesale.

## Information areas

- `requirements/` — product scope, functional and non-functional requirements, rules, and traceability.
- `architecture/` — layer boundaries, domain/provider contracts, and data flow.
- `development/` — local setup, coding standards, and Git workflow.
- `agent/` — agent workflow, Project Skills, and review rules.
- `security/` — security guidance and threat model.
- `testing/` — test strategy and platform validation.
- `adr/` — material technical decisions.
- `work/` — active and completed implementation work items.
- `features/` and `releases/` — add only when maintained feature designs or release notes exist.

## Initial requirements baseline

- [Product Requirements](requirements/PRODUCT-REQUIREMENTS.md)
- [Functional Requirements](requirements/FUNCTIONAL-REQUIREMENTS.md)
- [Non-Functional Requirements](requirements/NON-FUNCTIONAL-REQUIREMENTS.md)
- [Product Rules](requirements/PRODUCT-RULES.md)

## Architecture baseline

- [Architecture overview](architecture/ARCHITECTURE.md)
- [Domain model](architecture/DOMAIN-MODEL.md)
- [Platform adapters](architecture/PLATFORM-ADAPTERS.md)
- [Data flow](architecture/DATA-FLOW.md)
- [Architecture Decision Records](adr/README.md)

## Security and testing baseline

- [Engineering security baseline](security/SECURITY.md)
- [Threat model](security/THREAT-MODEL.md)
- [Testing strategy](testing/TEST-STRATEGY.md)
- [Initial test catalog](testing/TEST-CASES.md)
- [Platform testing](testing/PLATFORM-TESTING.md)

## Development

- [Local setup](development/LOCAL-SETUP.md)
- [Development commands and checks](development/DEVELOPMENT.md)
- [Continuous integration](development/CI.md)
- [Coding standards and formatter ownership](development/CODING-STANDARDS.md)
- [Visual design tokens and usage](development/VISUAL-DESIGN.md)
- [Git workflow](development/GIT-WORKFLOW.md)
- [Actual application directory structure](architecture/DIRECTORY-STRUCTURE.md)

## Agent guidance

- [Agent workflow](agent/AGENT-WORKFLOW.md)
- [Read-only review guidelines](agent/REVIEW-GUIDELINES.md)
- [Project Skills and invocation guidance](agent/SKILLS.md)
- [Third-party skill notices](agent/THIRD-PARTY-NOTICES.md)

## Current state

The requirements, architecture, security, and testing baselines are established. W013 integrates native listener discovery, process inspection, identity-checked actions, refresh, Tauri IPC, a React runtime list, and tray access into the First Usable Build. W013.1 adds the accepted visual identity and process-icon enrichment. Windows generic Graceful Stop remains unsupported by design. W014 adds client-side search/filter for visible listeners by process name or exact numeric port. W015 project-root detection is user accepted and integrated into `develop`; Windows working-directory availability remains limited by the existing provider. See `work/` for current work-item and integration status.
