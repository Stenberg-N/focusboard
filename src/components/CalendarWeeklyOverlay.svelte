<script lang="ts">
  import type { CalendarDay, CalendarEvent, CalendarEventWithLaneVertical } from "../types/types";

  let {
    days,
    events,
    brightColors,
    weeklyViewWidth,
    startEdit,
  }: {
    days: CalendarDay[];
    events: CalendarEvent[];
    brightColors: Array<string>;
    weeklyViewWidth: number;
    startEdit: (event: CalendarEvent) => void;
  } = $props();

  const dayNames = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  const hours = Array.from({ length: 24 }, (_, i) => i);
  const displayedEvents: CalendarEventWithLaneVertical[] = $derived.by(() => { return days.flatMap((day) => assignLanes(events.filter(e => e.event_date === day.isodate))); });
  const TOTAL_SECONDS = 86400;

  function getTop(start: number) {
    return (start / TOTAL_SECONDS) * 100;
  }

  function getHeight(end: number, start: number) {
    if (end === 0 && start === 0) return 1;
    return ((end - start) / TOTAL_SECONDS) * 100;
  }

  function getLeft(lanes: number, maxlanes: number) {
    return (lanes / maxlanes) * 100;
  }

  function assignLanes(events: CalendarEvent[]): CalendarEventWithLaneVertical[] {
    if (!events.length) return [];

    const sorted = events.sort((a, b) => a.event_start - b.event_start);

    const lanes: CalendarEvent[][] = [];

    for (const event of sorted) {
      let placed = false;

      for (const lane of lanes) {
        const overlaps = lane.some((e: CalendarEvent) => !(event.event_end <= e.event_start || event.event_start >= e.event_end));

        if (!overlaps) {
          lane.push(event);
          placed = true;
          break;
        }
      }

      if (!placed) {
        lanes.push([event]);
      }
    }

    const dayMaxLanes = lanes.length || 1;

    return lanes.flatMap((lane, index) => lane.map((event) => ({ ...event, lane: index, maxLanes: dayMaxLanes })));
  }
</script>

<div id="weeklyOverlay">
  <div id="mainContent">
    <div style="width: {weeklyViewWidth}%;">
      <div id="weekDayAxis">
        {#each dayNames as day (day)}
          <div class="weekDayName">{day}</div>
        {/each}
      </div>
      <div id="eventGrid">
        <div id="hours">
          {#each hours as hour (hour)}
            <div class="hourLabel">{hour.toString().padStart(2, '0')}:00</div>
          {/each}
        </div>
        <div id="timeSlots">
          {#each days as day (day)}
            <div class="daySlot">
              {#each displayedEvents.filter(e => e.event_date === day.isodate) as event (event.id)}
                <button class="weekEvent" onclick={() => startEdit(event)}
                  style="
                  background-color: {event.color}; color: {brightColors.some(c => c === event.color) ? 'black' : '#f6f6f6'};
                  top: {getTop(event.event_start)}%; height: {getHeight(event.event_end, event.event_start)}%; left: {getLeft(event.lane, event.maxLanes)}%; width: {(100 / event.maxLanes)}%;"
                >
                  {event.event_name}
                </button>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #weeklyOverlay {
    position: absolute;
    top: 70px;
    left: 300px;
    bottom: 0;
    z-index: 10000;
    display: flex;
    width: calc(100vw - 300px);
    background-color: #0f0f0f;
  }

  #mainContent {
    position: relative;
    display: flex;
    width: 100%;
    flex-direction: column;
    padding: 0 12px 12px 0;
    margin-right: 86px;
    overflow: auto;
  }

  #mainContent::-webkit-scrollbar-corner {
    display: none;
  }

  #mainContent::-webkit-scrollbar {
    height: 6px;
  }

  #weekDayAxis {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    max-height: 30px;
    margin: 0 0 5px 44px;
    gap: 10px;
    background-color: #0f0f0f;
    user-select: none;
  }

  .weekDayName {
    flex: 1 1 0;
    max-height: 30px;
    align-content: center;
    font-weight: 800;
  }

  #eventGrid {
    display: flex;
    flex: 1 1 0;
  }

  #hours {
    position: sticky;
    left: 0;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 100px;
    max-width: 44px;
    min-width: 44px;
    background-color: #0f0f0f;
  }

  .hourLabel {
    display: flex;
    align-items: center;
    height: 202px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  #timeSlots {
    position: relative;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    flex: 1;
    margin-top: 100px;
    background: repeating-linear-gradient(to bottom, transparent 0, transparent 100px, rgba(68, 68, 68, 0.3) 100px, rgba(68, 68, 68, 0.3) 101px);
    background-size: 100%;
    overflow-y: auto;
  }

  .daySlot {
    position: relative;
    flex: 1;
    border: 1px solid rgba(68, 68, 68, 0.3);
  }

  .daySlot:not(:last-child) {
    border-right: none;
  }

  .weekEvent {
    position: absolute;
    border: none;
    border-radius: 4px;
    padding: 4px 6px;
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .weekEvent:hover {
    cursor: pointer;
    opacity: 0.8;
  }
</style>