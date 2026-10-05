# Project Skills

Project Skills are instructions for the Agent, not dependencies or authority
over Thaa's accepted requirements, ADRs, architecture, security policy, testing
strategy, or read-only review workflow. Each selected skill is stored under
`.agents/skills/` and installed at **Project** scope. `skills-lock.json` records
source paths and hashes for reproducibility; it does not replace reviewing
upstream changes before an update. See [third-party notices](THIRD-PARTY-NOTICES.md).

## Selected skills

| Name                          | Category and purpose                                               | Source, publisher, revision                                                                                                                              | Trust and selection rationale                                                                                                                                                                | Thaa use                                                                                                        | Scope   |
| ----------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------- |
| `find-skills`                 | Skill discovery and evaluation                                     | [vercel-labs/skills](https://github.com/vercel-labs/skills), Vercel Labs; revision `3694740352eeef5cdd689af694c485f1ff62eec3`; MIT                       | Maintained open-source Skills CLI and project discovery workflow; required by project instructions                                                                                           | Discovery only; inspect candidate content and trust before installation                                         | Project |
| `rust-patterns`               | Rust ownership, errors, concurrency, and module practices          | [affaan-m/ECC](https://github.com/affaan-m/ECC), ECC community; revision `ef648e01899ba3e8dc6371642deaaf64b4477775`; MIT                                 | Reviewed skill text and repository license; practical Rust-specific guidance. Treat crate suggestions as examples, not dependency approval; project error/architecture rules take precedence | Rust implementation and review                                                                                  | Project |
| `vercel-react-best-practices` | React behavior, rendering, and bundle performance                  | [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills), Vercel; revision `063bee94c3f4df8453406c830b0a7df0f2860278`; skill declares MIT | Published by a trusted React ecosystem maintainer and reviewed. Use client React guidance; Next.js, server-rendering, and RSC rules do not apply to the Vite Tauri client                    | React component implementation and performance review when relevant                                             | Project |
| `frontend-design`             | Product-specific visual design and UI critique                     | [anthropics/skills](https://github.com/anthropics/skills), Anthropic; revision `8a1541c4a3ffa5a20a5a91de0dcf3f0bab1d1ef4`; Apache-2.0 license included   | Official publisher; instructions reviewed. Its visual direction is subordinate to the focused developer-utility scope and approved UX requirements                                           | UI design work, not backend or architecture decisions                                                           | Project |
| `accessibility`               | WCAG-oriented webview accessibility audit and practices            | [addyosmani/web-quality-skills](https://github.com/addyosmani/web-quality-skills), Addy Osmani; revision `afa8da942115f2961fdbfa80807ea0b232ff6c00`; MIT | Recognized web-quality maintainer; reviewed skill and supporting references. Accessibility guidance supplements, not replaces, platform testing                                              | React/Tauri webview accessibility implementation and review                                                     | Project |
| `security-and-hardening`      | Input, injection, privacy, and dependency-security review patterns | [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills), Addy Osmani; revision `9d0c60d406b454a78ccc0a175b19932047aa4dac`; MIT             | Reviewed skill and referenced patterns from a recognized maintainer. It is web-oriented; W004 and native/platform guidance remain authoritative                                              | Use relevant input, injection, privacy, and dependency checks; do not apply web-server assumptions mechanically | Project |
| `typescript-advanced-types`   | TypeScript type-system guidance for complex boundary types         | [wshobson/agents](https://github.com/wshobson/agents), Seth Hobson; revision `156b7a5e7a8b93642628a339ee4039c925b34c7f`; MIT                             | Reviewed skill and source repository license. Useful when designing transport/domain types, but does not justify advanced types absent a real need                                           | TypeScript boundary/type design when complexity warrants it                                                     | Project |

Revisions are upstream source commits inspected during W005.1. The exact
installed-file hashes are in `skills-lock.json`. Skills are copied into the
repository for review; no installer scripts are part of the installed set.

## Invocation guidance

- Rust changes: consult `rust-patterns` alongside the coding standards, W003
  architecture, and W004 security/testing rules.
- React/TypeScript changes: consult the Vercel React skill for relevant client
  behavior and `typescript-advanced-types` only when a real type-design problem
  warrants it.
- UI work: use `frontend-design` for design decisions and `accessibility` for
  keyboard, semantics, state announcements, and WCAG-oriented review.
- Security-sensitive changes: add `security-and-hardening` to the relevant
  implementation guidance; W004 remains the project's security authority.
- Documentation and PR review: use the repository's own document and
  [review guidelines](REVIEW-GUIDELINES.md). A review-only Sub-agent may report
  findings but must never edit or mutate the branch.
- Tauri-specific changes: follow [official Tauri documentation](https://v2.tauri.app/)
  and accepted ADRs. No discovered Tauri skill met the project trust bar.

Skills should be applied when relevant to the work item, not invoked as a
checklist on every change. Do not install them globally. Review upstream source,
revision, permissions, and changed instructions before updating any skill.

## Rejected or deferred candidates

- Community Tauri skill packs were reviewed but rejected: the inspected
  candidates had limited maintenance signals or questionable guidance about
  IPC/isolation. Official Tauri documentation is preferred.
- `affaan-m/ECC` Rust testing guidance was rejected because it prescribes a
  fixed coverage threshold contrary to W004's behavior/evidence-based policy.
- Generic TDD/domain-modeling skills were not selected because their workflow
  assumptions add process or documentation requirements that conflict with
  Thaa's established requirements and review process.
- Broad web design/review guidance and UI component-library skills were not
  selected where they duplicate the focused selected skills or imply a UI
  framework decision not made by the project.
- Separate macOS, Windows, CI, release-signing, and performance skills are
  deferred until a work item needs those specialized procedures. Use official
  platform/tool documentation in the meantime.

Discovery was performed with the project-scoped `find-skills` instructions
and Skills CLI. Search ranking alone was not treated as evidence of quality.
