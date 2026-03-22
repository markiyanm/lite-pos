# Lite POS

Cross-platform Point of Sale desktop application for retail merchants.

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri v2 |
| Frontend | SvelteKit (static adapter, SPA) + Svelte 5 runes |
| UI components | shadcn-svelte + Tailwind CSS v4 |
| Database | SQLite via `tauri-plugin-sql` |
| Auto-updater | `tauri-plugin-updater` + GitHub Releases |
| Icons | Lucide Svelte |

## MCP Servers

- **SvelteKit MCP** (`sveltekit`): Connected via `https://mcp.svelte.dev/mcp`
- **shadcn-svelte**: Fetch `https://www.shadcn-svelte.com/llms.txt` for component docs

## Architecture

### Routes

```
src/routes/
├── +layout.ts              # ssr = false, prerender = false
├── +layout.svelte          # App shell: sidebar + auth guard + toasts + updater
├── login/+page.svelte      # PIN login (numpad, keyboard support)
├── pos/+page.svelte        # Main POS screen (product grid + order panel + payment)
├── products/
│   ├── +page.svelte        # Product list with table, search, category filter
│   └── [id]/+page.svelte   # Product create/edit form
├── categories/+page.svelte # Category list with inline CRUD + color picker
├── customers/
│   ├── +page.svelte        # Customer list with table, search
│   └── [id]/+page.svelte   # Customer form + order history
├── orders/
│   ├── +page.svelte        # Orders list with filters, stats cards
│   └── [id]/+page.svelte   # Order detail + refund/void + customer assign
├── reports/+page.svelte    # Reports: sales over time, product metrics, inventory
└── settings/+page.svelte   # Tabbed settings (8 sections incl. Appearance)
```

### Stores (`src/lib/stores/`)

All use Svelte 5 runes in `.svelte.ts` files with function-based pattern and getter accessors.

- **session.svelte.ts** — `currentUser`, `isAuthenticated`, `isAdmin`, `login()`, `logout()`
- **order.svelte.ts** — Cart: items, customer, draftId, notes. Derived: subtotalCents, taxTotalCents, totalCents, itemCount
- **settings.svelte.ts** — Map-based: `get()`, `getNumber()`, `getBoolean()`, `getJson<T>()`, `load()`, `update()`
- **pos-filters.svelte.ts** — `searchQuery`, `selectedCategoryId`

### Commands (`src/lib/commands/`)

Raw SQL via `$lib/db/index.ts` (select/execute wrappers). Page components never call DB directly.

- **auth.ts** — `login(pin)`, `getUsers()`, `createUser()`, `updateUser()`
- **products.ts** — CRUD + `adjustStock()`
- **categories.ts** — CRUD
- **customers.ts** — CRUD
- **orders.ts** — `getOrders()`, `getOrder()`, `getOrderItems()`, `getOrderPayments()`, `getNextOrderNumber()`, `createOrder()`, `addOrderItem()`, `completeOrder()`, `voidOrder()`
- **payments.ts** — `addPayment()`
- **refunds.ts** — `createRefund()`, `addRefundItem()`, `getRefundsByOrder()`, `setOrderRefunded()`
- **reports.ts** — `getSalesByPeriod()`, `getProductMetrics()`, `getInventorySummary()`
- **settings.ts** — `getAllSettings()`, `getSetting()`, `updateSetting()`, `getSettingsByGroup()`

### Database

SQLite with WAL mode, foreign keys, soft deletes (`deleted_at`), `updated_at` triggers.

**Conventions:**
- Monetary values: integers in **cents** (1099 = $10.99)
- Tax rates: integers in **basis points** (875 = 8.75%)
- Timestamps: ISO 8601 TEXT
- UUIDs on every entity for future cloud sync

**Tables:** users, categories, products, customers, orders, order_items, payments, refunds, refund_items, settings

Migration file: `src-tauri/migrations/001_initial.sql`

## Conventions

- SSR disabled globally (`ssr = false` in +layout.ts), static adapter with `fallback: 'index.html'`
- Tailwind v4: `@theme inline` directive, `@custom-variant dark`, no tailwind.config file
- Animation: `tw-animate-css` (not `tailwindcss-animate`)
- shadcn components installed via `npx shadcn-svelte@latest add <name> --yes`
- `WithElementRef<T>` uses `any` default for El generic (required for shadcn type compatibility)
- Must export `WithoutChildren`, `WithoutChildrenOrChild`, `WithoutChild` from `$lib/utils.ts`

## Keyboard Shortcuts

- **Ctrl/Cmd+K** — Focus search input on current page
- **F2** — Go to POS
- **F3** — Go to Orders
- **F4** — Go to Products
- **Numpad keys** — Work in login screen and payment modal

## Default Admin

- PIN: `1234`, Name: `Admin`, Role: `admin`

## Development

```bash
npm install          # Install frontend deps
npm run dev          # Start SvelteKit dev server (port 1420)
npm run tauri:dev    # Start full Tauri app in dev mode
npm run check        # Run svelte-check
npm run tauri:build  # Production build
```

## CI/CD

GitHub Actions workflow at `.github/workflows/publish.yml`:
- Triggers on `v*` tags
- Builds for Linux (x86_64), macOS (ARM + Intel), Windows (x86_64)
- Creates draft GitHub Release with all platform artifacts
- Requires `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets for auto-updater

## Agentic Team Workflow

This project uses the Agentic Team workflow for autonomous batch work processing.
Configuration is in `.agentic-team/config.yaml` — see that file for browser, deploy,
pipeline, and notification settings.

### Commands
- `/auto-sweep` — Full-project audit: find bugs, create work items
- `/auto-batch` — Queue and run batch of work items autonomously
- `/auto-deploy-for-review` — Deploy shippable items to review environment
- `/auto-review-shippable` — Human approval of shippable items
- `/auto-merge` — Merge approved items into main
- `/auto-item` — Create a single work item
- `/auto-brief` — Decompose a project brief into work items
- `/auto-batch-status` — Morning review dashboard
- `/auto-board` — Terminal Kanban board
- `/auto-sitrep` — Quick status overview

### Work Item Lifecycle (10 states)
```
queued → defining → planning → designing → implementing → reviewing → testing → browser-testing → shippable → done
```

### Pipeline Templates
- `fullstack` — All 10 states
- `backend` — Skip designing, browser-testing
- `frontend` — All 10 states
- `bugfix` — Skip planning, designing
- `refactor` — Skip defining, designing

### Human Gates
Only `shippable → done` requires human approval. All other transitions are automated.

### Testing Rules

**Every code change must include tests.** This is enforced by the pipeline — the QA
persona will send items back to implementing if tests are missing.

**Backend tests:**
- Every new endpoint or handler must have at least one test
- Every bug fix must include a regression test that would have caught the bug
- Test both success paths and error/edge-case paths (invalid input, auth failures, empty results)
- Run the full backend test suite before advancing past the testing stage

**Frontend tests:**
- Every new component must have at least one test
- Test user interactions (clicks, form submissions, navigation)
- Test error states and loading states
- Run the full frontend test suite before advancing past the testing stage

**Browser/UI tests (automated via Playwright):**
- Test all routes affected by the change at desktop (1280px) and mobile (375px)
- Test interactive flows end-to-end (not just isolated components)
- Screenshot evidence is required for visual changes
- Auth-gated routes require a dev auth bypass — if none exists, create one as a work item

**Test coverage expectations:**
- New code should not decrease overall test coverage
- Security-sensitive code (auth, input validation, permissions) must have dedicated tests
- Items tagged `test-coverage` must include the specific missing test described in the work item

### Project Structure
- `docs/work-items/{status}/` — Work item files organized by status (10 folders)
- `docs/work-items/index.yaml` — Master index
- `docs/work-items/{slug}/` — Per-item persona artifacts
- `docs/research/`, `docs/design/`, `docs/architecture/` — Support artifacts
- `docs/planning/` — Implementation plans
- `.agentic-team/` — Batch state, browser daemon, config
