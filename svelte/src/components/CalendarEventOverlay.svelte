<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import type { CalendarEvent, CalendarEventWithLane } from "../types/types";

  let {
    events,
    colors,
    brightColors,
    selectedDate,
    selectedDateClean,
    yearMonth,
    setStatus,
    getEvents,
    secondsToHoursMinutes,
    setSelectedDate,
  }: {
    events: CalendarEvent[];
    colors: Array<string>;
    brightColors: Array<string>;
    selectedDate: string | null;
    selectedDateClean: string | null;
    yearMonth: string;
    setStatus: (msg: string) => void;
    getEvents: () => void;
    secondsToHoursMinutes: (totalSeconds: number) => void;
    setSelectedDate: (date: string | null) => void;
  } = $props();

  let eventNameInput = $state<HTMLInputElement>();
  let eventStartHours = $state<number>(0);
  let eventStartMinutes = $state<number>(0);
  let eventEndHours = $state<number>(0);
  let eventEndMinutes = $state<number>(0);
  let deleteEventId = $state<number | null>(null);

  let editEventNameInput = $state<HTMLInputElement>();
  let editEventStartHours = $state<number>(0);
  let editEventStartMinutes = $state<number>(0);
  let editEventEndHours = $state<number>(0);
  let editEventEndMinutes = $state<number>(0);

  let showAddEvent = $state<boolean>(false);
  let selectedEvent = $state<CalendarEvent | null>(null);
  let displayEventName = $state<string | null>(null);
  let displayedEvents = $state<CalendarEventWithLane[]>([]);
  const hours = Array.from({ length: 25 }, (_, i) => i);

  let height = $state<number>(0);
  let maxLanes = $state<number>(0);
  const TOTAL_SECONDS = 86400;

  $effect(() => {
    if (selectedEvent !== null) {
      displayEventName = selectedEvent.event_name;
    }
  })

  $effect(() => {
    if (showAddEvent) height = 364;
    if (selectedEvent) height = 388;
    if (showAddEvent && selectedEvent) height = 732;
  })

  $effect(() => {
    displayedEvents = assignLanes(events.filter(e => e.event_date === selectedDate))
  })

  $effect(() => {
    maxLanes = displayedEvents.length ? Math.max(...displayedEvents.map((e) => e.lane)) + 1 : 1;
  })

  async function saveEvent() {
    try {
      if (eventNameInput?.value.trim() == '') {
        setStatus("Event name missing! Please add a name for the event");
        return;
      }

      let randomColor = colors[Math.floor(Math.random() * colors.length)];
      let timeStart = (eventStartHours * 3600) + (eventStartMinutes * 60);
      let timeEnd = (eventEndHours * 3600) + (eventEndMinutes * 60);

      if (timeStart > timeEnd) {
        setStatus("Invalid event start and/or end times");
        return;
      } else {
        await invoke('insert_event', { eventDate: selectedDate, yearMonth: yearMonth, eventName: eventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd, color: randomColor });
        await getEvents();

        eventStartHours = 0;
        eventStartMinutes = 0;
        eventEndHours = 0;
        eventEndMinutes = 0;

        setStatus("Added event successfully");
      }

    } catch (error) {
      console.log("Error inserting event:", error);
      setStatus(`Failed to add event: ${error}`);
    }
  }

  async function updateEvent() {
    try {
      if (editEventNameInput?.value.trim() == '') {
        setStatus("Event name missing! Please add a name for the event");
        return;
      }

      let timeStart = (editEventStartHours * 3600) + (editEventStartMinutes * 60);
      let timeEnd = (editEventEndHours * 3600) + (editEventEndMinutes * 60);

      if (timeStart > timeEnd) {
        setStatus("Invalid event start and/or end times");
        return;
      } else {
        await invoke('update_event', { id: selectedEvent?.id, eventName: editEventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd });
        await getEvents();

        if (editEventNameInput) displayEventName = editEventNameInput?.value;

        setStatus("Event updated successfully");
      }

    } catch (error) {
      console.log("Error updating event:", error);
      setStatus(`Failed to update event: ${error}`);
    }
  }

  async function deleteEvent(eventId: number) {
    try {
      if (eventId !== null) {
        await invoke('delete_event', { id: eventId });
        await getEvents();
      }

      deleteEventId = null;
      setStatus(`Deleted event ${selectedEvent?.event_name} successfully`);
    } catch (error) {
      console.log("Error deleting event:", error);
      setStatus(`Failed to delete event: ${error}`);
    }
  }

  function startEdit(event: CalendarEvent) {
    selectedEvent = event;

    editEventStartHours = Math.floor(selectedEvent.event_start / 3600);
    editEventStartMinutes = Math.floor((selectedEvent.event_start % 3600) / 60);
    editEventEndHours = Math.floor(selectedEvent.event_end / 3600);
    editEventEndMinutes = Math.floor((selectedEvent.event_end % 3600) / 60);

    setStatus("Edit started");
  }

  function cancelEventUpdate() {
    selectedEvent = null;
    setStatus("Edit closed");
  }

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

{#if deleteEventId}
  <div id="deleteNotificationContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
    <div id="deleteNotificationContent">
      <div id="deleteNotificationMessage">
        <p>Are you sure you want to delete this event?</p>
        <p><strong>{events.find(e => e.id === deleteEventId)?.event_name}</strong></p>
      </div>
      <div style="display: flex; flex: 1; bottom-border: 1px solid #444;"></div>
      <div id="deleteNotificationButtons">
        <button onclick={() => { if (deleteEventId) deleteEvent(deleteEventId); }}>Confirm</button>
        <button onclick={() => { deleteEventId = null; }}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<div id="addEventOverlay">
  <div id="menuBar">
    <h2>Events on {selectedDateClean}</h2>
    <button onclick={() => { setSelectedDate(null); setStatus("Event view closed successfully"); }}>Close</button>
    <button onclick={() => { showAddEvent = true; }}>Add event</button>
  </div>
  <div id="mainContent">
    {#if showAddEvent || selectedEvent}
      <div id="addEditEventContainer" style="height: {height}px;">
        {#if showAddEvent}
          <div id="addEventContainer" role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); saveEvent(); } if (e.key === 'Escape') { e.preventDefault(); showAddEvent = false; setStatus("Event cancelled successfully"); }}}>
            <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
              <h3>Add event</h3>
              <div id="addEventInfo">
                <div id="addEventNameContainer">
                  <p>Event name</p>
                  <input bind:this={eventNameInput} />
                </div>
                <div class="eventStartEndContainer">
                  <div class="eventStartTimesContainer">
                    <p>Event start</p>
                    <div class="eventInputContainer">
                      <input type="number" bind:value={eventStartHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) eventStartHours = 23; if (value < 0) eventStartHours = 0; }} />
                      <input type="number" bind:value={eventStartMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) eventStartMinutes = 59; if (value < 0) eventStartMinutes = 0; }} />
                    </div>
                  </div>
                  <div class="eventEndTimesContainer">
                    <p>Event end</p>
                    <div class="eventInputContainer">
                      <input type="number" bind:value={eventEndHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) eventEndHours = 23; if (value < 0) eventEndHours = 0; }} />
                      <input type="number" bind:value={eventEndMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) eventEndMinutes = 59; if (value < 0) eventEndMinutes = 0; }} />
                    </div>
                  </div>
                </div>
              </div>
              <div class="eventFormButtons">
                <button onclick={saveEvent}>Add</button>
                <button onclick={() => { showAddEvent = false; setStatus("Event cancelled successfully"); }}>Cancel</button>
              </div>
            </OverlayScrollbarsComponent>
          </div>
        {/if}

        {#if selectedEvent}
          <div id="editEventContainer" class:moved={showAddEvent} role="button" tabindex="0" style="max-height: 348px;" onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); updateEvent(); } if (e.key === 'Escape') { e.preventDefault(); cancelEventUpdate(); }}}>
            <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
              <h3>
                Edit event:<br><span style="color: {selectedEvent.color}">{displayEventName}</span>
              </h3>
              <div id="editEventInfo">
                <div id="editEventNameContainer">
                  <p>Event name</p>
                  <input value={selectedEvent?.event_name} bind:this={editEventNameInput} />
                </div>
                <div class="eventStartEndContainer">
                  <div class="eventStartTimesContainer">
                    <p>Event start</p>
                    <div class="eventInputContainer">
                      <input type="number" bind:value={editEventStartHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) editEventStartHours = 23; if (value < 0) editEventStartHours = 0; }} />
                      <input type="number" bind:value={editEventStartMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) editEventStartMinutes = 59; if (value < 0) editEventStartMinutes = 0; }} />
                    </div>
                  </div>
                  <div class="eventEndTimesContainer">
                    <p>Event end</p>
                    <div class="eventInputContainer">
                      <input type="number" bind:value={editEventEndHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) editEventEndHours = 23; if (value < 0) editEventEndHours = 0; }} />
                      <input type="number" bind:value={editEventEndMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) editEventEndMinutes = 59; if (value < 0) editEventEndMinutes = 0; }} />
                    </div>
                  </div>
                </div>
              </div>
              <div class="eventFormButtons">
                <button onclick={updateEvent}>Save</button>
                <button onclick={cancelEventUpdate}>Close</button>
              </div>
            </OverlayScrollbarsComponent>
          </div>
        {/if}
      </div>
    {/if}

    <div id="timeline">
      <div id="timelineOverflow" style="width: 400%;">
        <div id="timeAxis">
          {#each hours as hour}
            <div class="tick" style="left: {(hour / 24) * 100}%;">{hour.toString().padStart(2, '0')}</div>
          {/each}
        </div>

        <div id="eventsBar" style="height: {maxLanes * 60}px;">
          <div class="gridlines"></div>
          {#each displayedEvents as event (event.id)}
            <div style="position: absolute; left: {getLeft(event.event_start)}%; top: {event.lane * 60}px; width: {getWidth(event.event_start, event.event_end)}%;">
              <div class="listedEventInfo" style="background: {event.color}; color: {brightColors.some(c => c === event.color) ? 'black' : '#f6f6f6'}; height: 50px; align-items: center;">
                <p>{secondsToHoursMinutes(event.event_start)}-{secondsToHoursMinutes(event.event_end)}</p>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <div id="eventsList">
      <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
        <div style="display: flex; flex-direction: column; gap: 20px; margin: 12px;">
          {#each events.filter(e => e.event_date === selectedDate) as event (event.id)}
            <div class="listedEvent">
              <div class="listedEventInfo" style="background: {event.color}; color: {brightColors.some(c => c === event.color) ? 'black' : '#f6f6f6'}">
                <div class="eventName">
                  <p class:sliding={event.event_name.length >= 12}>{event.event_name}</p>
                </div>
                <div class="spacer"></div>
                <p>{secondsToHoursMinutes(event.event_start)}-{secondsToHoursMinutes(event.event_end)}</p>
              </div>
              <div class="spacer"></div>
              <div class="listedEventControls">
                <button onclick={() => { startEdit(event); }}>Edit</button>
                <button onclick={() => { deleteEventId = event.id; }}>Delete</button>
              </div>
            </div>
          {/each}
        </div>
      </OverlayScrollbarsComponent>
    </div>
  </div>
</div>

<style>
  #addEventOverlay {
    position: fixed;
    display: flex;
    flex-direction: row;
    align-content: center;
    z-index: 10000;
    top: 0;
    left: 0;
    bottom: 20px;
    width: 100vw;
    height: 100vh - 20px;
    background: #0f0f0f;
  }

  #menuBar {
    position: fixed;
    display: flex;
    flex-direction: row;
    top: 0;
    left: 0;
    height: 70px;
    padding: 5px 20px;
    gap: 20px;
    border-bottom: 1px solid #444;
  }

  #menuBar h2 {
    margin: 0;
    max-width: 280px;
    width: fit-content;
  }

  #mainContent {
    position: fixed;
    display: flex;
    flex-direction: row;
    justify-content: flex-end;
    flex: 1;
    top: 70px;
    width: 100%;
    height: calc(100vh - 90px);
    padding: 10px;
    gap: 10px;
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

  #timeAxis {
    position: relative;
    display: flex;
    align-items: center;
    height: 30px;
    margin: 5px 15px;
    pointer-events: none;
    white-space: nowrap;
    border: 1px solid green;
  }

  #timeAxis .tick {
    position: absolute;
    transform: translateX(-50%);
    font-weight: 600;
  }

  #eventsBar {
    position: relative;
    margin: 5px 15px;
    border: 1px solid yellow;
  }

  #eventsBar .gridlines {
    position: absolute;
    inset: 0;
    background-image: linear-gradient(to right, rgba(255,255,255,0.1) 2px, transparent 1px);
    background-size: calc(100% / 48) 100%;
    pointer-events: none;
  }

  #timeline::-webkit-scrollbar {
    width: 6px;
    height: 6px;
    background: transparent;
    border-radius: 10px;
  }

  #timeline::-webkit-scrollbar-corner {
    background: transparent;
  }

  #timeline:hover::-webkit-scrollbar {
    background: #444;
  }

  #timeline::-webkit-scrollbar-thumb {
    border-radius: 10px;
    background: transparent;
  }

  #timeline:hover::-webkit-scrollbar-thumb {
    background: #888;
  }

  #timeline::-webkit-scrollbar-thumb:hover {
    background: #ccc;
  }

  #eventsList {
    display: flex;
    flex-direction: column;
    width: 15%;
    border-radius: 8px;
    background: #151515;
    padding: 2px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .listedEvent {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    max-height: 120px;
    padding: 10px;
    border-radius: 8px;
    background: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .listedEventInfo {
    display: flex;
    flex-direction: row;
    width: 100%;
    padding: 10px;
    border-radius: 8px;
  }

  .listedEventControls {
    display: flex;
    flex-direction: row;
    justify-content: right;
    gap: 10px;
    padding-top: 10px;
  }

  #eventsList p {
    margin: 0;
    font-weight: 800;
    font-size: 14px;
  }

  .eventName {
    display: flex;
    max-width: 50%;
    overflow: hidden;
  }

  .eventName p {
    max-width: 100%;
  }

  .eventName p.sliding:hover {
    animation: slideText 3.5s linear infinite;
  }

  #addEditEventContainer {
    position: fixed;
    display: flex;
    flex-direction: column;
    top: 90px;
    left: 20px;
    z-index: 10001;
    gap: 20px;
    padding: 20px;
    border-radius: 8px;
    background: #222;
  }

  #addEventContainer, #editEventContainer {
    z-index: 10002;
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    max-width: 500px;
    overflow: auto;
    background: #151515;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  #addEventContainer {
    max-height: 324px;
  }

  #editEventContainer {
    max-height: 348px;
  }

  #editEventContainer.moved {
    top: 434px;
  }

  .eventFormButtons {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    justify-content: flex-start;
    max-width: 200px;
    gap: 20px;
    padding: 10px;
    border-radius: 8px;
  }

  .eventFormButtons button, #menuBar button, .listedEventControls button {
    width: 80px;
    height: 30px;
    background: #222;
    border: 0;
    border-radius: 6px;
    color: #f6f6f6;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .eventFormButtons button:hover, #menuBar button:hover, .listedEventControls button:hover {
    cursor: pointer;
    background: #333;
  }

  #addEventInfo, #editEventInfo {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    height: 240px;
    gap: 20px;
    padding: 10px;
    border-radius: 8px;
  }

  #addEventNameContainer, #editEventNameContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    justify-content: center;
    max-height: 84px;
    background: #222;
    border-radius: 12px;
    padding: 10px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  #addEventNameContainer:hover, #editEventNameContainer:hover, .eventStartTimesContainer:hover, .eventEndTimesContainer:hover, .eventFormButtons button:hover, #menuBar button:hover, .listedEventControls button:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,1);
  }

  #addEventNameContainer input, #editEventNameContainer input {
    max-height: 40px;
    height: 100%;
    width: 100%;
    background: #151515;
    border-radius: 8px;
    padding: 2px 10px;
    color: #f6f6f6;
    font-size: 16px;
  }

  #addEventNameContainer input:focus, #editEventNameContainer input:focus, .eventStartEndContainer .eventInputContainer input:focus {
    border: 1px solid #723fffd0;
  }

  .eventStartEndContainer {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    justify-content: flex-start;
    gap: 20px;
  }

  .eventStartTimesContainer, .eventEndTimesContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    max-width: 150px;
    padding: 10px;
    gap: 5px;
    border-radius: 12px;
    background: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  #addEventInfo p, #editEventInfo p {
    margin: 0;
    text-align: left;
  }

  #addEventContainer h3, #editEventContainer h3 {
    margin: 5px;
  }

  .eventInputContainer {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    align-self: center;
    justify-content: center;
    align-items: center;
    padding: 10px;
    gap: 10px;
    border-radius: 8px;
    background: #151515;
  }

  .eventStartEndContainer .eventInputContainer input {
    margin: 0;
    max-height: 100vh;
    height: 100%;
    width: 100%;
    background: #222;
    border-radius: 6px;
    padding: 2px 10px;
    color: #f6f6f6;
    text-align: center;
    font-size: 20px;
    font-weight: 800;
  }

  .eventStartEndContainer .eventInputContainer input[type="number"]::-webkit-outer-spin-button, .eventStartEndContainer .eventInputContainer input[type="number"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  #deleteNotificationContainer {
    position: fixed;
    display: flex;
    right: 5px;
    bottom: 25px;
    z-index: 10005;
    max-width: 400px;
    width: 100%;
    max-height: 150px;
    height: 100%;
    background: #151515;
    border: 1px solid #444;
    border-radius: 8px;
  }

  #deleteNotificationContent {
    display: flex;
    flex: 1 1 0;
    flex-direction: column;
    padding: 10px;
  }

  #deleteNotificationButtons {
    display: flex;
    flex-direction: row;
    max-height: 30px;
    height: 100%;
    gap: 10px;
  }

  #deleteNotificationContent p {
    flex: 1 1 0;
    overflow-y: auto;
    text-align: left;
    white-space: pre-wrap;
    word-break: break-word;
    word-wrap: break-word;
    margin: 0;
  }

  #deleteNotificationButtons button {
    background-color: #222;
    color: #f6f6f6;
    font-size: 16px;
    border: none;
    border-radius: 6px;
    max-width: 50px;
    width: 100%;
    max-height: 26px;
    height: 100%;
    font-size: 14px;
    margin-top: 4px;
    padding: 2px 10px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  #deleteNotificationButtons button {
    max-width: 70px;
  }

  #deleteNotificationButtons button:hover {
    cursor: pointer;
    background: #333;
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0,0,0,1);
  }

  @keyframes slideText {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-300%);
    }
  }

  :global {
    .os-theme-dark {
      --os-handle-bg: #888;
      --os-handle-bg-hover: #ccc;
      --os-handle-bg-active: #ccc;
      --os-track-bg: #444;
    }
  }

</style>
