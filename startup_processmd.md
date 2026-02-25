# Nexus Dashboard Suite (Vue 3)

This project is a Vue 3 + TypeScript rebuild of the original Nexus dashboard UI. The UI and behavior are kept consistent while the codebase is refactored into Vue components with Composition API.

## Run Locally

**Prerequisites:** Node.js, pnpm

1. Install dependencies:
   `pnpm install`
2. (Optional) Set `GEMINI_API_KEY` in `.env.local`
3. Start the dev server:
   `pnpm dev`

## Tests

Run unit tests:
`pnpm test`

## Lint & Format

- `pnpm lint`
- `pnpm format:check`

## Project Structure

- `src/app/App.vue`: App shell + navigation state
- `src/features`: Feature modules (store, marketplace, settings, workspace, chat)
- `src/shared`: Reusable components and composables
- `src/i18n`: Locale files and i18n setup
- `src/styles/global.css`: Scrollbar and layout utilities

See `docs/ENGINEERING_GUIDE.md` for conventions and structure details.

Legacy React code is moved to `C:\project\user\nexus-dashboard-suite-legacy`.
