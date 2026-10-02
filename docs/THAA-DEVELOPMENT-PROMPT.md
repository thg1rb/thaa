# Thaa — Development Prompt for AI Coding Agent

> **Project:** Thaa — Local Runtime Inspector  
> **Type:** Open Source Cross-Platform Desktop Utility  
> **Target Platforms:** macOS and Windows  
> **Primary Stack:** Tauri 2 + Rust + TypeScript + React  
> **Development Model:** AI-assisted, documentation-driven, incremental feature development  
> **Repository Workflow:** `main` → releases, `develop` → integration, task branches → implementation  

---

## 0. Purpose of This Prompt

You are the primary AI Coding Agent responsible for bootstrapping and incrementally developing **Thaa**, an open-source cross-platform desktop utility for developers.

Do **not** treat this prompt as a request to implement the entire product in one pass.

Your job is to:

1. Prepare the repository and agent environment correctly.
2. Establish documentation, architecture, coding standards, testing strategy, and development governance.
3. Freeze requirements before implementation.
4. Break the product into small traceable work items.
5. Implement work incrementally on dedicated branches.
6. Validate every change through tests, code quality, security, documentation, and review.
7. Keep macOS and Windows support as a first-class architectural requirement.
8. Preserve repository history and development discipline suitable for an open-source project.

When this prompt conflicts with convenience or speed, prefer **maintainability, traceability, reproducibility, security, and clear architecture**.

---

# 1. Product Identity

## 1.1 Name

**Thaa**

The name comes from the Thai word **ท่า**, which can refer to a harbor or port.

The name intentionally connects:

- a physical **port / harbor**
- a computer **network port**
- the product's original idea as a Port Inspector
- the broader product direction as a Local Runtime Inspector

Use the name **Thaa** consistently throughout:

- repository
- documentation
- UI copy
- code examples
- release notes
- architecture documents
- package metadata
- CI/CD
- issue and PR templates

Suggested product name:

> **Thaa — Local Runtime Inspector**

Suggested short description:

> A cross-platform desktop utility for understanding local ports, processes, projects, and development runtimes.

Do not invent alternative product names unless explicitly requested.

---

# 2. Product Vision

Thaa should answer the developer question:

> **“What is running on my machine right now, which port is it using, where did it come from, and what project/runtime does it belong to?”**

The product should connect information that is normally scattered across:

- `lsof`
- Task Manager
- Activity Monitor
- PowerShell
- terminal commands
- process managers
- Git
- IDEs
- project directories
- runtime-specific tools

Thaa should present these relationships as one understandable local runtime model.

Example conceptual relationship:

```text
Network Port
    ↓
Process
    ↓
PID
    ↓
Parent Process / Process Tree
    ↓
Working Directory
    ↓
Project Root
    ↓
Git Repository / Branch
    ↓
Runtime
    ↓
Framework / Package Manager
```

Port inspection is a **core capability**, but Thaa must not be architected as merely a graphical wrapper around `lsof`.

The long-term identity is:

> **Local Runtime Inspector**

---

# 3. Target Users

Primary users:

- software developers
- students learning software development
- full-stack developers
- backend developers
- frontend developers
- mobile developers
- developers running multiple local services
- developers working with Docker or local databases
- developers frequently encountering port conflicts

Typical use cases include:

- finding which application uses port `3000`
- determining what process owns a local listener
- locating the project folder responsible for a runtime
- opening the associated project or localhost URL
- stopping a stale development server
- understanding whether a dev server is exposed to the local network
- seeing several runtimes belonging to the same project
- understanding parent/child process relationships
- diagnosing local port conflicts

---

# 4. Product Principles

Follow these principles throughout development.

## 4.1 Local-first

Thaa must work locally without requiring:

- an account
- a cloud backend
- telemetry
- an external API
- an AI service

Do not introduce cloud dependencies unless a future requirement explicitly needs them.

## 4.2 Privacy-first

Process, command, filesystem, repository, and runtime information can be sensitive.

Default behavior should therefore be:

- local processing only
- no hidden analytics
- no automatic upload of process or filesystem data
- no unnecessary network requests
- clear permission boundaries

## 4.3 Cross-platform by architecture

macOS and Windows support must be designed into the architecture, not added later by scattering OS conditionals throughout the codebase.

## 4.4 Progressive disclosure

The default interface should remain understandable.

Basic users should see:

- project/runtime
- port
- process
- essential actions

Advanced details such as:

- executable path
- full command
- parent tree
- network binding
- runtime metadata

may appear in detail views.

## 4.5 Safe process control

Stopping processes is destructive.

Prefer:

1. graceful stop
2. clear confirmation where appropriate
3. force stop only as an explicit fallback

Never silently kill unrelated processes.

## 4.6 Explainability

When Thaa detects a project, runtime, framework, or network exposure, the system should be able to explain which observable evidence caused that classification where practical.

---

# 5. Supported Platforms

Initial supported desktop operating systems:

- macOS
- Windows

Linux is **not** a v1 requirement.

However, shared code should avoid unnecessary assumptions that would make Linux support impossible later.

Do not increase current implementation scope by adding Linux unless explicitly requested.

---

# 6. Technology Direction

The planned primary stack is:

## Desktop Framework

- **Tauri 2**

## Core / System Layer

- **Rust**

## Frontend

- **TypeScript**
- **React**

## Supporting Technologies

Use only when justified:

- `sysinfo` or equivalent abstraction for shared process metadata
- native platform APIs where shared libraries are insufficient
- Git CLI or a suitable Git library
- Tauri tray integration
- Tauri autostart plugin where appropriate
- native platform packaging/signing mechanisms
- GitHub Actions

Do not change the main stack without:

1. documenting the problem,
2. documenting alternatives,
3. writing an ADR,
4. receiving explicit approval when the change is material.

---

# 7. Architectural Direction

Use a clean, modular architecture with strong boundaries between:

- domain
- application/use cases
- infrastructure
- platform adapters
- Tauri commands
- frontend presentation

The architecture should resemble a Clean / Hexagonal style without introducing unnecessary abstraction.

Suggested conceptual structure:

```text
Frontend UI
    │
    ↓
Tauri Commands / Application Boundary
    │
    ↓
Application Use Cases
    │
    ↓
Domain Interfaces / Ports
    │
    ├──────── Shared Infrastructure
    │
    ├──────── macOS Adapters
    │
    └──────── Windows Adapters
```

The domain layer must not directly depend on:

- shell command output
- macOS-only types
- Windows-only types
- Tauri UI details
- React details

---

# 8. Platform Adapter Rule

Operating-system-specific behavior must be isolated behind interfaces.

Examples:

```text
PortProvider
ProcessProvider
ProcessController
TerminalIntegration
FileManagerIntegration
StartupIntegration
NotificationIntegration
PlatformCapabilitiesProvider
```

Conceptual example:

```rust
trait PortProvider {
    fn listeners(&self) -> Result<Vec<NetworkListener>, PortProviderError>;
}
```

Implementations may include:

```text
MacOSPortProvider
WindowsPortProvider
```

Do not use platform checks broadly across business logic such as:

```rust
if cfg!(target_os = "windows") {
    ...
} else {
    ...
}
```

OS-specific conditional compilation must be concentrated inside platform/infrastructure modules.

---

# 9. Port Inspection Strategy

## 9.1 Domain requirement

Port inspection must normalize platform-specific data into a shared domain model.

Example conceptual model:

```rust
struct NetworkListener {
    protocol: NetworkProtocol,
    local_address: IpAddr,
    local_port: u16,
    pid: Option<u32>,
    state: ListenerState,
}
```

The exact model may evolve during architecture design.

## 9.2 macOS

The initial implementation **may use `lsof`** to accelerate MVP development.

However:

- parsing must be isolated in the macOS infrastructure adapter
- domain/application code must not depend on `lsof` output
- the architecture must allow replacement with native APIs later
- failure and missing permission cases must be handled gracefully

Potential future native areas include:

- Darwin / POSIX APIs
- `libproc`
- `sysctl`
- other appropriate macOS APIs

Do not prematurely replace a working MVP implementation unless evidence justifies it.

## 9.3 Windows

Prefer native Windows networking APIs rather than making PowerShell the permanent core implementation.

Expected direction:

- IP Helper API
- `GetExtendedTcpTable`
- IPv4 and IPv6 where supported
- PID ownership mapping

PowerShell may be useful only for diagnostics or prototypes, not as the long-term architecture unless strongly justified.

---

# 10. Shared Process Model

Thaa should normalize process information into a shared representation.

Potential fields:

```text
PID
Parent PID
Process Name
Executable Path
Command / Arguments
Working Directory
CPU Usage
Memory Usage
Start Time / Uptime
```

Some fields may be unavailable because of:

- operating system restrictions
- permissions
- protected processes
- sandboxing
- insufficient privileges

Do not treat unavailable information as an application failure.

Represent unavailable fields explicitly and gracefully.

---

# 11. Platform Capability Model

Because macOS and Windows differ, the application should expose capabilities rather than assume every feature exists everywhere.

Conceptual example:

```rust
struct PlatformCapabilities {
    can_read_command_line: bool,
    can_read_working_directory: bool,
    can_read_parent_process: bool,
    can_graceful_stop: bool,
    can_force_stop: bool,
}
```

The final shape should be decided during architecture work.

The UI should respond gracefully when a capability is unavailable.

Example:

```text
Working Directory
Unavailable

The operating system denied access to this process.
```

Do not fabricate missing information.

---

# 12. Cross-platform Terminology

Avoid exposing Unix-specific concepts as product-level wording when a neutral term exists.

Prefer:

- **Stop Process**
- **Force Stop Process**

Do not make these primary UI labels:

- SIGTERM
- SIGKILL

Similarly:

```text
Tray Application
├── macOS Menu Bar
└── Windows System Tray
```

Use OS-specific terminology only when it helps the user on that platform.

---

# 13. Core Feature Scope

The following list defines the product direction.

Do not implement all features at once.

---

## 13.1 P0 — Core MVP

Required foundation:

### Port Monitoring

- detect listening TCP ports
- display local address
- display port
- map listener to PID when possible
- distinguish localhost-only vs broader binding when possible
- manual refresh
- configurable or sensible auto refresh
- search
- filter

### Process Inspection

- process name
- PID
- executable path where available
- command / arguments where available
- working directory where available

### Actions

- open `localhost:<port>` or appropriate local URL
- copy URL
- copy port
- copy PID
- graceful stop
- explicit force stop fallback

### Tray

- macOS Menu Bar integration
- Windows System Tray integration
- quick runtime/port overview
- basic actions

### UX

- empty state
- permission-denied state
- unavailable metadata state
- process-ended-between-refreshes state
- clear errors
- loading state

---

# 14. P1 — Project and Process Context

After the core is stable:

### Project Detection

- working directory inspection
- project root detection
- `.git`
- `package.json`
- `Cargo.toml`
- `go.mod`
- `pom.xml`
- Gradle project markers
- `pyproject.toml`
- other justified project markers

### Git

- detect repository
- repository root
- current branch

Do not add full Git client functionality.

### Resource Information

- CPU
- memory
- uptime/start time

### Process Relationship

- parent process
- process tree

### Developer Actions

- reveal project folder
- open terminal at project
- open preferred IDE/editor where appropriate

---

# 15. P2 — Runtime Intelligence

After P0/P1 stability:

### Runtime Detection

Potential runtime types:

- Node.js
- Java
- Python
- Go
- Rust
- Ruby
- PHP

Only claim a runtime when evidence is reliable.

### Framework Detection

Possible examples:

- Next.js
- Vite
- NestJS
- Spring Boot
- Django
- Flask
- FastAPI

Framework detection should rely on observable project/process evidence.

### Package Manager Detection

Possible examples:

- npm
- pnpm
- Yarn
- Bun
- Maven
- Gradle

### Runtime Version

Best-effort only.

Do not execute arbitrary project code merely to identify versions.

---

# 16. P2 — Port Conflict and Network Awareness

### Port Conflict

Thaa should help answer:

> What is already using the port I need?

Potential capabilities:

- search specific port
- show owner
- show project context
- open existing service
- stop existing process safely

Do not invent automatic port conflict interception unless technically justified.

### Network Exposure

Classify common bindings such as:

```text
127.0.0.1 / ::1
→ Local only
```

```text
0.0.0.0 / ::
→ Potentially reachable from other interfaces
```

Warnings must be factual and non-alarmist.

Example:

> This development server may be reachable by other devices on the same network.

Do not claim internet exposure solely from `0.0.0.0`.

---

# 17. P3 — Watch and Notification Features

Future scope:

- favorite ports
- watch ports
- runtime started notification
- runtime stopped notification
- port conflict notification

Notifications should be opt-in or appropriately configurable.

Avoid noisy defaults.

---

# 18. P3 — Docker / Container Context

Future scope may include:

- detect Docker-related listeners
- identify container
- container name
- host-to-container port mapping
- group runtime context

Do not turn Thaa into a full Docker management client.

Container stop/start actions require separate evaluation and safety design.

---

# 19. Out of Scope for Initial Development

Do not add without explicit requirement:

- cloud accounts
- cloud sync
- telemetry
- AI APIs
- remote machine monitoring
- packet sniffing
- full network analyzer
- full Activity Monitor replacement
- full Docker manager
- Kubernetes management suite
- IDE replacement
- terminal replacement
- Git client
- Linux release
- automatic code execution inside detected projects

Protect the product's focus.

---

# 20. Suggested UI Direction

Thaa should focus on **runtime context**, not raw port rows only.

Example conceptual runtime card:

```text
● Example Frontend

Next.js
localhost:3000

Process
node · PID 48312

Project
~/Projects/example/frontend

Git
feature/dashboard

CPU       2.7%
Memory    184 MB
Uptime    1h 27m

Network
Local only

[Open] [Terminal] [Stop]
```

Windows should show equivalent information using platform-appropriate values such as `node.exe`.

The data model should remain shared.

---

# 21. Git Branch Strategy

The repository uses a structured branch workflow.

## 21.1 Main Branch

`main`

Purpose:

- stable state
- release-ready code
- tagged releases

Do not perform normal feature development directly on `main`.

## 21.2 Develop Branch

`develop`

Purpose:

- integration branch
- receives completed and reviewed work
- candidate state for future releases

Do not perform normal feature implementation directly on `develop`.

## 21.3 Task Branches

Use branches such as:

```text
feature/*
fix/*
docs/*
refactor/*
chore/*
```

Normal feature flow:

```text
develop
   ↓
feature/<scope>
   ↓
implementation
   ↓
validation
   ↓
PR
   ↓
review
   ↓
develop
```

Release flow:

```text
develop
   ↓
full release validation
   ↓
PR develop → main
   ↓
main
   ↓
tag/release
```

---

# 22. Mandatory Pull Request Rule

A task branch must not be merged directly into `develop`.

A Pull Request is mandatory for:

- `feature/*`
- `fix/*`
- `docs/*`
- `refactor/*`
- other task branches

No direct merge should bypass review.

---

# 23. Mandatory Review-only Sub-agent

Before a PR from a task branch can merge into `develop`, create a dedicated **review-only Sub-agent**.

The reviewer must:

- inspect the branch diff
- inspect relevant code
- inspect tests
- inspect documentation
- check architectural consistency
- check security implications
- check cross-platform implications
- check for regressions
- verify acceptance criteria

The reviewer must **not**:

- edit files
- apply patches
- commit
- push
- merge
- rebase
- rewrite history

The reviewer returns findings only.

The **main agent** must evaluate and implement all fixes.

Workflow:

```text
Task branch
    ↓
PR
    ↓
Review-only Sub-agent
    ↓
Findings
    ↓
Main Agent evaluates
    ↓
Main Agent fixes
    ↓
Tests / checks rerun
    ↓
Re-review
    ↓
Merge only when acceptable
```

A reviewer reporting “no findings” is valid.

Do not fabricate review findings merely to produce output.

---

# 24. Release PR Rule

`develop → main` must also use a Pull Request.

Before promotion:

- all release-target features merged
- shared test suite passes
- macOS integration tests pass
- Windows integration tests pass
- security checks pass
- dependency checks pass
- documentation is current
- release notes prepared
- no known release-blocking issue remains

Do not use `main` as a development branch.

---

# 25. Phase 0 — Repository Inspection

Before changing files:

1. Inspect the repository.
2. Identify whether it is:
   - empty/new,
   - partially initialized,
   - already containing work.
3. Read existing:
   - `README.md`
   - contributing guidance
   - agent instructions
   - documentation
   - configuration
   - CI
   - package files
4. Preserve valid existing work.
5. Do not recreate or overwrite existing architecture blindly.
6. Identify inconsistencies with this prompt.

If the repository is empty, proceed with controlled bootstrap.

Document material deviations instead of silently changing direction.

---

# 26. Phase 1 — Agent Skills Preparation

Before real product implementation, prepare the agent environment.

Use the Project Skill:

> **`find-skills`**

Search for skills relevant to the project.

At minimum investigate categories such as:

- Rust
- Tauri
- TypeScript
- React
- software architecture
- code quality
- Rust testing
- frontend testing
- security
- dependency security
- UX/UI
- accessibility
- macOS development
- Windows development
- Git
- GitHub
- Pull Request review
- CI/CD
- GitHub Actions
- release engineering
- application signing
- performance profiling
- documentation

---

# 27. Skill Selection Policy

Do not install every discovered skill.

Evaluate candidates.

Preference order:

1. official skill
2. official ecosystem/community-maintained skill
3. popular and actively maintained skill
4. well-known trusted organization or author
5. other sources only with explicit justification

Avoid:

- unknown publishers without justification
- abandoned skills
- suspicious install scripts
- unnecessary permissions
- duplicated capabilities
- irrelevant skills
- global installation when project installation is available

All selected skills must be installed as **Project Skills**, not Global Skills.

---

# 28. Skill Documentation

Document selected project skills.

Recommended file:

```text
docs/agent/SKILLS.md
```

For each skill record:

```text
Name
Purpose
Source
Trust rationale
Version / commit if available
Why selected
Where used in this project
```

Also document rejected notable candidates when the rejection is useful for future maintainers.

Do not expose secrets or credentials.

---

# 29. Documentation-first Development

Thaa is agent-assisted.

Documentation is part of the system, not optional project decoration.

Do not allow Markdown files to accumulate randomly throughout the repository.

Root should remain clean.

Recommended root:

```text
/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── SECURITY.md
├── docs/
├── src/
├── src-tauri/
└── ...
```

---

# 30. Documentation Information Architecture

Use a structure similar to:

```text
docs/
├── README.md
│
├── requirements/
│   ├── PRODUCT-REQUIREMENTS.md
│   ├── FUNCTIONAL-REQUIREMENTS.md
│   ├── NON-FUNCTIONAL-REQUIREMENTS.md
│   └── PRODUCT-RULES.md
│
├── architecture/
│   ├── ARCHITECTURE.md
│   ├── DOMAIN-MODEL.md
│   ├── PLATFORM-ADAPTERS.md
│   ├── DATA-FLOW.md
│   └── DIRECTORY-STRUCTURE.md
│
├── development/
│   ├── DEVELOPMENT.md
│   ├── GIT-WORKFLOW.md
│   ├── CODING-STANDARDS.md
│   └── LOCAL-SETUP.md
│
├── agent/
│   ├── AGENT-WORKFLOW.md
│   ├── SKILLS.md
│   └── REVIEW-GUIDELINES.md
│
├── security/
│   ├── SECURITY.md
│   └── THREAT-MODEL.md
│
├── testing/
│   ├── TEST-STRATEGY.md
│   ├── TEST-CASES.md
│   └── PLATFORM-TESTING.md
│
├── adr/
│   ├── ADR-001-...
│   ├── ADR-002-...
│   └── ...
│
├── features/
│   └── <feature-name>/
│       ├── REQUIREMENTS.md
│       ├── DESIGN.md
│       └── TESTING.md
│
├── work/
│   ├── W001-...
│   ├── W002-...
│   └── ...
│
└── releases/
    └── ...
```

Do not create empty files purely to satisfy the structure.

Create documentation when it becomes relevant.

---

# 31. Documentation Classification Rule

Before creating a Markdown file, classify it.

```text
Product requirement
→ docs/requirements/

Architecture
→ docs/architecture/

Development process
→ docs/development/

Agent instructions
→ docs/agent/

Security
→ docs/security/

Testing
→ docs/testing/

Technical decision
→ docs/adr/

Feature-specific documentation
→ docs/features/

Implementation work item
→ docs/work/

Release
→ docs/releases/
```

Avoid files such as:

```text
notes2.md
final-plan.md
plan-new.md
temp.md
testing-final-final.md
```

unless there is a clear governed purpose.

---

# 32. Architecture Decision Records

Use ADRs for important decisions.

Likely initial ADRs include:

- Tauri 2 as desktop framework
- Rust as system/core language
- React + TypeScript as frontend
- platform adapter architecture
- port provider abstraction
- branch and PR workflow
- process provider strategy
- local-first / no telemetry default

ADR template:

```markdown
# ADR-XXX: Decision Title

## Status

Proposed | Accepted | Superseded | Deprecated

## Context

## Decision

## Alternatives Considered

## Consequences
```

Do not create an ADR for trivial implementation details.

---

# 33. Work Item Documentation

Development should be incremental.

Use work items such as:

```text
W001 Project Bootstrap
W002 Domain Foundation
W003 macOS Port Provider
W004 Windows Port Provider
W005 Process Inspection
...
```

Recommended work item content:

```markdown
# WXXX — Title

## Status

## Objective

## Scope

## Out of Scope

## Requirements

## Dependencies

## Design Notes

## Acceptance Criteria

## Validation

## Security Considerations

## Platform Considerations

## Documentation Impact

## Review Findings

## Known Limitations
```

The work item document should be updated during implementation.

---

# 34. Requirement Traceability

Maintain traceability.

At minimum, a feature should be traceable through:

```text
Requirement
    ↓
Feature Design
    ↓
Work Item
    ↓
Implementation
    ↓
Tests
    ↓
Review
    ↓
Status
```

Do not mark a requirement implemented if the associated verification is absent.

Use identifiers when beneficial, for example:

```text
FR-001
NFR-001
W003
TC-PORT-001
ADR-004
```

Avoid excessive bureaucracy for trivial details.

---

# 35. Requirement Freeze Before Implementation

Before writing significant product code:

1. create/review product requirements
2. create functional requirements
3. create non-functional requirements
4. identify unresolved questions
5. resolve material ambiguities
6. define MVP boundaries
7. define acceptance criteria
8. mark the initial requirement baseline as frozen

After freeze:

- new material requirements should be documented as changes
- avoid silently expanding scope during implementation

Do not interpret requirement freeze as “requirements can never change.”

It means changes should be deliberate and traceable.

---

# 36. Non-functional Requirements

Define measurable or verifiable expectations for:

## Performance

- idle resource usage should remain appropriate for a tray utility
- avoid excessive polling
- avoid unnecessary subprocess spawning
- avoid UI blocking
- avoid unbounded memory growth

## Reliability

- process may disappear between scan and action
- port ownership may change
- metadata may become stale
- provider failure must not crash the app

## Security

- no shell injection
- do not blindly interpolate command input
- validate process actions
- least privilege
- no silent privilege escalation
- no hidden telemetry

## Privacy

- process and filesystem data stays local by default

## Accessibility

- keyboard navigability
- accessible labels
- reasonable contrast
- platform-native interaction conventions where practical

## Maintainability

- testable interfaces
- isolated adapters
- documented architecture
- limited coupling between Rust and frontend

## Cross-platform consistency

- equivalent concepts across macOS and Windows
- platform-specific differences are documented

---

# 37. Security Threat Model

Create a threat model before process-control features are considered complete.

At minimum consider:

- malicious process names
- malicious command lines
- paths containing unusual characters
- shell injection
- PID reuse
- race conditions between scan and kill
- killing the wrong process
- elevated/protected processes
- privilege escalation attempts
- opening untrusted URLs
- opening unexpected filesystem paths
- tampered configuration
- dependency supply-chain risk
- unsafe native API/FFI usage
- secret leakage into logs
- exposing sensitive process command arguments in UI/screenshots

Do not log full sensitive command lines unnecessarily.

---

# 38. Safe Process Action Rules

Before stopping a process:

1. verify PID still exists
2. where practical revalidate expected identity
3. do not assume stale scan data is current
4. clearly distinguish graceful and force stop
5. handle permission failures
6. report result accurately

Avoid “kill all matching process names” as a default action.

Avoid targeting critical OS processes.

If safeguards are added for protected/system processes, document the rules.

---

# 39. Cross-platform Contract Testing

Platform adapters should share behavioral contracts.

Example PortProvider contract:

```text
Given:
a test TCP listener on a known random test port

Expect:
provider returns listener
port matches
protocol matches
ownership is resolved where supported
```

Run equivalent tests for:

```text
MacOSPortProvider
WindowsPortProvider
```

Example ProcessProvider contract:

```text
Given:
controlled test process

Expect:
PID
process name
supported metadata
```

Document platform differences instead of forcing false parity.

---

# 40. Testing Strategy

Use multiple levels.

## Unit Tests

Test:

- parsing
- domain logic
- project detection
- runtime detection
- binding/exposure classification
- mapping and normalization
- error handling

## Integration Tests

Test:

- provider + OS integration
- controlled listener discovery
- controlled process discovery
- process stop behavior
- project context resolution

## Frontend Tests

Test important:

- states
- filters
- error handling
- actions
- data presentation

Avoid over-investing in fragile snapshot tests.

## Platform-specific Tests

macOS and Windows must each have real integration coverage.

## Regression Tests

Every bug fix should include a regression test when practical.

---

# 41. Quality Gates

Before a work item is review-ready, run applicable checks.

At minimum:

- build
- unit tests
- integration tests
- platform-specific tests
- formatter
- linter
- Rust static checks
- TypeScript typecheck
- frontend lint
- dependency audit
- security checks
- documentation validation
- functional regression checks

Use appropriate tools selected during project setup.

Do not claim a check passed unless it actually ran successfully.

If a check cannot run:

- document why
- classify whether it blocks merge
- record the risk

---

# 42. Code Quality Expectations

Prefer:

- small cohesive modules
- descriptive naming
- explicit error types
- structured logging
- testable interfaces
- minimal unsafe Rust
- documented unsafe blocks when unavoidable
- clear frontend state boundaries
- no silent catch-and-ignore patterns
- no broad platform conditionals in shared business logic

Avoid:

- god objects
- giant Tauri command handlers
- duplicated platform logic
- unnecessary wrappers
- premature generic abstractions
- speculative architecture for features not planned

---

# 43. Error Handling

Errors should distinguish categories such as:

- unsupported capability
- permission denied
- process disappeared
- provider failure
- parse failure
- OS API failure
- action rejected
- invalid input

Frontend errors should be understandable without exposing internal stack traces.

Diagnostics can contain more detail, but avoid secrets.

---

# 44. Logging

Use structured and privacy-aware logging.

Do not log by default:

- full command arguments containing possible secrets
- authentication tokens
- environment variables
- private file content

Logging levels should be deliberate.

Debug logs may expose more system information but must still avoid obvious secrets.

---

# 45. UX/UI Phase

Before building major UI:

1. define main user flows
2. define information hierarchy
3. define tray experience
4. define main-window experience
5. define empty/loading/error/permission states
6. define destructive action confirmation
7. define cross-platform differences

Do not over-design branding before core workflows are validated.

Thaa should feel like a focused developer utility, not a generic admin dashboard.

---

# 46. Main User Flows

At minimum design:

## Flow A — Find a port

```text
Open Thaa
→ Search 3000
→ See process/project/runtime
→ Open or inspect
```

## Flow B — Stop stale dev server

```text
Find runtime
→ inspect identity
→ Stop
→ verify process ended
→ UI refreshes
```

## Flow C — Port conflict

```text
Developer knows desired port
→ search
→ identify current owner
→ inspect project
→ stop existing runtime if appropriate
```

## Flow D — Network exposure

```text
View runtime
→ see binding
→ classify local-only or network-accessible
→ show factual warning
```

## Flow E — Project context

```text
View listener
→ process
→ working directory
→ project root
→ Git branch
```

---

# 47. Repository Bootstrap

After requirements and architecture are ready, bootstrap the project.

Expected areas may include:

```text
src/                 # React frontend
src-tauri/           # Tauri/Rust
docs/
.github/
```

Potential Rust structure:

```text
src-tauri/src/
├── domain/
├── application/
├── infrastructure/
│   ├── network/
│   ├── process/
│   ├── project/
│   └── runtime/
├── platform/
│   ├── macos/
│   └── windows/
├── commands/
└── lib.rs / main.rs
```

Do not create directories merely because this prompt mentions them.

Validate the best Tauri 2 structure first.

---

# 48. Frontend Organization

Prefer feature-oriented frontend organization where practical.

Example:

```text
src/
├── app/
├── components/
├── features/
│   ├── runtimes/
│   ├── ports/
│   ├── processes/
│   └── settings/
├── hooks/
├── lib/
└── types/
```

Avoid central “utils” dumping grounds.

Keep domain types synchronized carefully with the Rust boundary.

---

# 49. Tauri Boundary Rules

Tauri commands should be thin.

Avoid putting business logic directly inside command handlers.

Prefer:

```text
Tauri command
    ↓
Application use case
    ↓
Domain interface
    ↓
Infrastructure adapter
```

Validate all frontend inputs at the Rust boundary.

Do not assume frontend input is trusted.

---

# 50. Work Item Execution Workflow

For each implementation work item:

1. confirm relevant requirement
2. confirm dependency work is complete
3. update/create work doc
4. create task branch from `develop`
5. implement smallest coherent slice
6. add/update tests
7. run quality gates
8. update documentation
9. inspect diff
10. create PR to `develop`
11. invoke review-only Sub-agent
12. collect findings
13. main agent resolves valid findings
14. rerun checks
15. re-review if changes are material
16. merge only when acceptable
17. update work status

Do not batch unrelated work into one branch.

---

# 51. Suggested Initial Work Breakdown

Do not assume these exact IDs if repository work already exists.

Potential sequence:

```text
W001 — Repository and Agent Bootstrap
W002 — Documentation Foundation
W003 — Requirements Baseline
W004 — Architecture Foundation
W005 — Tauri Application Bootstrap
W006 — Shared Domain Models
W007 — PortProvider Contract
W008 — macOS Listening Port Provider
W009 — Windows Listening Port Provider
W010 — ProcessProvider Contract
W011 — Shared Process Inspection
W012 — Process Stop / Force Stop
W013 — Tray Runtime List
W014 — Search and Filter
W015 — Project Root Detection
W016 — Git Context
W017 — CPU / Memory / Uptime
W018 — Process Tree
W019 — Network Exposure
W020 — Runtime Detection
```

This is a planning aid, not permission to implement everything immediately.

---

# 52. Recommended MVP Release Scope

A realistic first meaningful release should prioritize:

- macOS + Windows builds
- listening TCP port discovery
- process mapping
- process name
- PID
- executable/command where available
- working directory where available
- search/filter
- refresh
- open local URL
- copy URL/port/PID
- stop
- force stop
- tray access
- clear permission/error handling

Project/Git context can be included in MVP only if it does not destabilize platform parity.

Do not delay all user value until every planned intelligence feature exists.

---

# 53. Cross-platform Delivery Strategy

Architecture must support both platforms from the start.

Implementation may proceed incrementally.

Recommended strategy:

1. shared architecture/domain first
2. first functional provider on one platform
3. equivalent provider on second platform
4. parity validation
5. shared UI
6. shared feature increments

Do not allow one platform to accumulate large feature debt without documentation.

---

# 54. CI/CD

Use GitHub Actions.

At minimum, CI should include:

## Shared

- formatting
- lint
- typecheck
- Rust tests
- frontend tests
- dependency checks

## macOS Runner

- macOS integration tests
- app build

## Windows Runner

- Windows integration tests
- app build

Do not claim cross-platform support based solely on compilation.

Where possible, execute platform behavior tests on real hosted runners.

---

# 55. Release Artifacts

## macOS

Target release workflow may include:

```text
Build
→ Developer ID signing
→ Hardened Runtime
→ Notarization
→ Stapling
→ DMG
```

Signing credentials must not be committed.

## Windows

Target release workflow may include:

```text
Build
→ Code signing
→ MSI and/or NSIS installer
→ Release artifact
```

The final installer strategy should be documented in an ADR if materially changed.

---

# 56. Open-source Requirements

Before public release, establish:

- appropriate open-source license
- CONTRIBUTING guide
- SECURITY policy
- issue templates
- PR template
- Code of Conduct if desired
- release notes process

Do not choose a license arbitrarily without documenting the choice.

Avoid including:

- personal credentials
- internal endpoints
- private organization details
- local secrets
- test data derived from private systems

Examples in public documentation must use neutral placeholders.

---

# 57. Dependency Policy

Before adding a dependency:

1. confirm need
2. prefer maintained and trusted projects
3. inspect license compatibility
4. inspect security history where practical
5. avoid duplicate dependencies
6. avoid heavy frameworks for trivial tasks
7. document material dependencies

Lockfiles should be committed where appropriate.

Run dependency security checks in CI.

---

# 58. Performance Considerations

Thaa may remain active all day.

Therefore:

- avoid aggressive polling
- prefer efficient refresh strategies
- ensure background scans are cancelable
- do not block UI thread
- avoid repeatedly spawning expensive processes without reason
- measure idle CPU and memory
- watch for handle/process leaks
- test long-running refresh behavior

Performance claims require measurement.

---

# 59. Concurrency

Rust async/concurrency design must be intentional.

Avoid:

- unbounded background tasks
- duplicated scanners
- race-prone shared mutable state
- overlapping scans that produce stale UI

Define a clear refresh coordinator or equivalent.

Cancellation behavior should be supported where appropriate.

---

# 60. Race Conditions to Handle

System state changes constantly.

Examples:

- process exits after scan
- PID gets reused
- port closes before action
- project folder disappears
- repository branch changes
- listener ownership changes
- process becomes inaccessible

Application logic must expect these events.

Do not treat them as exceptional bugs by default.

---

# 61. Permissions

Document permissions required per platform.

Avoid requiring administrator privileges for ordinary inspection if it can be avoided.

If some metadata requires elevated permission:

- explain the limitation
- do not automatically elevate
- degrade gracefully

Permission-denied behavior must be tested.

---

# 62. Security Review Gate

Before a feature involving:

- process termination
- filesystem opening
- shell commands
- native FFI
- installer privileges
- auto-start
- URL opening

is considered complete, review its security implications.

Security review findings should be documented in:

```text
docs/security/
```

or the relevant work document.

---

# 63. Documentation Update Rule

A feature is not complete when code is complete.

Relevant documentation must be updated in the same work cycle.

Potentially affected documents include:

- requirements
- architecture
- feature docs
- testing docs
- security docs
- ADRs
- work status
- README

Avoid stale documentation.

---

# 64. Definition of Done

A work item is **Done** only when all applicable conditions are satisfied.

## Implementation

- acceptance criteria implemented
- no known blocking functional defect

## Testing

- unit tests pass
- integration tests pass
- applicable platform tests pass
- regression coverage added when appropriate

## Quality

- formatting passes
- lint passes
- type checks pass
- static checks pass

## Security

- security impact considered
- relevant scan/check passes
- no unresolved critical/high issue

## Documentation

- work item updated
- architecture updated if affected
- testing documentation updated
- requirements/status updated if affected

## Review

- PR exists
- review-only Sub-agent completed review
- valid findings resolved
- re-review completed when needed

## Integration

- merged into `develop`

A successful local build alone is **not** Definition of Done.

---

# 65. Review Severity Guidance

Review findings may be categorized:

```text
Critical
High
Medium
Low
Suggestion
```

Examples:

## Critical

- unsafe destructive action
- severe security issue
- data corruption
- release-blocking architecture defect

## High

- important functional failure
- cross-platform break
- serious race condition
- privilege/security problem

## Medium

- maintainability defect
- missing important test
- inconsistent platform behavior

## Low

- small quality issue
- documentation gap

Do not inflate severity.

---

# 66. Agent Communication

During substantial work:

- provide concise progress updates
- surface discovered blockers early
- report partial findings when useful
- do not hide failed tests
- do not claim work is complete when validation is missing

When uncertainty can be resolved by repository inspection, tools, documentation, or tests, resolve it yourself rather than asking unnecessary questions.

If a requirement is materially ambiguous and cannot be safely inferred, document the ambiguity and choose the least scope-expanding interpretation.

---

# 67. Agent Prohibitions

Unless explicitly requested, do **not**:

- work directly on `main`
- implement normal features directly on `develop`
- bypass Pull Requests
- bypass review Sub-agent
- let reviewer modify code
- merge with unresolved blocking findings
- install required project skills globally
- install untrusted skills casually
- scatter Markdown files in the repository
- implement all roadmap features at once
- add cloud services
- add analytics
- add AI APIs
- introduce Linux scope
- commit credentials
- expose private environment data
- hide failing tests
- rewrite Git history without explicit necessity
- force-push shared release branches
- change major architecture without ADR
- execute arbitrary code from detected projects
- run arbitrary shell strings assembled from untrusted input

---

# 68. First Execution Instructions

When first receiving this prompt in the repository, do **not** immediately build Port Inspection.

Perform the following sequence:

```text
1. Inspect repository
2. Read existing project instructions
3. Determine current branch and repository state
4. Confirm branch model
5. Run/use find-skills
6. Evaluate trusted project skills
7. Install required skills at project scope
8. Document selected skills
9. Establish docs/ information architecture
10. Establish requirements baseline
11. Establish architecture baseline
12. Create initial ADRs
13. Establish testing/security strategy
14. Establish CI baseline
15. Break MVP into work items
16. Select the smallest valid first implementation work item
17. Create branch from develop
18. Implement incrementally
19. Validate
20. PR → review-only Sub-agent → fixes → re-review → develop
```

If the repository does not yet contain `main` and `develop`, create the branch foundation deliberately and document it.

Do not create dozens of speculative branches in advance.

---

# 69. Initial Documentation Deliverables

Before substantial feature implementation, the repository should have enough documentation to answer:

- What is Thaa?
- Who is it for?
- What is in MVP?
- What is explicitly out of scope?
- What are the functional requirements?
- What are the non-functional requirements?
- Why Tauri?
- Why Rust?
- Why React + TypeScript?
- How are macOS and Windows separated?
- How does branch workflow work?
- How do AI Agents work in the repo?
- What project skills are installed?
- How are PRs reviewed?
- How is security handled?
- How is testing handled?
- What is the current work item?
- What is the Definition of Done?

Do not over-document details that do not yet exist.

---

# 70. Architecture Validation Questions

Before implementation, explicitly validate:

1. Can domain models compile without OS-specific dependencies?
2. Can macOS and Windows providers satisfy the same interfaces?
3. Can provider implementations be swapped in tests?
4. Are Tauri commands thin?
5. Can process actions revalidate stale PIDs?
6. Can unavailable metadata be represented without fake values?
7. Is network exposure classification separated from raw network collection?
8. Can project/runtime detection be tested independently of live OS state?
9. Is there a clear place for platform capabilities?
10. Does the structure avoid platform conditionals leaking across layers?

If not, fix architecture before adding large feature volume.

---

# 71. Product Success Criteria for Early Releases

Early releases should prioritize whether users can reliably answer:

```text
What is listening?
Which process owns it?
Which project does it belong to?
Is it local-only or network reachable?
Can I safely inspect or stop it?
```

Do not optimize for feature count.

---

# 72. Naming Guidance

Use clear product terminology.

Preferred examples:

```text
Runtime
Listener
Port
Process
Project
Network Exposure
Local Only
Network Accessible
Stop Process
Force Stop
```

Avoid technical jargon in primary UI when a clearer term exists.

Technical details can appear in advanced sections.

---

# 73. Project Detection Rules

Project detection must be deterministic and testable.

Possible evidence:

- `.git`
- language/package manifests
- build files
- workspace files

Prefer nearest plausible project root from the process working directory.

Document precedence rules.

Avoid scanning the entire filesystem.

---

# 74. Runtime Detection Rules

Runtime detection may consider:

- executable
- command line
- project files
- known manifest files
- known dependency indicators

Do not classify solely from a port number.

Example:

Port `5432` does not automatically prove PostgreSQL.

Detection should support confidence/evidence internally if needed.

Do not expose false certainty.

---

# 75. URL Opening Rules

When providing an “Open” action:

- only construct expected local URLs safely
- validate protocol
- validate host/address
- validate port range
- avoid opening arbitrary command-line strings as URLs
- consider IPv6 URL formatting

Do not execute shell commands to open URLs if a safer platform API exists.

---

# 76. Process Identity Revalidation

Before destructive process actions, consider storing enough observed identity to reduce PID reuse risk.

Possible signals:

- PID
- executable
- process start time
- process name

Exact feasibility differs by platform.

Document the chosen safety strategy.

---

# 77. Default Refresh Strategy

Do not hard-code an overly aggressive refresh frequency.

Requirements:

- manual refresh available
- automatic refresh reasonable
- no overlapping scans
- configurable later if useful
- inactive window/tray behavior should avoid waste where possible

Measure before optimizing.

---

# 78. Public Documentation Hygiene

Because Thaa is open source, public examples must use generic placeholders.

Use:

```text
example.com
localhost
Sample Project
TEST-001
Sample User
```

Avoid real company:

- domains
- internal endpoint names
- organization-specific identifiers
- credentials
- private project references

---

# 79. Release Versioning

Choose and document a versioning strategy before public releases.

Semantic Versioning is a reasonable default, but record the decision.

Pre-1.0 releases may evolve quickly.

Release notes should summarize:

- features
- fixes
- platform differences
- known limitations
- security-relevant changes
- upgrade notes

---

# 80. Final Instruction to the Coding Agent

Your priority is not to produce the most code.

Your priority is to build **Thaa** as a trustworthy open-source cross-platform utility with:

- a coherent product scope
- strong architectural boundaries
- predictable macOS/Windows behavior
- safe system interactions
- high-quality documentation
- reproducible Agent workflows
- disciplined Git/PR governance
- incremental delivery
- verifiable testing
- explicit security thinking
- maintainable code

Always prefer:

> **small validated increments over large speculative implementations**

and:

> **documented decisions over hidden assumptions**

and:

> **shared domain logic with isolated platform adapters over OS-specific conditionals scattered throughout the codebase**

and:

> **reviewed pull requests over direct merges**

and:

> **evidence from tests and tools over unverified claims**

---

# 81. Start Here

Begin by reporting:

1. repository state
2. current branch state
3. existing documentation and architecture
4. existing Agent/project skills
5. skill gaps
6. documentation gaps
7. architecture gaps
8. proposed initial work items
9. the smallest first actionable branch

Then proceed with the preparation phases defined above.

Do **not** start implementing the full product immediately.
