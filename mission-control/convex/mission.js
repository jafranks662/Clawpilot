import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

export const dashboard = query({
  args: {},
  handler: async (ctx) => {
    const [tasks, pipeline, calendar, memories, agents] = await Promise.all([
      ctx.db.query("tasks").collect(),
      ctx.db.query("pipelineItems").collect(),
      ctx.db.query("calendarEvents").collect(),
      ctx.db.query("memories").order("desc").take(20),
      ctx.db.query("agents").collect()
    ]);
    return { tasks, pipeline, calendar, memories, agents };
  }
});

export const createTask = mutation({
  args: {
    title: v.string(),
    description: v.optional(v.string()),
    assignee: v.union(v.literal("me"), v.literal("you"))
  },
  handler: async (ctx, args) => {
    const now = Date.now();
    await ctx.db.insert("tasks", {
      title: args.title,
      description: args.description,
      assignee: args.assignee,
      status: "todo",
      priority: "medium",
      createdAt: now,
      updatedAt: now
    });
  }
});

export const updateTask = mutation({
  args: {
    id: v.id("tasks"),
    status: v.optional(v.union(v.literal("todo"), v.literal("in_progress"), v.literal("blocked"), v.literal("done"))),
    assignee: v.optional(v.union(v.literal("me"), v.literal("you"))),
    priority: v.optional(v.union(v.literal("low"), v.literal("medium"), v.literal("high"))),
    description: v.optional(v.string())
  },
  handler: async (ctx, args) => {
    const patch = { updatedAt: Date.now() };
    if (args.status) patch.status = args.status;
    if (args.assignee) patch.assignee = args.assignee;
    if (args.priority) patch.priority = args.priority;
    if (args.description !== undefined) patch.description = args.description;
    await ctx.db.patch(args.id, patch);
  }
});

export const upsertPipeline = mutation({
  args: {
    id: v.optional(v.id("pipelineItems")),
    title: v.string(),
    stage: v.union(v.literal("idea"), v.literal("research"), v.literal("outline"), v.literal("draft"), v.literal("review"), v.literal("design"), v.literal("publish")),
    brief: v.optional(v.string()),
    script: v.optional(v.string()),
    imageUrls: v.array(v.string()),
    owner: v.union(v.literal("me"), v.literal("you"))
  },
  handler: async (ctx, args) => {
    const payload = { ...args, updatedAt: Date.now() };
    delete payload.id;
    if (args.id) {
      await ctx.db.patch(args.id, payload);
      return args.id;
    }
    return await ctx.db.insert("pipelineItems", payload);
  }
});

export const createCalendarEvent = mutation({
  args: {
    title: v.string(),
    category: v.union(v.literal("meeting"), v.literal("cron"), v.literal("delivery"), v.literal("focus")),
    startAt: v.number(),
    endAt: v.number(),
    owner: v.union(v.literal("me"), v.literal("you")),
    notes: v.optional(v.string())
  },
  handler: async (ctx, args) => {
    return await ctx.db.insert("calendarEvents", args);
  }
});

export const createMemory = mutation({
  args: { title: v.string(), body: v.string(), tags: v.array(v.string()) },
  handler: async (ctx, args) => {
    return await ctx.db.insert("memories", { ...args, createdAt: Date.now() });
  }
});

export const searchMemories = query({
  args: { query: v.string() },
  handler: async (ctx, args) => {
    if (!args.query.trim()) {
      return await ctx.db.query("memories").order("desc").take(30);
    }
    return await ctx.db
      .query("memories")
      .withSearchIndex("search_body", (q) => q.search("body", args.query))
      .take(30);
  }
});

export const upsertAgent = mutation({
  args: {
    id: v.optional(v.id("agents")),
    name: v.string(),
    role: v.union(v.literal("developer"), v.literal("writer"), v.literal("designer"), v.literal("operator")),
    responsibility: v.string(),
    status: v.union(v.literal("working"), v.literal("idle"), v.literal("reviewing")),
    area: v.string(),
    avatar: v.string()
  },
  handler: async (ctx, args) => {
    const payload = { ...args, updatedAt: Date.now() };
    delete payload.id;
    if (args.id) {
      await ctx.db.patch(args.id, payload);
      return args.id;
    }
    return await ctx.db.insert("agents", payload);
  }
});

export const seed = mutation({
  args: {},
  handler: async (ctx) => {
    const hasAgents = await ctx.db.query("agents").first();
    if (!hasAgents) {
      const now = Date.now();
      await ctx.db.insert("agents", {
        name: "ZeroClawAgent",
        role: "operator",
        responsibility: "Coordinates mission control, schedules tasks, and manages global queue.",
        status: "working",
        area: "Command Deck",
        avatar: "🤖",
        updatedAt: now
      });
      await ctx.db.insert("agents", {
        name: "zeroclaw_dev",
        role: "developer",
        responsibility: "Builds product features, tests integrations, maintains runtime quality.",
        status: "working",
        area: "Engineering Bay",
        avatar: "🛠️",
        updatedAt: now
      });
      await ctx.db.insert("agents", {
        name: "zeroclaw_writer",
        role: "writer",
        responsibility: "Develops briefs, scripts, and publish-ready content artifacts.",
        status: "reviewing",
        area: "Content Studio",
        avatar: "✍️",
        updatedAt: now
      });
      await ctx.db.insert("agents", {
        name: "zeroclaw_designer",
        role: "designer",
        responsibility: "Creates visuals, layouts, and media-ready creative assets.",
        status: "idle",
        area: "Design Lab",
        avatar: "🎨",
        updatedAt: now
      });
    }
  }
});
