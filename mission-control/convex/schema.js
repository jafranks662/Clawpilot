import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
  tasks: defineTable({
    title: v.string(),
    description: v.optional(v.string()),
    status: v.union(v.literal("todo"), v.literal("in_progress"), v.literal("blocked"), v.literal("done")),
    assignee: v.union(v.literal("me"), v.literal("you")),
    priority: v.union(v.literal("low"), v.literal("medium"), v.literal("high")),
    createdAt: v.number(),
    updatedAt: v.number()
  }).index("by_status", ["status"]),
  pipelineItems: defineTable({
    title: v.string(),
    stage: v.union(
      v.literal("idea"),
      v.literal("research"),
      v.literal("outline"),
      v.literal("draft"),
      v.literal("review"),
      v.literal("design"),
      v.literal("publish")
    ),
    brief: v.optional(v.string()),
    script: v.optional(v.string()),
    imageUrls: v.array(v.string()),
    owner: v.union(v.literal("me"), v.literal("you")),
    updatedAt: v.number()
  }).index("by_stage", ["stage"]),
  calendarEvents: defineTable({
    title: v.string(),
    category: v.union(v.literal("meeting"), v.literal("cron"), v.literal("delivery"), v.literal("focus")),
    startAt: v.number(),
    endAt: v.number(),
    owner: v.union(v.literal("me"), v.literal("you")),
    notes: v.optional(v.string())
  }).index("by_start", ["startAt"]),
  memories: defineTable({
    title: v.string(),
    body: v.string(),
    tags: v.array(v.string()),
    createdAt: v.number()
  }).searchIndex("search_body", {
    searchField: "body",
    filterFields: ["tags"]
  }),
  agents: defineTable({
    name: v.string(),
    role: v.union(v.literal("developer"), v.literal("writer"), v.literal("designer"), v.literal("operator")),
    responsibility: v.string(),
    status: v.union(v.literal("working"), v.literal("idle"), v.literal("reviewing")),
    area: v.string(),
    avatar: v.string(),
    updatedAt: v.number()
  }).index("by_role", ["role"]),
  activity: defineTable({
    createdAt: v.number(),
    actor: v.union(v.literal("me"), v.literal("you")),
    entityType: v.union(v.literal("task"), v.literal("pipeline"), v.literal("memory"), v.literal("agent"), v.literal("calendar")),
    entityId: v.string(),
    action: v.string(),
    summary: v.string(),
    metadata: v.optional(v.any())
  })
    .index("by_createdAt", ["createdAt"])
    .index("by_entity", ["entityType", "entityId"])
});
