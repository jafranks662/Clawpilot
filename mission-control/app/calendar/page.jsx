"use client";

import Link from "next/link";
import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";

const WEEK_IN_MS = 7 * 24 * 60 * 60 * 1000;

export default function CalendarPage() {
  const board = useQuery(api.mission.dashboard) || { calendar: [] };
  const now = Date.now();
  const weekAhead = now + WEEK_IN_MS;

  const upcomingEvents = board.calendar
    .filter((event) => event.startAt >= now && event.startAt <= weekAhead)
    .sort((a, b) => a.startAt - b.startAt);

  return (
    <main className="page">
      <header className="hero">
        <h1>Calendar</h1>
        <p>Upcoming events in the next 7 days.</p>
        <Link href="/">← Back to Mission Control</Link>
      </header>

      <section className="panel">
        {upcomingEvents.map((event) => (
          <article className="card" key={event._id}>
            <strong>{event.title}</strong>
            <p>{new Date(event.startAt).toLocaleString()} - {new Date(event.endAt).toLocaleString()}</p>
            <small>{event.category} · {event.owner}</small>
            {event.notes ? <p>{event.notes}</p> : null}
          </article>
        ))}
        {upcomingEvents.length === 0 && <p>No upcoming events in the next week.</p>}
      </section>
    </main>
  );
}
