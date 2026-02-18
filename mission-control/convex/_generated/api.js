export const api = {
  mission: {
    dashboard: "mission:dashboard",
    createTask: "mission:createTask",
    updateTask: "mission:updateTask",
    upsertPipeline: "mission:upsertPipeline",
    createCalendarEvent: "mission:createCalendarEvent",
    createMemory: "mission:createMemory",
    searchMemories: "mission:searchMemories",
    upsertAgent: "mission:upsertAgent",
    seed: "mission:seed"
  },
  boards: {
    ensureDefaultBoard: "boards:ensureDefaultBoard",
    listBoards: "boards:listBoards",
    createBoard: "boards:createBoard",
    renameBoard: "boards:renameBoard",
    deleteBoard: "boards:deleteBoard",
    backfillTaskBoards: "boards:backfillTaskBoards"
  }
};
