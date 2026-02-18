import { mutation, query } from "./_generated/server";
import { v } from "convex/values";

const DEFAULT_BOARD_NAME = "Default";

async function getDefaultBoard(ctx) {
  return await ctx.db
    .query("boards")
    .withIndex("by_is_default", (q) => q.eq("isDefault", true))
    .first();
}

export const ensureDefaultBoard = mutation({
  args: {},
  handler: async (ctx) => {
    const defaultBoard = await getDefaultBoard(ctx);
    if (defaultBoard) {
      return defaultBoard._id;
    }

    return await ctx.db.insert("boards", {
      name: DEFAULT_BOARD_NAME,
      isDefault: true,
      createdAt: Date.now()
    });
  }
});

export const listBoards = query({
  args: {},
  handler: async (ctx) => {
    const boards = await ctx.db.query("boards").collect();
    return boards.sort((a, b) => {
      if (a.isDefault && !b.isDefault) return -1;
      if (!a.isDefault && b.isDefault) return 1;
      return a.name.localeCompare(b.name);
    });
  }
});

export const createBoard = mutation({
  args: { name: v.string() },
  handler: async (ctx, args) => {
    const name = args.name.trim();
    if (!name) {
      throw new Error("Board name is required");
    }

    return await ctx.db.insert("boards", {
      name,
      isDefault: false,
      createdAt: Date.now()
    });
  }
});

export const renameBoard = mutation({
  args: { boardId: v.id("boards"), name: v.string() },
  handler: async (ctx, args) => {
    const name = args.name.trim();
    if (!name) {
      throw new Error("Board name is required");
    }

    await ctx.db.patch(args.boardId, { name });
  }
});

export const deleteBoard = mutation({
  args: { boardId: v.id("boards") },
  handler: async (ctx, args) => {
    const board = await ctx.db.get(args.boardId);
    if (!board) {
      throw new Error("Board not found");
    }

    if (board.isDefault) {
      throw new Error("Default board cannot be deleted");
    }

    const defaultBoardId = await ensureDefaultBoard.handler(ctx, {});
    const tasks = await ctx.db
      .query("tasks")
      .withIndex("by_board", (q) => q.eq("boardId", args.boardId))
      .collect();

    await Promise.all(tasks.map((task) => ctx.db.patch(task._id, { boardId: defaultBoardId, updatedAt: Date.now() })));
    await ctx.db.delete(args.boardId);
  }
});

export const backfillTaskBoards = mutation({
  args: {},
  handler: async (ctx) => {
    const defaultBoardId = await ensureDefaultBoard.handler(ctx, {});
    const tasks = await ctx.db.query("tasks").collect();
    let updated = 0;

    for (const task of tasks) {
      if (!task.boardId) {
        await ctx.db.patch(task._id, { boardId: defaultBoardId, updatedAt: Date.now() });
        updated += 1;
      }
    }

    return { updated, defaultBoardId };
  }
});
