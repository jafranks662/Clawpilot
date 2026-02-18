# Mission Control (Next.js + Convex)

This app provides a collaborative mission-control workspace with realtime screens for:

- Task board (status + assignee `me`/`you`)
- Lightweight personal boards for task organization
- Content pipeline (idea to publish, scripts, image URLs)
- Calendar for scheduled tasks and cron jobs
- Memory documents with search
- Team structure for primary/sub agents
- Digital office visualization with live statuses

## Setup

1. Install dependencies:

```bash
npm install
```

2. Start Convex in this folder (requires Convex login/config):

```bash
npx convex dev
```

3. Run Next.js in another terminal:

```bash
npm run dev
```

Set `NEXT_PUBLIC_CONVEX_URL` if your Convex URL differs from the default local URL.

## Boards + backfill behavior

- A default board is created automatically on app load through `boards.ensureDefaultBoard`.
- New tasks are created in whichever board is selected in the board dropdown.
- Existing tasks that pre-date boards can be backfilled from the UI.

### One-time backfill instructions

1. Open the app (`npm run dev`) and wait for Mission Control to load.
2. In the **Task Board** section, click **Run one-time backfill**.
3. Wait for the confirmation message (`Backfill complete. Updated N task(s).`).

This runs `boards.backfillTaskBoards`, which assigns every task without `boardId` to the default board.
