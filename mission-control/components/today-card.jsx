import Link from "next/link";

const MAX_ITEMS = 3;

function formatDate(timestamp) {
  return new Date(timestamp).toLocaleDateString();
}

function Section({ title, count, items, emptyText, href, renderMeta }) {
  return (
    <article className="brief-section">
      <div className="brief-header-row">
        <h3>{title}</h3>
        <span className="brief-count">{count}</span>
      </div>
      {items.length > 0 ? (
        <ul>
          {items.slice(0, MAX_ITEMS).map((item) => (
            <li key={item._id}>
              <strong>{item.title}</strong>
              {renderMeta ? <small>{renderMeta(item)}</small> : null}
            </li>
          ))}
        </ul>
      ) : (
        <p className="brief-empty">{emptyText}</p>
      )}
      <Link href={href}>View all</Link>
    </article>
  );
}

export function TodayCard({ brief }) {
  if (!brief) {
    return (
      <section className="panel brief-panel">
        <h2>Today</h2>
        <p className="brief-empty">Loading daily brief...</p>
      </section>
    );
  }

  return (
    <section className="panel brief-panel">
      <h2>Today</h2>
      <div className="brief-grid">
        <Section
          title="Top priorities"
          count={brief.priorities.length}
          items={brief.priorities}
          emptyText="No high-priority tasks pending."
          href="/tasks?filter=priority_high"
          renderMeta={(task) => `${task.status.replace("_", " ")} · ${task.assignee}`}
        />
        <Section
          title="Blocked"
          count={brief.blocked.length}
          items={brief.blocked}
          emptyText="No blocked tasks."
          href="/tasks?filter=blocked"
          renderMeta={(task) => `Updated ${formatDate(task.updatedAt)}`}
        />
        <Section
          title="Stale (7+ days)"
          count={brief.stale.length}
          items={brief.stale}
          emptyText="No stale tasks."
          href="/tasks?filter=stale"
          renderMeta={(task) => `Updated ${formatDate(task.updatedAt)}`}
        />
        <Section
          title="Upcoming events"
          count={brief.upcomingEvents.length}
          items={brief.upcomingEvents}
          emptyText="No events in the next 7 days."
          href="/calendar"
          renderMeta={(event) => new Date(event.startAt).toLocaleString()}
        />
      </div>
    </section>
  );
}
