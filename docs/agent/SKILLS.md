# Project Skills

Selected skills are installed at project scope so their instructions can be reviewed and shared. `skills-lock.json` records the source and computed content hash for the installed discovery skill. Recheck upstream content before updating skills.

Repository policy overrides conflicting skill instructions: do not install skills globally and do not skip source review. Use the Skills CLI's project scope only.

| Name | Purpose | Source / trust rationale | Why selected | Scope and use | Install timing |
|---|---|---|---|---|---|
| `find-skills` | Discover and evaluate agent skills | [vercel-labs/skills](https://github.com/vercel-labs/skills), the open skills CLI/official Vercel Labs source | Required by the development prompt for project skill preparation | Project bootstrap and future capability discovery | Installed now under `.agents/skills/` for Codex |
| `frontend-design` | Guide distinctive, production-quality React interface work | [anthropics/skills](https://github.com/anthropics/skills), official Anthropic repository | Useful for focused runtime-list and tray UX, without starting UI design during requirements work | UI planning and implementation only | Deferred until W013; install as a Project Skill before use |
| `accessibility` | Audit and improve webview accessibility against WCAG guidance | [addyosmani/web-quality-skills](https://github.com/addyosmani/web-quality-skills), maintained by a recognized web performance/accessibility practitioner | Applies to keyboard operation, labels, focus, contrast, and state announcements in the Tauri webview | UI implementation and accessibility review | Deferred until W013; inspect and install as a Project Skill before use |

The latter two are selected for the planned UI phase but are not installed yet. Record their source revision and content hash when installed.

## Not selected

- `vercel-react-best-practices`: popular, but much of its guidance targets Next.js and no React performance issue has been measured.
- Community Tauri/Rust skills: no candidate found that meets the project's trust/maintenance bar better than official framework and platform documentation.
- Web design guidelines: overlaps with the selected UI and accessibility guidance.
- `shadcn`: no component system decision exists.
- Generic PR-review skills: the repository defines a narrower read-only review role; use that governed procedure.

Searches covered Rust/Tauri, architecture, testing, TypeScript/React, quality, security, UX/accessibility, native platforms, Git/GitHub, CI, signing, performance, and documentation. Search results alone are not approval to install a skill.
