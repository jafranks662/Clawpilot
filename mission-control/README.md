# Mission Control (Next.js + Convex)

This app provides a collaborative mission-control workspace with realtime screens for:

- Task board (status + assignee `me`/`you`)
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
