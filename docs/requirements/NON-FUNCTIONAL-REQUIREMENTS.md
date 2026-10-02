# Non-Functional Requirements

**Baseline:** THAA-REQ-0.1 · **Status:** Frozen for initial implementation · **Date:** 2026-10-02

These requirements define observable constraints. They do not invent numeric performance budgets absent from the prompt; P0 release work must collect measurements and record justified budgets before making performance claims.

| ID | Requirement and acceptance evidence | Source | Planned verification |
|---|---|---|---|
| NFR-001 | **Privacy/local-first:** no account, cloud service, telemetry, AI service, or remote API is required for normal operation; process/filesystem data stays local by default. Dependency/network review and source inspection confirm no hidden reporting path. | Prompt §4, §19, §36, §44 | W004 threat review; dependency/config review in W006+ |
| NFR-002 | **Least privilege:** ordinary inspection does not require administrator/root elevation. Permission-denied data/actions degrade clearly; the app never silently elevates. | §11, §36, §61 | Native permission tests in W008–W012 |
| NFR-003 | **Safe input/actions:** observed names, arguments, paths, and frontend inputs are untrusted; no shell injection, arbitrary command interpolation, arbitrary project execution, or arbitrary URL opening. Stop actions revalidate identity and report races/failures. | §36–38, §49, §62, §75–76 | Security review and adversarial fixtures in W008, W012–W013 |
| NFR-004 | **Sensitive data:** do not log full command arguments, environment variables, tokens, or private file contents by default. Detail presentation and screenshots account for sensitive process data. | §10, §37, §44 | Logging review and UI review/tests in W010/W013 |
| NFR-005 | **Reliability under changing state:** process, port, path, repository, or permissions may change during inspection. Provider errors and disappearing processes do not crash the app; stale state cannot trigger action against a different process. | §36, §43, §59–60 | Unit, contract, race tests in W007–W014 |
| NFR-006 | **Refresh/performance:** scans do not overlap, can be cancelled where practical, stay off the UI thread, avoid unbounded work and unnecessary subprocess launches, and use a conservative cadence. Record idle CPU/memory, scan duration, and longer-run stability on both target OSes before release claims. | §36, §58–59, §77 | Repeatable local/native-runner measurements before P0 release |
| NFR-007 | **Accessibility:** core flows are keyboard navigable; controls have accessible names/labels; focus and errors are perceivable; contrast is reasonable; platform conventions are respected where practical. | §36, §45 | Automated checks plus manual keyboard/webview checks in W013–W014 |
| NFR-008 | **Maintainability:** domain/application code compiles without OS-specific dependencies; platform conditionals remain in adapters; provider contracts are substitutable in tests; Tauri commands remain thin. | §7–8, §42, §49, §70 | Architecture checks and contract tests from W007 onward |
| NFR-009 | **Cross-platform support:** macOS and Windows each run native provider/action integration tests; shared concepts use shared contracts and differences are documented. Compilation alone is insufficient validation. | §5, §39–40, §53–54 | Native CI and P0 checklist in W008–W014 |
| NFR-010 | **Dependency/supply chain:** dependencies have a documented need, maintained/trusted source, compatible license, committed lockfile where applicable, and advisory checks. CI actions use minimal permissions and immutable references. | §27, §41, §57 | W005–W006 dependency review and recurring checks |

## Performance evidence rule

Before claiming acceptable idle usage or refresh performance, run a repeatable scenario on documented macOS and Windows environments. Record OS/tool versions, listener/process count, idle duration, refresh cadence, scan duration, CPU/memory observations, and limitations. Set numerical budgets only after measurement and retain evidence with the relevant work/release record.
