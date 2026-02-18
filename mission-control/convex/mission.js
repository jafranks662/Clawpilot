import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

async function writeActivity(ctx, entry) {
  const doc = {
    createdAt: Date.now(),
    ...entry
  };
  if (doc.metadata === undefined) {
    delete doc.metadata;
  }
  await ctx.db.insert("activity", doc);
}

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
    const id = await ctx.db.insert("tasks", {
      title: args.title,
      description: args.description,
      assignee: args.assignee,
      status: "todo",
      priority: "medium",
      createdAt: now,
      updatedAt: now
    });
    await writeActivity(ctx, {
      actor: args.assignee,
      entityType: "task",
      entityId: String(id),
      action: "task.created",
      summary: `${args.assignee} created task \"${args.title}\"`,
      metadata: { status: "todo" }
    });
    return id;
  }
});

export const updateTask = mutation({
  args: {
    id: v.id("tasks"),
    status: v.optional(v.union(v.literal("todo"), v.literal("in_progress"), v.literal("blocked"), v.literal("done"))),
    assignee: v.optional(v.union(v.literal("me"), v.literal("you"))),
    description: v.optional(v.string())
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      throw new Error("Task not found");
    }
    const patch = { updatedAt: Date.now() };
    if (args.status) patch.status = args.status;
    if (args.assignee) patch.assignee = args.assignee;
    if (args.description !== undefined) patch.description = args.description;
    await ctx.db.patch(args.id, patch);

    const nextStatus = args.status ?? existing.status;
    const nextAssignee = args.assignee ?? existing.assignee;
    const changes = {};
    if (args.status && args.status !== existing.status) {
      changes.status = { from: existing.status, to: args.status };
    }
    if (args.assignee && args.assignee !== existing.assignee) {
      changes.assignee = { from: existing.assignee, to: args.assignee };
    }
    if (args.description !== undefined && args.description !== existing.description) {
      changes.descriptionUpdated = true;
    }

    await writeActivity(ctx, {
      actor: nextAssignee,
      entityType: "task",
      entityId: String(args.id),
      action: args.status && args.status !== existing.status ? "task.moved" : "task.updated",
      summary: `${nextAssignee} updated task \"${existing.title}\"`,
      metadata: changes
    });
  }
});

export const deleteTask = mutation({
  args: { id: v.id("tasks") },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      throw new Error("Task not found");
    }
    await ctx.db.delete(args.id);
    await writeActivity(ctx, {
      actor: existing.assignee,
      entityType: "task",
      entityId: String(args.id),
      action: "task.deleted",
      summary: `${existing.assignee} deleted task \"${existing.title}\"`
    });
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
      const existing = await ctx.db.get(args.id);
      if (!existing) {
        throw new Error("Pipeline item not found");
      }
      await ctx.db.patch(args.id, payload);
      await writeActivity(ctx, {
        actor: args.owner,
        entityType: "pipeline",
        entityId: String(args.id),
        action: args.stage !== existing.stage ? "pipeline.stage_changed" : "pipeline.updated",
        summary: `${args.owner} updated pipeline item \"${args.title}\"`,
        metadata: args.stage !== existing.stage ? { from: existing.stage, to: args.stage } : undefined
      });
      return args.id;
    }
    const id = await ctx.db.insert("pipelineItems", payload);
    await writeActivity(ctx, {
      actor: args.owner,
      entityType: "pipeline",
      entityId: String(id),
      action: "pipeline.created",
      summary: `${args.owner} created pipeline item \"${args.title}\"`,
      metadata: { stage: args.stage }
    });
    return id;
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
    const id = await ctx.db.insert("calendarEvents", args);
    await writeActivity(ctx, {
      actor: args.owner,
      entityType: "calendar",
      entityId: String(id),
      action: "calendar.created",
      summary: `${args.owner} created calendar event \"${args.title}\"`,
      metadata: { category: args.category, startAt: args.startAt, endAt: args.endAt }
    });
    return id;
  }
});

export const updateCalendarEvent = mutation({
  args: {
    id: v.id("calendarEvents"),
    title: v.optional(v.string()),
    category: v.optional(v.union(v.literal("meeting"), v.literal("cron"), v.literal("delivery"), v.literal("focus"))),
    startAt: v.optional(v.number()),
    endAt: v.optional(v.number()),
    owner: v.optional(v.union(v.literal("me"), v.literal("you"))),
    notes: v.optional(v.string())
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      throw new Error("Calendar event not found");
    }
    const patch = {};
    if (args.title !== undefined) patch.title = args.title;
    if (args.category !== undefined) patch.category = args.category;
    if (args.startAt !== undefined) patch.startAt = args.startAt;
    if (args.endAt !== undefined) patch.endAt = args.endAt;
    if (args.owner !== undefined) patch.owner = args.owner;
    if (args.notes !== undefined) patch.notes = args.notes;
    await ctx.db.patch(args.id, patch);
    const owner = args.owner ?? existing.owner;
    await writeActivity(ctx, {
      actor: owner,
      entityType: "calendar",
      entityId: String(args.id),
      action: "calendar.updated",
      summary: `${owner} updated calendar event \"${args.title ?? existing.title}\"`,
      metadata: patch
    });
  }
});

export const deleteCalendarEvent = mutation({
  args: { id: v.id("calendarEvents") },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      throw new Error("Calendar event not found");
    }
    await ctx.db.delete(args.id);
    await writeActivity(ctx, {
      actor: existing.owner,
      entityType: "calendar",
      entityId: String(args.id),
      action: "calendar.deleted",
      summary: `${existing.owner} deleted calendar event \"${existing.title}\"`
    });
  }
});

export const createMemory = mutation({
  args: { title: v.string(), body: v.string(), tags: v.array(v.string()) },
  handler: async (ctx, args) => {
    const id = await ctx.db.insert("memories", { ...args, createdAt: Date.now() });
    await writeActivity(ctx, {
      actor: "me",
      entityType: "memory",
      entityId: String(id),
      action: "memory.added",
      summary: `me added memory \"${args.title}\"`,
      metadata: { tags: args.tags }
    });
    return id;
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
      const existing = await ctx.db.get(args.id);
      if (!existing) {
        throw new Error("Agent not found");
      }
      await ctx.db.patch(args.id, payload);
      await writeActivity(ctx, {
        actor: "you",
        entityType: "agent",
        entityId: String(args.id),
        action: args.status !== existing.status ? "agent.status_changed" : "agent.updated",
        summary: `you updated agent \"${args.name}\"`,
        metadata: args.status !== existing.status ? { from: existing.status, to: args.status } : undefined
      });
      return args.id;
    }
    const id = await ctx.db.insert("agents", payload);
    await writeActivity(ctx, {
      actor: "you",
      entityType: "agent",
      entityId: String(id),
      action: "agent.created",
      summary: `you created agent \"${args.name}\"`,
      metadata: { status: args.status, role: args.role }
    });
    return id;
  }
});

export const updateAgentStatus = mutation({
  args: {
    id: v.id("agents"),
    status: v.union(v.literal("working"), v.literal("idle"), v.literal("reviewing"))
  },
  handler: async (ctx, args) => {
    const existing = await ctx.db.get(args.id);
    if (!existing) {
      throw new Error("Agent not found");
    }
    await ctx.db.patch(args.id, { status: args.status, updatedAt: Date.now() });
    await writeActivity(ctx, {
      actor: "you",
      entityType: "agent",
      entityId: String(args.id),
      action: "agent.status_changed",
      summary: `you changed ${existing.name} to ${args.status}`,
      metadata: { from: existing.status, to: args.status }
    });
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
