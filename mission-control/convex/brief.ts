import { query } from "./_generated/server";

const WEEK_IN_MS = 7 * 24 * 60 * 60 * 1000;

export const getDailyBrief = query({
  args: {},
  handler: async (ctx) => {
    const now = Date.now();
    const staleCutoff = now - WEEK_IN_MS;
    const eventsUntil = now + WEEK_IN_MS;

    const [allTasks, upcomingEvents] = await Promise.all([
      ctx.db.query("tasks").collect(),
      ctx.db
        .query("calendarEvents")
        .withIndex("by_start", (q) => q.gte("startAt", now).lte("startAt", eventsUntil))
        .collect()
    ]);

    const openTasks = allTasks.filter((task) => task.status !== "done");

    const priorities = openTasks
      .filter((task) => task.priority === "high")
      .sort((a, b) => b.updatedAt - a.updatedAt);

    const blocked = openTasks
      .filter((task) => task.status === "blocked")
      .sort((a, b) => b.updatedAt - a.updatedAt);

    const stale = openTasks
      .filter((task) => task.updatedAt < staleCutoff)
      .sort((a, b) => a.updatedAt - b.updatedAt);

    return {
      priorities,
      blocked,
      stale,
      upcomingEvents: upcomingEvents.sort((a, b) => a.startAt - b.startAt)
    };
  }
});
