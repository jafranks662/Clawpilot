"use client";

import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";

const stages = ["idea", "research", "outline", "draft", "review", "design", "publish"];
const statuses = ["todo", "in_progress", "blocked", "done"];

function normalizeTag(rawTag) {
  return rawTag.trim().toLowerCase().slice(0, 20);
}

export default function Page() {
  const board = useQuery(api.mission.dashboard) || { tasks: [], pipeline: [], calendar: [], memories: [], agents: [] };
  const [memoryQuery, setMemoryQuery] = useState("");
  const filteredMemories = useQuery(api.mission.searchMemories, { query: memoryQuery }) || [];

  const [taskSearch, setTaskSearch] = useState("");
  const [taskStatusFilter, setTaskStatusFilter] = useState("all");
  const [selectedTaskTags, setSelectedTaskTags] = useState([]);
  const [savedTaskFilters, setSavedTaskFilters] = useState([]);

  const seed = useMutation(api.mission.seed);
  const createTask = useMutation(api.mission.createTask);
  const updateTask = useMutation(api.mission.updateTask);
  const addTaskTag = useMutation(api.mission.addTaskTag);
  const removeTaskTag = useMutation(api.mission.removeTaskTag);
  const upsertPipeline = useMutation(api.mission.upsertPipeline);
  const addPipelineTag = useMutation(api.mission.addPipelineTag);
  const removePipelineTag = useMutation(api.mission.removePipelineTag);
  const createCalendarEvent = useMutation(api.mission.createCalendarEvent);
  const createMemory = useMutation(api.mission.createMemory);

  useEffect(() => {
    seed();
  }, [seed]);

  const availableTaskTags = useMemo(
    () => [...new Set(board.tasks.flatMap((task) => task.tags || []))].sort(),
    [board.tasks]
  );

  const filteredTasks = useMemo(() => {
    const search = taskSearch.trim().toLowerCase();
    return board.tasks.filter((task) => {
      const taskTags = task.tags || [];
      const matchesSearch =
        !search ||
        task.title.toLowerCase().includes(search) ||
        (task.description || "").toLowerCase().includes(search);
      const matchesStatus = taskStatusFilter === "all" || task.status === taskStatusFilter;
      const matchesTags = selectedTaskTags.every((tag) => taskTags.includes(tag));
      return matchesSearch && matchesStatus && matchesTags;
    });
  }, [board.tasks, taskSearch, taskStatusFilter, selectedTaskTags]);

  const groupedTasks = useMemo(
    () =>
      statuses.reduce((acc, status) => {
        acc[status] = filteredTasks.filter((task) => task.status === status);
        return acc;
      }, {}),
    [filteredTasks]
  );

  const saveCurrentFilter = () => {
    const hasSelection = taskSearch.trim() || taskStatusFilter !== "all" || selectedTaskTags.length > 0;
    if (!hasSelection) return;

    const newFilter = {
      id: `${Date.now()}`,
      label: taskSearch.trim() || `status:${taskStatusFilter}`,
      search: taskSearch,
      status: taskStatusFilter,
      tags: selectedTaskTags
    };
    setSavedTaskFilters((current) => [newFilter, ...current].slice(0, 5));
  };

  const applySavedFilter = (filter) => {
    setTaskSearch(filter.search);
    setTaskStatusFilter(filter.status);
    setSelectedTaskTags(filter.tags);
  };

  return (
    <main className="page">
      <header className="hero">
        <h1>Mission Control</h1>
        <p>Realtime collaboration hub powered by Next.js + Convex.</p>
      </header>

      <section className="panel">
        <h2>Task Board</h2>
        <TaskComposer onCreate={createTask} />
        <TaskFilters
          taskSearch={taskSearch}
          onTaskSearch={setTaskSearch}
          taskStatusFilter={taskStatusFilter}
          onTaskStatusFilter={setTaskStatusFilter}
          availableTaskTags={availableTaskTags}
          selectedTaskTags={selectedTaskTags}
          onSelectedTaskTags={setSelectedTaskTags}
          onSave={saveCurrentFilter}
          savedTaskFilters={savedTaskFilters}
          onApplySavedFilter={applySavedFilter}
        />
        <div className="kanban">
          {statuses.map((status) => (
            <div className="column" key={status}>
              <h3>{status.replace("_", " ")}</h3>
              {groupedTasks[status]?.map((task) => (
                <article key={task._id} className="card">
                  <strong>{task.title}</strong>
                  <p>{task.description || "No description"}</p>
                  <TagEditor
                    tags={task.tags || []}
                    onAdd={(tag) => addTaskTag({ id: task._id, tag })}
                    onRemove={(tag) => removeTaskTag({ id: task._id, tag })}
                  />
                  <div className="row">
                    <select value={task.status} onChange={(e) => updateTask({ id: task._id, status: e.target.value })}>
                      {statuses.map((value) => (
                        <option key={value} value={value}>{value}</option>
                      ))}
                    </select>
                    <select value={task.assignee} onChange={(e) => updateTask({ id: task._id, assignee: e.target.value })}>
                      <option value="me">me</option>
                      <option value="you">you</option>
                    </select>
                  </div>
                </article>
              ))}
            </div>
          ))}
        </div>
      </section>

      <section className="panel">
        <h2>Content Pipeline</h2>
        <PipelineComposer onSave={upsertPipeline} />
        <div className="grid">
          {stages.map((stage) => (
            <div className="column" key={stage}>
              <h3>{stage}</h3>
              {board.pipeline.filter((item) => item.stage === stage).map((item) => (
                <article key={item._id} className="card">
                  <strong>{item.title}</strong>
                  <p><b>Owner:</b> {item.owner}</p>
                  <p>{item.brief || "No brief"}</p>
                  <TagEditor
                    tags={item.tags || []}
                    onAdd={(tag) => addPipelineTag({ id: item._id, tag })}
                    onRemove={(tag) => removePipelineTag({ id: item._id, tag })}
                  />
                  <details>
                    <summary>Script</summary>
                    <pre>{item.script || "No script yet"}</pre>
                  </details>
                  {item.imageUrls.length > 0 && <p><b>Images:</b> {item.imageUrls.join(", ")}</p>}
                </article>
              ))}
            </div>
          ))}
        </div>
      </section>

      <section className="panel two-col">
        <div>
          <h2>Calendar</h2>
          <CalendarComposer onCreate={createCalendarEvent} />
          {board.calendar
            .slice()
            .sort((a, b) => a.startAt - b.startAt)
            .map((event) => (
              <article key={event._id} className="card">
                <strong>{event.title}</strong>
                <p>{new Date(event.startAt).toLocaleString()} - {new Date(event.endAt).toLocaleString()}</p>
                <p><b>{event.category}</b> · {event.owner}</p>
                {event.notes && <p>{event.notes}</p>}
              </article>
            ))}
        </div>

        <div>
          <h2>Memory Documents</h2>
          <MemoryComposer onSave={createMemory} />
          <input
            placeholder="Search memories..."
            value={memoryQuery}
            onChange={(e) => setMemoryQuery(e.target.value)}
          />
          {filteredMemories.map((memory) => (
            <article key={memory._id} className="card document">
              <strong>{memory.title}</strong>
              <p>{memory.body}</p>
              <small>{memory.tags.join(" · ")}</small>
            </article>
          ))}
        </div>
      </section>

      <section className="panel two-col">
        <div>
          <h2>Team Structure</h2>
          {board.agents.map((agent) => (
            <article key={agent._id} className="card">
              <p className="avatar">{agent.avatar} {agent.name}</p>
              <p><b>{agent.role}</b> · {agent.status}</p>
              <p>{agent.responsibility}</p>
            </article>
          ))}
        </div>
        <div>
          <h2>Digital Office</h2>
          <div className="office">
            {board.agents.map((agent) => (
              <div className="desk" key={agent._id}>
                <div className="pc">💻</div>
                <div className="person">{agent.avatar}</div>
                <p>{agent.name}</p>
                <small>{agent.area}</small>
                <small className={agent.status}>{agent.status}</small>
              </div>
            ))}
          </div>
        </div>
      </section>
    </main>
  );
}

function TaskFilters({
  taskSearch,
  onTaskSearch,
  taskStatusFilter,
  onTaskStatusFilter,
  availableTaskTags,
  selectedTaskTags,
  onSelectedTaskTags,
  onSave,
  savedTaskFilters,
  onApplySavedFilter
}) {
  const toggleTag = (tag) => {
    onSelectedTaskTags(
      selectedTaskTags.includes(tag)
        ? selectedTaskTags.filter((value) => value !== tag)
        : [...selectedTaskTags, tag]
    );
  };

  return (
    <div className="filterBar">
      <input
        placeholder="Search title or description"
        value={taskSearch}
        onChange={(event) => onTaskSearch(event.target.value)}
      />
      <select value={taskStatusFilter} onChange={(event) => onTaskStatusFilter(event.target.value)}>
        <option value="all">all statuses</option>
        {statuses.map((status) => (
          <option key={status} value={status}>{status}</option>
        ))}
      </select>
      <div className="chipsWrap">
        {availableTaskTags.map((tag) => (
          <button
            key={tag}
            type="button"
            className={`chip ${selectedTaskTags.includes(tag) ? "chipActive" : ""}`}
            onClick={() => toggleTag(tag)}
          >
            #{tag}
          </button>
        ))}
      </div>
      <button type="button" onClick={onSave}>Save current filter</button>
      {savedTaskFilters.length > 0 && (
        <div className="chipsWrap">
          {savedTaskFilters.map((filter) => (
            <button key={filter.id} type="button" className="chip" onClick={() => onApplySavedFilter(filter)}>
              {filter.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function TagEditor({ tags, onAdd, onRemove }) {
  const [value, setValue] = useState("");

  const addTag = async () => {
    const normalized = normalizeTag(value);
    if (!normalized) return;
    await onAdd(normalized);
    setValue("");
  };

  return (
    <div>
      <div className="chipsWrap">
        {tags.map((tag) => (
          <button key={tag} type="button" className="chip" onClick={() => onRemove(tag)}>
            #{tag} ×
          </button>
        ))}
      </div>
      <input
        placeholder="Add tag"
        value={value}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            addTag();
          }
        }}
      />
    </div>
  );
}

function TaskComposer({ onCreate }) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [assignee, setAssignee] = useState("you");
  const [tags, setTags] = useState([]);
  const [tagInput, setTagInput] = useState("");

  const addTag = () => {
    const normalized = normalizeTag(tagInput);
    if (!normalized) return;
    setTags((current) => [...new Set([...current, normalized])]);
    setTagInput("");
  };

  const submit = async (event) => {
    event.preventDefault();
    if (!title.trim()) return;
    await onCreate({ title, description, assignee, tags });
    setTitle("");
    setDescription("");
    setTags([]);
  };

  return (
    <form className="composer" onSubmit={submit}>
      <input placeholder="Task title" value={title} onChange={(e) => setTitle(e.target.value)} />
      <input placeholder="Description" value={description} onChange={(e) => setDescription(e.target.value)} />
      <select value={assignee} onChange={(e) => setAssignee(e.target.value)}>
        <option value="me">me</option>
        <option value="you">you</option>
      </select>
      <div className="chipsWrap">
        {tags.map((tag) => (
          <button key={tag} type="button" className="chip" onClick={() => setTags((current) => current.filter((value) => value !== tag))}>
            #{tag} ×
          </button>
        ))}
      </div>
      <input
        placeholder="Add tag"
        value={tagInput}
        onChange={(event) => setTagInput(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            addTag();
          }
        }}
      />
      <button type="submit">Add task</button>
    </form>
  );
}

function PipelineComposer({ onSave }) {
  const [form, setForm] = useState({ title: "", stage: "idea", brief: "", script: "", imageUrls: "", owner: "you" });
  const [tags, setTags] = useState([]);
  const [tagInput, setTagInput] = useState("");

  const addTag = () => {
    const normalized = normalizeTag(tagInput);
    if (!normalized) return;
    setTags((current) => [...new Set([...current, normalized])]);
    setTagInput("");
  };

  const submit = async (event) => {
    event.preventDefault();
    if (!form.title.trim()) return;
    await onSave({
      title: form.title,
      stage: form.stage,
      brief: form.brief,
      script: form.script,
      imageUrls: form.imageUrls.split(",").map((value) => value.trim()).filter(Boolean),
      owner: form.owner,
      tags
    });
    setForm({ title: "", stage: "idea", brief: "", script: "", imageUrls: "", owner: "you" });
    setTags([]);
  };

  return (
    <form className="composer" onSubmit={submit}>
      <input placeholder="Content title" value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
      <select value={form.stage} onChange={(e) => setForm({ ...form, stage: e.target.value })}>
        {stages.map((stage) => <option key={stage} value={stage}>{stage}</option>)}
      </select>
      <select value={form.owner} onChange={(e) => setForm({ ...form, owner: e.target.value })}>
        <option value="me">me</option>
        <option value="you">you</option>
      </select>
      <input placeholder="Brief" value={form.brief} onChange={(e) => setForm({ ...form, brief: e.target.value })} />
      <textarea placeholder="Full script" value={form.script} onChange={(e) => setForm({ ...form, script: e.target.value })} />
      <input placeholder="Image URLs (comma separated)" value={form.imageUrls} onChange={(e) => setForm({ ...form, imageUrls: e.target.value })} />
      <div className="chipsWrap">
        {tags.map((tag) => (
          <button key={tag} type="button" className="chip" onClick={() => setTags((current) => current.filter((value) => value !== tag))}>
            #{tag} ×
          </button>
        ))}
      </div>
      <input
        placeholder="Add tag"
        value={tagInput}
        onChange={(event) => setTagInput(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            addTag();
          }
        }}
      />
      <button type="submit">Save pipeline item</button>
    </form>
  );
}

function CalendarComposer({ onCreate }) {
  const now = new Date();
  const defaultStart = new Date(now.getTime() + 60 * 60 * 1000).toISOString().slice(0, 16);
  const defaultEnd = new Date(now.getTime() + 2 * 60 * 60 * 1000).toISOString().slice(0, 16);
  const [form, setForm] = useState({ title: "", category: "cron", startAt: defaultStart, endAt: defaultEnd, owner: "you", notes: "" });

  const submit = async (event) => {
    event.preventDefault();
    if (!form.title.trim()) return;
    await onCreate({ ...form, startAt: new Date(form.startAt).getTime(), endAt: new Date(form.endAt).getTime() });
    setForm({ ...form, title: "", notes: "" });
  };

  return (
    <form className="composer" onSubmit={submit}>
      <input placeholder="Scheduled task" value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} />
      <select value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })}>
        <option value="cron">cron</option>
        <option value="meeting">meeting</option>
        <option value="delivery">delivery</option>
        <option value="focus">focus</option>
      </select>
      <input type="datetime-local" value={form.startAt} onChange={(e) => setForm({ ...form, startAt: e.target.value })} />
      <input type="datetime-local" value={form.endAt} onChange={(e) => setForm({ ...form, endAt: e.target.value })} />
      <select value={form.owner} onChange={(e) => setForm({ ...form, owner: e.target.value })}>
        <option value="me">me</option>
        <option value="you">you</option>
      </select>
      <input placeholder="Notes" value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
      <button type="submit">Add event</button>
    </form>
  );
}

function MemoryComposer({ onSave }) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [tags, setTags] = useState("");

  const submit = async (event) => {
    event.preventDefault();
    if (!title.trim() || !body.trim()) return;
    await onSave({
      title,
      body,
      tags: tags.split(",").map((value) => value.trim()).filter(Boolean)
    });
    setTitle("");
    setBody("");
    setTags("");
  };

  return (
    <form className="composer" onSubmit={submit}>
      <input placeholder="Memory title" value={title} onChange={(e) => setTitle(e.target.value)} />
      <textarea placeholder="Memory body" value={body} onChange={(e) => setBody(e.target.value)} />
      <input placeholder="Tags (comma separated)" value={tags} onChange={(e) => setTags(e.target.value)} />
      <button type="submit">Save memory</button>
    </form>
  );
}
