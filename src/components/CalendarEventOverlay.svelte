<script lang="ts">
  import type { CalendarEvent, CalendarEventWithLane } from "../types/types";

  let {
    events,
    brightColors,
    selectedDate,
    secondsToHoursMinutes,
    startEdit,
  }: {
    events: CalendarEvent[];
    brightColors: Array<string>;
    selectedDate: string | null;
    secondsToHoursMinutes: (totalSeconds: number) => void;
    startEdit: (event: CalendarEvent) => void;
  } = $props();

  let selectedEvent = $state<CalendarEvent | null>(null);
  let displayEventName = $state<string | null>(null);
  let displayedEvents: CalendarEventWithLane[] = $derived.by(() => { return assignLanes(events.filter(e => e.event_date === selectedDate)) });
  const hours = Array.from({ length: 24 }, (_, i) => i);

  let maxLanes = $state<number>(0);
  const TOTAL_SECONDS = 86400;

  $effect(() => {
    if (selectedEvent !== null) {
      displayEventName = selectedEvent.event_name;
    }
  })

  $effect(() => {
    maxLanes = displayedEvents.length ? Math.max(...displayedEvents.map((e) => e.lane)) + 1 : 1;
  })

  function getLeft(start: number) {
    return (start / TOTAL_SECONDS) * 100;
  }

  function getWidth(start: number, end: number) {
    return ((end - start) / TOTAL_SECONDS) * 100;
  }

  function assignLanes(events: CalendarEvent[]) {
    if (!events.length) return [];

    const lanes = [];

    for (const event of events) {
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

    return lanes.flatMap((lane, index) => lane.map((event) => ({ ...event, lane: index })));
  }
</script>

<div id="addEventOverlay">
  <div id="mainContent">
    <div id="timeline">
      <div id="timelineOverflow" style="width: 400%;">
        <div id="timeAxisBackground"></div>
        <div id="timeAxis">
          {#each hours as hour}
            <div class="tick" style="left: {(hour / 24) * 100}%;">{hour.toString().padStart(2, '0')}</div>
          {/each}
        </div>

        <div id="eventsBar" style="min-height: calc(100vh - 161px); height: {maxLanes * 60}px;">
          <div class="gridlines"></div>
          {#each displayedEvents as event (event.id)}
            <button class="timelineEvent" style="position: absolute; left: {getLeft(event.event_start)}%; top: {event.lane * 60}px; width: {getWidth(event.event_start, event.event_end)}%;" onclick={() => { startEdit(event); }}>
              <div class="listedEventInfo" style="background-color: {event.color}; color: {brightColors.some(c => c === event.color) ? 'black' : '#f6f6f6'}; height: 50px; align-items: center;">
                <p>{secondsToHoursMinutes(event.event_start)}-{secondsToHoursMinutes(event.event_end)}</p>
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #addEventOverlay {
    position: absolute;
    display: flex;
    flex-direction: row;
    align-content: center;
    z-index: 10000;
    top: 70px;
    left: 300px;
    bottom: 0;
    width: calc(100% - 300px);
    height: 100vh - 20px;
    background-color: #0f0f0f;
  }

  #mainContent {
    position: relative;
    display: flex;
    flex-direction: row;
    justify-content: flex-end;
    flex: 1;
    width: 100%;
    height: calc(100vh - 90px);
    padding: 0 10px 10px 10px;
    gap: 5px;
  }

  #timeline {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    width: 100%;
    height: 100%;
    overflow: auto;
    scrollbar-gutter: stable;
  }

  #timeAxisBackground {
    position: absolute;
    left: 0;
    width: 100%;
    height: 40px;
    z-index: 1;
    border-bottom: 1px solid #444;
    background-color: #151515;
  }

  #timeAxis {
    position: sticky;
    top: 5px;
    z-index: 1;
    display: flex;
    align-items: center;
    height: 30px;
    margin: 5px 15px;
    pointer-events: none;
    white-space: nowrap;
  }

  #timeAxis .tick {
    position: absolute;
    transform: translateX(-50%);
    font-weight: 600;
  }

  #eventsBar {
    position: relative;
    margin: 10px 15px;
  }

  #eventsBar .gridlines {
    position: absolute;
    inset: 0;
    background-image: repeating-linear-gradient(to right, rgba(255,255,255,0.1) 0, rgba(255,255,255,0.1) 1px, transparent 1px, transparent calc(100% / 48));
    background-size: calc(100% - 1px) 100%;
    pointer-events: none;
  }

  #eventsBar .timelineEvent {
    padding: 0;
    background-color: none;
    border: none;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  #eventsBar .timelineEvent:hover {
    cursor: pointer;
    filter: brightness(0.8);
  }

  #timeline::-webkit-scrollbar {
    height: 6px;
  }

  #timeline::-webkit-scrollbar-track {
    margin-top: 40px;
  }

  #timeline::-webkit-scrollbar-corner {
    background-color: transparent;
  }

  .listedEventInfo {
    display: flex;
    flex-direction: row;
    width: 100%;
    padding: 10px;
    border-radius: 8px;
  }

  #eventsBar .timelineEvent:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,1);
  }

</style>
