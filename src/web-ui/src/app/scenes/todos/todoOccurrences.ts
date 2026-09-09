/**
 * Turns scheduled jobs into dated occurrences and splits them into the two
 * tiers the Todos scene renders: a list for everything due within 24 hours and
 * a calendar for everything further out.
 *
 * Occurrence times are derived the same way the backend derives them, so the
 * panel never advertises a run the scheduler will not perform. Cron semantics
 * live in `cronExpression.ts`; `at` and `every` are handled here.
 */

import type { CronJob, CronSchedule } from '@/infrastructure/api';
import { getNextExecutionAtMs } from '@/app/components/scheduled-jobs/scheduledJobDraft';
import { cronOccurrencesInRange } from './cronExpression';

export const DAY_IN_MS = 24 * 60 * 60 * 1000;

/**
 * Per-job caps that keep a high-frequency schedule from swamping either tier.
 *
 * A five-minute Todo fires 288 times a day, which is neither readable as a list
 * nor useful as calendar chips. Each tier takes only the leading runs it can
 * show; the row's schedule summary ("Every 5 minutes") is what tells the reader
 * it keeps going. The calendar budget is per day rather than per month so a
 * frequent job still marks every day it runs instead of exhausting its budget
 * in the first few hours.
 */
export const MAX_LIST_OCCURRENCES_PER_JOB = 3;
export const MAX_CALENDAR_OCCURRENCES_PER_JOB_PER_DAY = 4;

/** Default cap for a single `scheduleOccurrencesInRange` call. */
export const MAX_OCCURRENCES_PER_JOB = 200;

export interface TodoOccurrence {
  job: CronJob;
  /** When this run fires. */
  atMs: number;
  /** True when the scheduler has this exact run queued as its next action. */
  isNextRun: boolean;
  /** True when the run time has passed and the job is not currently running. */
  isOverdue: boolean;
  /** True while the scheduler is executing this run. */
  isRunning: boolean;
}

/**
 * Why a job has nothing upcoming — drives the "no upcoming run" group so a
 * paused or finished job stays reachable instead of vanishing from the panel.
 */
export type InactiveReason = 'disabled' | 'completed' | 'invalid';

export interface InactiveTodo {
  job: CronJob;
  reason: InactiveReason;
  /** Parse/validation failure text, when `reason` is `invalid`. */
  error?: string;
}

export interface TodoBuckets {
  /** Due within 24 hours (including overdue and running), ascending. */
  upcoming: TodoOccurrence[];
  /** More than 24 hours out, ascending. */
  later: TodoOccurrence[];
  /**
   * Everything still scheduled, near-term included, for the calendar tier.
   *
   * The calendar shows the whole agenda rather than only the far half: a run
   * happening today is exactly what someone opening a calendar expects to see.
   * Only jobs with nothing left to run stay out of it.
   */
  calendar: TodoOccurrence[];
  /** Jobs with no upcoming run at all. */
  inactive: InactiveTodo[];
}

/**
 * Whether the scheduler is executing this job right now.
 *
 * A running job keeps its pending trigger timestamp in the past until the turn
 * finishes, so without this check an in-flight run reads as overdue.
 */
export function isJobRunning(job: CronJob): boolean {
  return job.state.activeTurnId != null
    || job.state.lastRunStatus === 'running'
    || job.state.lastRunStatus === 'queued';
}

export interface BuildTodoBucketsOptions {
  nowMs: number;
  /** End of the calendar window, normally the last day of the visible month. */
  rangeEndMs: number;
  /** Boundary between the list tier and the calendar tier. */
  listWindowMs?: number;
}

function everyOccurrencesInRange(
  everyMs: number,
  anchorMs: number,
  afterMs: number,
  untilMs: number,
  limit: number,
): number[] {
  if (!Number.isFinite(everyMs) || everyMs <= 0) return [];

  // Mirrors compute_every_next_run_ms: the series is anchored, not relative to
  // "now", so a job created at 08:03 with a 1h interval fires at :03.
  let first: number;
  if (anchorMs > afterMs) {
    first = anchorMs;
  } else {
    const steps = Math.floor((afterMs - anchorMs) / everyMs) + 1;
    first = anchorMs + steps * everyMs;
  }

  const occurrences: number[] = [];
  for (let atMs = first; atMs <= untilMs && occurrences.length < limit; atMs += everyMs) {
    occurrences.push(atMs);
  }
  return occurrences;
}

/**
 * Lists the runs a schedule produces in `(afterMs, untilMs]`.
 *
 * Throws only for schedules the backend would also reject; callers surface that
 * as an inactive job rather than dropping it silently.
 */
export function scheduleOccurrencesInRange(
  schedule: CronSchedule,
  createdAtMs: number,
  afterMs: number,
  untilMs: number,
  limit = MAX_OCCURRENCES_PER_JOB,
): number[] {
  if (schedule.kind === 'at') {
    const atMs = new Date(schedule.at).getTime();
    if (!Number.isFinite(atMs)) {
      throw new Error(`Invalid ISO-8601 timestamp '${schedule.at}'`);
    }
    return atMs > afterMs && atMs <= untilMs ? [atMs] : [];
  }

  if (schedule.kind === 'every') {
    if (!Number.isFinite(schedule.everyMs) || schedule.everyMs <= 0) {
      throw new Error('Recurring schedule everyMs must be greater than 0');
    }
    return everyOccurrencesInRange(
      schedule.everyMs,
      schedule.anchorMs ?? createdAtMs,
      afterMs,
      untilMs,
      limit,
    );
  }

  return cronOccurrencesInRange(schedule.expr, schedule.tz, {
    afterMs,
    untilMs,
    limit,
  });
}

/**
 * Expands the calendar tier one local day at a time.
 *
 * Budgeting per day rather than across the whole window keeps a frequent job
 * visible on every day it runs; a single windowed call would spend the entire
 * budget on the first day and leave the rest of the month looking empty.
 */
function expandCalendarOccurrences(
  job: CronJob,
  fromMs: number,
  untilMs: number,
): number[] {
  if (untilMs <= fromMs) return [];

  const occurrences: number[] = [];
  const cursor = new Date(fromMs);
  const lastDay = new Date(untilMs);

  // Start at the day containing `fromMs`; its earlier hours are excluded by the
  // exclusive lower bound below.
  cursor.setHours(0, 0, 0, 0);
  lastDay.setHours(0, 0, 0, 0);

  while (cursor.getTime() <= lastDay.getTime()) {
    const dayStart = cursor.getTime();
    const dayEnd = new Date(
      cursor.getFullYear(), cursor.getMonth(), cursor.getDate(), 23, 59, 59, 999,
    ).getTime();

    occurrences.push(...scheduleOccurrencesInRange(
      job.schedule,
      job.createdAtMs,
      Math.max(dayStart - 1, fromMs),
      Math.min(dayEnd, untilMs),
      MAX_CALENDAR_OCCURRENCES_PER_JOB_PER_DAY,
    ));

    cursor.setDate(cursor.getDate() + 1);
  }

  return occurrences;
}

/**
 * A one-shot job whose run already happened is finished, not broken.
 *
 * The backend clears `nextRunAtMs` once it enqueues a one-shot run, so the
 * absence of a next run plus a recorded run is what marks it complete.
 */
function hasCompletedOneShot(job: CronJob): boolean {
  if (job.schedule.kind !== 'at') return false;
  if (job.handling === 'manual') {
    return job.completionStatus === 'completed';
  }
  return (
    job.state.lastEnqueuedAtMs != null
    || job.state.lastRunFinishedAtMs != null
    || job.state.lastRunStartedAtMs != null
  );
}

/**
 * Groups every job into the list tier, the calendar tier, or the inactive
 * group. `rangeEndMs` bounds calendar expansion; the list tier always covers
 * the full 24-hour window even when the calendar is showing a later month.
 */
export function buildTodoBuckets(
  jobs: CronJob[],
  { nowMs, rangeEndMs, listWindowMs = DAY_IN_MS }: BuildTodoBucketsOptions,
): TodoBuckets {
  const upcoming: TodoOccurrence[] = [];
  const later: TodoOccurrence[] = [];
  const inactive: InactiveTodo[] = [];

  const listBoundaryMs = nowMs + listWindowMs;
  const expansionEndMs = Math.max(rangeEndMs, listBoundaryMs);

  for (const job of jobs) {
    if (!job.enabled) {
      inactive.push({ job, reason: 'disabled' });
      continue;
    }

    const scheduledAtMs = getNextExecutionAtMs(job);

    let occurrenceTimes: number[];
    try {
      // Expand from just before the scheduler's own next run when that is in
      // the past, so an overdue or retrying run still shows up.
      const expandAfterMs = scheduledAtMs != null
        ? Math.min(scheduledAtMs - 1, nowMs)
        : nowMs;

      occurrenceTimes = [
        ...scheduleOccurrencesInRange(
          job.schedule,
          job.createdAtMs,
          expandAfterMs,
          listBoundaryMs,
          MAX_LIST_OCCURRENCES_PER_JOB,
        ),
        ...expandCalendarOccurrences(job, listBoundaryMs, expansionEndMs),
      ];
    } catch (error) {
      inactive.push({
        job,
        reason: 'invalid',
        error: error instanceof Error ? error.message : String(error),
      });
      continue;
    }

    // A one-shot that already ran expands to nothing; report it as finished
    // rather than as a job with an unexplained empty schedule.
    if (occurrenceTimes.length === 0 && scheduledAtMs == null) {
      inactive.push({
        job,
        reason: hasCompletedOneShot(job) ? 'completed' : 'invalid',
      });
      continue;
    }

    // The scheduler's pending trigger wins over a recomputed time: a retry sits
    // off the nominal series and is still the run that will actually happen.
    if (scheduledAtMs != null && !occurrenceTimes.includes(scheduledAtMs)) {
      occurrenceTimes.unshift(scheduledAtMs);
      occurrenceTimes.sort((a, b) => a - b);
    }

    const running = isJobRunning(job);

    for (const atMs of occurrenceTimes) {
      const occurrence: TodoOccurrence = {
        job,
        atMs,
        isNextRun: atMs === scheduledAtMs,
        // A run that is executing is late only in the clock sense; surfacing it
        // as overdue misreports healthy work as a problem.
        isOverdue: atMs < nowMs && !running,
        isRunning: running && atMs === scheduledAtMs,
      };
      if (atMs <= listBoundaryMs) {
        upcoming.push(occurrence);
      } else {
        later.push(occurrence);
      }
    }
  }

  const byTime = (a: TodoOccurrence, b: TodoOccurrence) =>
    a.atMs - b.atMs || a.job.name.localeCompare(b.job.name);

  upcoming.sort(byTime);
  later.sort(byTime);
  inactive.sort((a, b) => b.job.configUpdatedAtMs - a.job.configUpdatedAtMs);

  const calendar = [...upcoming, ...later].sort(byTime);

  return { upcoming, later, calendar, inactive };
}

/** Local calendar-day key (`YYYY-MM-DD`) used to bucket occurrences into cells. */
export function localDayKey(timestampMs: number): string {
  const date = new Date(timestampMs);
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${date.getFullYear()}-${month}-${day}`;
}

export function groupOccurrencesByDay(
  occurrences: TodoOccurrence[],
): Map<string, TodoOccurrence[]> {
  const byDay = new Map<string, TodoOccurrence[]>();
  for (const occurrence of occurrences) {
    const key = localDayKey(occurrence.atMs);
    const bucket = byDay.get(key);
    if (bucket) {
      bucket.push(occurrence);
    } else {
      byDay.set(key, [occurrence]);
    }
  }
  return byDay;
}

/** Six-week grid (42 local days) covering the month `monthAnchorMs` falls in. */
export function buildMonthGrid(monthAnchorMs: number): Date[] {
  const anchor = new Date(monthAnchorMs);
  const firstOfMonth = new Date(anchor.getFullYear(), anchor.getMonth(), 1);
  const gridStart = new Date(firstOfMonth);
  gridStart.setDate(firstOfMonth.getDate() - firstOfMonth.getDay());

  return Array.from({ length: 42 }, (_, index) => {
    const day = new Date(gridStart);
    day.setDate(gridStart.getDate() + index);
    return day;
  });
}

export function monthRangeMs(monthAnchorMs: number): { startMs: number; endMs: number } {
  const grid = buildMonthGrid(monthAnchorMs);
  const first = grid[0];
  const last = grid[grid.length - 1];
  return {
    startMs: new Date(first.getFullYear(), first.getMonth(), first.getDate()).getTime(),
    endMs: new Date(last.getFullYear(), last.getMonth(), last.getDate(), 23, 59, 59, 999).getTime(),
  };
}

/** Left-pane filter categories. */
export type TodoTab = 'today' | 'inProgress' | 'pending' | 'all';

export const TODO_TAB_ORDER: readonly TodoTab[] = ['today', 'inProgress', 'pending', 'all'];

function startOfLocalDayMs(nowMs: number): number {
  const date = new Date(nowMs);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

function endOfLocalDayMs(nowMs: number): number {
  const date = new Date(nowMs);
  return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999).getTime();
}

/**
 * Whether an unfinished Todo is due today: its planned completion is today or
 * already overdue, or — when no planned completion is set — it has a run
 * scheduled for today.
 */
export function isDueToday(job: CronJob, nowMs: number): boolean {
  if (job.completionStatus === 'completed') return false;
  if (job.plannedCompletionAtMs != null) {
    return job.plannedCompletionAtMs <= endOfLocalDayMs(nowMs);
  }
  const startMs = startOfLocalDayMs(nowMs);
  const endMs = endOfLocalDayMs(nowMs);
  try {
    return scheduleOccurrencesInRange(
      job.schedule,
      job.createdAtMs,
      startMs - 1,
      endMs,
      1,
    ).length > 0;
  } catch {
    return false;
  }
}

function sortJobsForList(jobs: CronJob[]): CronJob[] {
  return [...jobs].sort((a, b) => {
    const aMs = getNextExecutionAtMs(a) ?? a.plannedCompletionAtMs ?? Number.MAX_SAFE_INTEGER;
    const bMs = getNextExecutionAtMs(b) ?? b.plannedCompletionAtMs ?? Number.MAX_SAFE_INTEGER;
    if (aMs !== bMs) return aMs - bMs;
    return a.name.localeCompare(b.name);
  });
}

/** Filters the full job list for a left-pane tab and returns it in display order. */
export function filterJobsByTab(jobs: CronJob[], tab: TodoTab, nowMs: number): CronJob[] {
  let filtered: CronJob[];
  switch (tab) {
    case 'today':
      filtered = jobs.filter((job) => isDueToday(job, nowMs));
      break;
    case 'inProgress':
      filtered = jobs.filter((job) => job.completionStatus === 'in_progress');
      break;
    case 'pending':
      filtered = jobs.filter((job) => job.completionStatus !== 'completed');
      break;
    case 'all':
      filtered = jobs;
      break;
  }
  return sortJobsForList(filtered);
}
