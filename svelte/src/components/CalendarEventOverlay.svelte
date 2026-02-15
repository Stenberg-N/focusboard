<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';

  import type { CalendarEvent } from "../types/types";

  let {
    events,
    colors,
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

  let editEventNameInput = $state<HTMLInputElement>();
  let editEventStartHours = $state<number>(0);
  let editEventStartMinutes = $state<number>(0);
  let editEventEndHours = $state<number>(0);
  let editEventEndMinutes = $state<number>(0);

  let selectedEvent = $state<CalendarEvent | null>(null);

  async function saveEvent() {
    try {
      if (eventNameInput?.value.trim() == '') {
        setStatus("Event name missing! Please add a name for the event");
        return;
      }

      let randomColor = colors[Math.floor(Math.random() * colors.length)];
      let timeStart = (eventStartHours * 3600) + (eventStartMinutes * 60);
      let timeEnd = (eventEndHours * 3600) + (eventEndMinutes * 60);

      await invoke('insert_event', { eventDate: selectedDate, yearMonth: yearMonth, eventName: eventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd, color: randomColor });
      await getEvents();

      eventStartHours = 0;
      eventStartMinutes = 0;
      eventEndHours = 0;
      eventEndMinutes = 0;

      setStatus("Added event successfully");

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

      await invoke('update_event', { id: selectedEvent?.id, eventName: editEventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd });
      await getEvents();

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
</script>

<div id="addEventOverlay">
  <div id="menuBar">
    <h2>Events on {selectedDateClean}</h2>
    <button onclick={() => { setSelectedDate(null); setStatus("Event view closed successfully"); }}>Close</button>
  </div>
  <div id="mainContent">
    <div id="addEditEventContainer">
      <div id="addEventContainer">
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
                  <input bind:value={eventStartHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) eventStartHours = 23; if (value < 0) eventStartHours = 0; }} />
                  <input bind:value={eventStartMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) eventStartMinutes = 59; if (value < 0) eventStartMinutes = 0; }} />
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
            <button onclick={saveEvent}>Add event</button>
            <button onclick={() => { setSelectedDate(null); setStatus("Event view closed successfully"); }}>Cancel</button>
          </div>
        </OverlayScrollbarsComponent>
      </div>

      <div id="editEventContainer" style="max-height: 348px;">
        {#if !selectedEvent}
          <p style="margin: auto;">No currently selected event.</p>
        {:else}
          <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
            <h3>
              Edit event:<br><span style="color: {selectedEvent.color}">{selectedEvent.event_name}</span>
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
                    <input bind:value={editEventStartHours} min="0" max="23" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 23) editEventStartHours = 23; if (value < 0) editEventStartHours = 0; }} />
                    <input bind:value={editEventStartMinutes} min="0" max="59" oninput={(e) => { const target = e.target as HTMLInputElement; const value = Number(target.value); if (value > 59) editEventStartMinutes = 59; if (value < 0) editEventStartMinutes = 0; }} />
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
              <button onclick={cancelEventUpdate}>Cancel</button>
            </div>
          </OverlayScrollbarsComponent>
        {/if}
      </div>
    </div>

    <div id="eventsList">
      <div id="listedEventsContainer">
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <div style="display: flex; flex-direction: column; gap: 20px; margin: 20px;">
            {#each events.filter(e => e.event_date === selectedDate) as event (event.id)}
              <div class="listedEvent">
                <div class="listedEventInfo" style="background: {event.color}">
                  <div class="eventName">
                    <p class:sliding={event.event_name.length > 15}>{event.event_name}</p>
                  </div>
                  <div class="spacer"></div>
                  <p>{secondsToHoursMinutes(event.event_start)}-{secondsToHoursMinutes(event.event_end)}</p>
                </div>
                <div class="spacer"></div>
                <div class="listedEventControls">
                  <button onclick={() => { startEdit(event); }}>Edit</button>
                  <button onclick={() => { deleteEvent(event.id); }}>Delete</button>
                </div>
              </div>
            {/each}
          </div>
        </OverlayScrollbarsComponent>
      </div>
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
    flex: 1;
    top: 70px;
    width: 100%;
    height: calc(100vh - 90px);
    padding: 20px;
    gap: 20px;
  }

  #addEditEventContainer {
    display: flex;
    flex-direction: column;
    width: 40%;
    gap: 20px;
  }

  #eventsList {
    display: flex;
    flex-direction: column;
    width: 60%;
    border-radius: 8px;
    background: #151515;
    padding: 10px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  #listedEventsContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    gap: 20px;
    padding: 10px;
    background: #222;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .listedEvent {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    max-height: 65px;
    padding: 10px;
    border-radius: 8px;
    background: #151515;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .listedEventInfo {
    display: flex;
    flex-direction: row;
    max-width: 250px;
    width: 100%;
    padding: 10px;
    border-radius: 8px;
  }

  .listedEventControls {
    display: flex;
    flex-direction: row;
    gap: 10px;
    padding: 10px;
  }

  #eventsList p {
    margin: 0;
    font-weight: 800;
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

  #addEventContainer, #editEventContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    width: 100%;
    max-height: 324px;
    overflow: auto;
    background: #151515;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
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
    max-width: 250px;
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
