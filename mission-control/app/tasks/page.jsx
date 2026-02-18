"use client";

import { Suspense } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";

const WEEK_IN_MS = 7 * 24 * 60 * 60 * 1000;

function matchesFilter(task, filter, staleCutoff) {
  if (task.status === "done") return false;
  if (filter === "blocked") return task.status === "blocked";
  if (filter === "priority_high") return task.priority === "high";
  if (filter === "stale") return task.updatedAt < staleCutoff;
  return true;
}

function TasksList() {
  const searchParams = useSearchParams();
  const filter = searchParams.get("filter") || "all";
  const board = useQuery(api.mission.dashboard) || { tasks: [] };
  const staleCutoff = Date.now() - WEEK_IN_MS;

  const tasks = board.tasks
    .filter((task) => matchesFilter(task, filter, staleCutoff))
    .sort((a, b) => b.updatedAt - a.updatedAt);

  return (
    <>
      <header className="hero">
        <h1>Tasks</h1>
        <p>Filter: <b>{filter}</b></p>
        <Link href="/">← Back to Mission Control</Link>
      </header>

      <section className="panel">
        {tasks.map((task) => (
          <article className="card" key={task._id}>
            <strong>{task.title}</strong>
            <p>{task.description || "No description"}</p>
            <small>{task.status.replace("_", " ")} · {task.priority} · updated {new Date(task.updatedAt).toLocaleDateString()}</small>
          </article>
        ))}
        {tasks.length === 0 && <p>No matching tasks.</p>}
      </section>
    </>
  );
}

export default function TasksPage() {
  return (
    <main className="page">
      <Suspense fallback={<p>Loading tasks…</p>}>
        <TasksList />
      </Suspense>
    </main>
  );
}
