<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import debounce from 'lodash/debounce';
  import VirtualList from '@sveltejs/svelte-virtual-list';
  import { getContext } from 'svelte';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import CalendarEventOverlay from "./CalendarEventOverlay.svelte";

  import type { CalendarDay, CalendarEvent } from "../types/types";
  import 'overlayscrollbars/overlayscrollbars.css';

  const dayNames = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  let monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
  const colors = ['#FF6384', '#36A2EB', '#4BC0C0', '#9966FF', '#FF9F40', '#AD60E0', '#76D7C4', '#F1948A', '#F7DC6F', '#E7E9ED', '#C9CBCF', '#FFCE56'];
  const brightColors = colors.slice(-4);

  let now = new Date();
  let year = $state<number>(now.getFullYear());
  let month = $state<number>(now.getMonth());
  let currentDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  let selectedDate = $state<string | null>(null);
  let selectedDateClean = $state<string | null>(null);
  let yearMonth = $derived.by(() => `${String(year)}-${String(Number(month+1))}`);

  let events = $state<CalendarEvent[]>([]);
  let eventsMap = $derived.by(() => {
    const map = new Map<string, CalendarEvent[]>();
    events.forEach(e => {
      if (!map.has(e.event_date)) map.set(e.event_date, []);
      map.get(e.event_date)?.push(e);
    });
    return map;
  });

  let headers = $state<Array<string>>([]);
  let days = $state<CalendarDay[]>([]);

  const setDeleteEventId = getContext<(id: number | null) => void>('setDeleteEventId');
  let deleteEventId = $state<number | null>(null);

  let selectedEvent = $state<CalendarEvent | null>(null);
  let showAddEvent = $state<boolean>(false);
  let displayEventName = $state<string | null>(null);
  let height = $state<number>(0);

  let editEventNameInput = $state<HTMLInputElement>();
  let editEventStartHours = $state<number>(0);
  let editEventStartMinutes = $state<number>(0);
  let editEventEndHours = $state<number>(0);
  let editEventEndMinutes = $state<number>(0);

  let eventNameInput = $state<HTMLInputElement>();
  let eventStartHours = $state<number>(0);
  let eventStartMinutes = $state<number>(0);
  let eventEndHours = $state<number>(0);
  let eventEndMinutes = $state<number>(0);

  let {
    setStatus,
  }: {
    setStatus: (msg: string) => void;
  } = $props();

  onMount(() => {
    getEvents();
    headers = dayNames;
    initMonth();

    setStatus("Calendar loaded successfully");
  });

  $effect(() => {
    if (selectedDate) {
      let day = new Date(selectedDate)
      selectedDateClean = `${monthNames[day.getMonth()].slice(0, 3)} ${day.getDate()}, ${day.getFullYear()}`
    }
  });

  $effect(() => {
    if (showAddEvent) height = 366;
    if (selectedEvent) height = 390;
    if (showAddEvent && selectedEvent) height = 736;
  });

  $effect(() => {
    if (selectedEvent !== null) {
      displayEventName = selectedEvent.event_name;
    }
  });

  const getEvents = debounce(async () =>{
    try {
      const data = await invoke<CalendarEvent[]>('get_events', { yearMonth: yearMonth });
      events = data;

    } catch (error) {
      console.log("Failed to retrieve events:", error);
      setStatus(`Failed to retrieve events: ${error}`);
    }
  }, 200);

  function initMonth() {
    days = [];

    let monthAbbreviation = monthNames[month].slice(0, 3);
    let nextMonthAbbreviation = monthNames[(month+1)%12].slice(0, 3);
    var firstDay = new Date(year, month, 1).getDay();
    let offset = firstDay === 0 ? 6 : firstDay - 1;
    var daysInCurrentMonth = new Date(year, month + 1, 0).getDate();
    var daysInLastMonth = new Date(year, month, 0).getDate();
    var previousMonth = month == 0 ? 11 : month - 1;

    for (let i = daysInLastMonth - offset; i < daysInLastMonth; i++) {
      let day = new Date(previousMonth == 11 ? year - 1 : year, previousMonth, i+1);

      days.push({ monthabbrev:'', name:'' + (i+1), enabled:false, date:day, isodate:'' });
    }

    for (let i = 0; i < daysInCurrentMonth; i++) {
      let day = new Date(year, month, i+1);
      let isoDate = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;

      if (i==0) days.push({ monthabbrev:monthAbbreviation, name:'' + (i+1), enabled:true, date:day, isodate:isoDate });
      else days.push({ monthabbrev:'', name:'' + (i+1), enabled:true, date:day, isodate:isoDate });
    }

    for (let i = 0; days.length%7; i++) {
      let day = new Date((month == 11 ? year + 1 : year), (month + 1)%12, i+1);

      if (i==0) days.push({ monthabbrev:nextMonthAbbreviation, name:'' + (i+1), enabled:false, date:day, isodate:'' });
      else days.push({ monthabbrev:'', name:'' + (i+1), enabled:false, date:day, isodate:'' });
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

  function cancelEventUpdate() {
    selectedEvent = null;
    setStatus("Edit closed");
  }

  async function deleteEvent(eventId: number) {
    try {
      if (eventId !== null) {
        await invoke('delete_event', { id: eventId });
        await getEvents();
      }

      deleteEventId = null;
      setDeleteEventId(null)
      setStatus(`Deleted event ${selectedEvent?.event_name} successfully`);
    } catch (error) {
      console.log("Error deleting event:", error);
      setStatus(`Failed to delete event: ${error}`);
    }
  }

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
        eventNameInput!.value = '';

        setStatus("Added event successfully");
      }

    } catch (error) {
      console.log("Error inserting event:", error);
      setStatus(`Failed to add event: ${error}`);
    }
  }

  async function nextMonth() {
    month++;

    if (month == 12) {
      year++;
      month = 0;
    }

    await getEvents();
    initMonth();
  }

  async function prevMonth() {
    if (month == 0) {
      year--;
      month = 11;
    } else {
      month--;
    }

    await getEvents();
    initMonth();
  }

  function nextDay() {
    if (!selectedDate) return;

    const day = new Date(selectedDate);
    const lastDay = new Date(day.getFullYear(), day.getMonth() + 1, 0).getDate();

    if ((day.getDate() + 1) > lastDay) {
      nextMonth();
      selectedDate = `${day.getFullYear()}-${String(day.getMonth() + 2).padStart(2, '0')}-01`;
    } else {
      selectedDate = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate() + 1).padStart(2, '0')}`;
    }
  }

  function prevDay() {
    if (!selectedDate) return;

    const day = new Date(selectedDate);
    const lastDayLastMonth = new Date(day.getFullYear(), day.getMonth(), 0).getDate();

    if ((day.getDate()) === 1) {
      prevMonth();
      selectedDate = `${day.getFullYear()}-${String(day.getMonth()).padStart(2, '0')}-${lastDayLastMonth}`
    } else {
      selectedDate = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate() - 1).padStart(2, '0')}`;
    }
  }

  function secondsToHoursMinutes(totalSeconds: number) {
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    return `${hours}:${String(minutes).padStart(2, '0')}`
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
        <button onclick={() => { deleteEventId = null; setDeleteEventId(null); }}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

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

<div id="eventList">
  <VirtualList items={selectedDate ? eventsMap.get(selectedDate) ?? [] : (Array.from(eventsMap.values()).flat() || [])} let:item>
    <div class="listedEvent">
      <div class="listedEventInfo" style="background: {item.color}; color: {brightColors.some(c => c === item.color) ? 'black' : '#f6f6f6'}">
        <div class="eventName">
          <p class:sliding={item.event_name.length >= 12}>{item.event_name}</p>
        </div>
        <div class="spacer"></div>
        <p>{secondsToHoursMinutes(item.event_start)}-{secondsToHoursMinutes(item.event_end)}</p>
      </div>
      <div class="spacer"></div>
      <div class="listedEventControls">
        <button onclick={() => { deleteEventId = item.id; setDeleteEventId(item.id); }}>Delete</button>
        <button onclick={() => {startEdit(item); }}>Edit</button>
      </div>
    </div>
  </VirtualList>
</div>

<div id="calendarView">
  <div id="calendarControls">
    <button onclick={() => selectedDate ? prevDay() : prevMonth() }>
      <img id="prevArrowIcon" src="down-arrow.svg" alt="prevArrow">
    </button>
    <button onclick={() => selectedDate ? nextDay() : nextMonth() }>
      <img id="nextArrowIcon" src="down-arrow.svg" alt="nextArrow">
    </button>
    {#if selectedDate}
      <p id="date">{selectedDateClean}</p>
    {:else}
      <p id="date">{monthNames[month]} {year}</p>
    {/if}
    {#if selectedDate}
      <button id="closeOverlay" style="color: white;" onclick={() => { selectedDate = null; showAddEvent = false; selectedEvent = null; }}>Close</button>
      <button id="addEvent" style="color: white;" onclick={() => { showAddEvent = true; }}>Add event</button>
    {/if}
  </div>

  {#if selectedDate}
    <CalendarEventOverlay {events} {brightColors} {selectedDate} secondsToHoursMinutes={secondsToHoursMinutes} startEdit={startEdit}/>
  {/if}

  <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
    <div id="calendar">
      <div id="weekDays">
        {#each headers as header}
          <div class="header">{header}</div>
        {/each}
      </div>
      <div id="calendarDays">
        {#each days as day (day.date)}
          <button class="dayContainer" class:nonCurrent={day.enabled == false} onclick={() => { if (!day.enabled) return; selectedDate = day.isodate; setStatus("Event view opened successfully"); }}>
            <div class="dayInfo">
              <p class="monthAbbreviation">{day.monthabbrev}</p>
              <div class="dayNameContainer" class:currentDay={+day.date === +currentDate}>
                <p style="margin: 0;">{day.name}</p>
              </div>
            </div>
            {#if events.length > 0}
              <div class="dayEvents">
                <VirtualList items={eventsMap.get(day.isodate) || []} let:item>
                  <div class="eventContainer" style="background: {item.color};">
                    <div class="eventName">
                      <p class:sliding={item.event_name.length >= 18} style="color: {brightColors.some(c => c === item.color) ? 'black' : '#f6f6f6'}">{item.event_name}</p>
                    </div>
                    <p style="color: {brightColors.some(c => c === item.color) ? 'black' : '#f6f6f6'}">{secondsToHoursMinutes(item.event_start)}-{secondsToHoursMinutes(item.event_end)}</p>
                  </div>
                </VirtualList>
              </div>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </OverlayScrollbarsComponent>
</div>

<style>
  #calendarView {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    min-height: 0;
    width: calc(100vw - 85px);
    overflow-y: auto;
    overflow-x: hidden;
  }

  #calendar {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    height: calc(100vh - 90px);
    background: transparent;
    padding: 12px;
  }

  #calendarControls {
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
    height: 70px;
    margin: 0;
    gap: 10px;
    padding-left: 10px;
    font-size: 24px;
    font-weight: 800;
    border-bottom: 1px solid #444;
  }

  #date {
    width: 300px;
    margin: 0;
    text-align: left;
  }

  #calendarControls button {
    display: flex;
    max-height: 40px;
    max-width: 40px;
    background: transparent;
    border: none;
    padding: 0;
    transition: transform 0.2s;
  }

  #calendarControls button#addEvent, #calendarControls button#closeOverlay {
    max-height: 30px;
    max-width: 80px;
    align-items: center;
    justify-content: center;
  }

  #calendarControls button:hover {
    cursor: pointer;
    transform: translateY(-4px);
  }

  #calendarControls button #nextArrowIcon, #calendarControls button #prevArrowIcon {
    align-self: center;
    max-height: 20px;
    max-width: 20px;
    filter: brightness(1) invert(0.7);
  }

  #calendarControls button #nextArrowIcon {
    rotate: -90deg;
  }

  #calendarControls button #prevArrowIcon {
    rotate: 90deg;
  }

  #weekDays {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    max-height: 30px;
    margin-bottom: 5px;
    gap: 10px;
    background: transparent;
  }

  .header {
    flex: 1 1 0;
    background: transparent;
    max-height: 30px;
    align-content: center;
    font-weight: 800;
  }

  #calendarDays {
    display: grid;
    grid-template-columns: repeat(7, minmax(50px, 1fr));
    grid-template-rows: repeat(6, minmax(50px, 1fr));
    flex: 1 1 0;
    gap: 10px;
  }

  .dayContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    justify-content: flex-start;
    min-width: calc(100% / 7 - 10px);
    min-height: calc(100% / 6 - 10px);
    background: #222;
    border: 0;
    border-radius: 8px;
    padding: 10px;
    gap: 10px;
    text-align: center;
    font-weight: 800;
    color: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s, background 0.2s;
  }

  .dayContainer .dayInfo {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-end;
  }

  .dayContainer .monthAbbreviation {
    margin: 0;
    margin-right: 5px;
    color: #f6f6f6;
  }

  .dayContainer:not(.nonCurrent):hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,1);
    cursor: pointer;
    background: #333;
  }

  .dayContainer.nonCurrent {
    cursor: not-allowed;
    transition: none;
    opacity: 0.5;
    background-image: repeating-linear-gradient(
      45deg,
      #0f0f0f 0,
      #0f0f0f 10px,
      transparent 0,
      transparent 50%
    );
    background-size: 30px 30px;
    background-attachment: fixed;
  }

  .dayContainer .dayEvents {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    overflow: auto;
  }

  :global(svelte-virtual-list-contents) {
    margin-right: 5px;
  }

  .dayContainer .dayEvents .eventContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    align-items: flex-start;
    max-width: 100%;
    height: 50px;
    margin-bottom: 5px;
    border-radius: 6px;
    padding: 5px;
  }

  .dayContainer .dayEvents .eventContainer p {
    margin: 0;
    color: #f6f6f6;
  }

  .eventName {
    display: flex;
    max-width: 100%;
    overflow: hidden;
  }

  .eventName p {
    max-width: 100%;
    font-size: 14px;
  }

  .eventName p.sliding:hover {
    animation: slideText 3s linear infinite;
  }

  .dayNameContainer {
    width: 30px;
    height: 30px;
    align-content: center;
    border-radius: 50%;
    background: #888;
  }

  .dayNameContainer.currentDay {
    background: #723fffd0;
  }

  @keyframes slideText {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-120%);
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

  #eventList {
    display: flex;
    flex-direction: column;
    width: 300px;
    background: transparent;
    border-right: 1px solid #444;
    padding: 16px 5px 1px 16px;
  }

  .listedEvent {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    max-height: 130px;
    padding: 10px;
    border-bottom: 1px solid #444;
    background: transparent;
  }

  .listedEventInfo {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 10px;
    border-radius: 8px;
  }

  .listedEventControls {
    display: flex;
    flex-direction: row;
    justify-content: left;
    gap: 10px;
    padding-top: 10px;
  }

  #eventList p {
    margin: 0;
    font-weight: 800;
    font-size: 14px;
  }

  #addEditEventContainer {
    position: fixed;
    display: flex;
    flex-direction: column;
    top: 80px;
    left: 395px;
    z-index: 10001;
    gap: 20px;
    padding: 20px;
    border-radius: 8px;
    border: 1px solid #444;
    background: #0f0f0f;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
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
    max-height: 326px;
  }

  #editEventContainer {
    max-height: 350px;
  }

  #editEventContainer.moved {
    top: 436px;
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

  .eventFormButtons button, .listedEventControls button, #calendarControls button#addEvent, #calendarControls button#closeOverlay {
    width: 80px;
    height: 30px;
    background: #222;
    border: 0;
    border-radius: 6px;
    color: #f6f6f6;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .eventFormButtons button:hover, .listedEventControls button:hover, #calendarControls button#addEvent:hover, #calendarControls button#closeOverlay:hover {
    cursor: pointer;
    background: #333;
    transform: translateY(-4px);
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

  #addEventNameContainer:hover, #editEventNameContainer:hover, .eventStartTimesContainer:hover, .eventEndTimesContainer:hover {
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

</style>
