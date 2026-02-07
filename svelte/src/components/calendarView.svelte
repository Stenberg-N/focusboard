<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from '@tauri-apps/api/core';
  import type { CalendarDay, CalendarEvent } from "../types/types";

  const dayNames = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  let monthNames = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

  let now = new Date();
  let year = $state<number>(now.getFullYear());
  let month = $state<number>(now.getMonth());
  let currentDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  let selectedDate = $state<string | null>(null);
  let eventNameInput = $state<HTMLInputElement>();
  let eventStart = $state<HTMLInputElement>();
  let eventEnd = $state<HTMLInputElement>();

  let events = $state<CalendarEvent[]>([])

  let headers = $state<Array<string>>([]);
  let days = $state<CalendarDay[]>([]);

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

  async function saveEvent() {
    try {
      await invoke('insert_event', { eventDate: selectedDate, eventName: eventNameInput?.value, eventStart: Number(eventStart?.value), eventEnd: Number(eventEnd?.value) });
      await getEvents();
      selectedDate = null;

      setStatus("Added event successfully");

    } catch (error) {
      console.log("Error inserting event:", error);
      setStatus(`Failed to add event: ${error}`);
    }
  }

  async function getEvents() {
    try {
      const data = await invoke<CalendarEvent[]>('get_events');
      events = data;

    } catch (error) {
      console.log("Failed to retrieve events:", error);
      setStatus(`Failed to retrieve events: ${error}`);
    }
  }

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

  function next() {
    month++;

    if (month == 12) {
      year++;
      month = 0;
    }
  }

  function prev() {
    if (month == 0) {
      year--;
      month = 11;
    } else {
      month--;
    }
  }
</script>

{#if selectedDate}
  <div id="addEventOverlay">
    <div id="addEventContainer">
      <h3>Event info</h3>
      <div id="addEventInfo">
        <p>Event name</p>
        <input bind:this={eventNameInput} />
        <p>Event start</p>
        <input bind:this={eventStart} />
        <p>Event end</p>
        <input bind:this={eventEnd} />
      </div>
      <button onclick={saveEvent}>Save</button>
      <button onclick={() => { selectedDate = null }}>Close</button>
    </div>
  </div>
{/if}

<div id="calendarView">
  <div id="calendarControls">
    <button onclick={() => { prev(); initMonth(); }}>
      <img id="prevArrowIcon" src="down-arrow.svg" alt="prevArrow">
    </button>
    <p id="date">{monthNames[month]} {year}</p>
    <button onclick={() => { next(); initMonth(); }}>
      <img id="nextArrowIcon" src="down-arrow.svg" alt="nextArrow">
    </button>
  </div>
  <div id="calendar">
    <div id="weekDays">
      {#each headers as header}
        <div class="header">{header}</div>
      {/each}
    </div>
    <div id="calendarDays">
      {#each days as day (day.date)}
        <button class="dayContainer" class:nonCurrent={day.enabled == false} onclick={() => { selectedDate = day.isodate }}>
          <div class="dayInfo">
            <p class="monthAbbreviation">{day.monthabbrev}</p>
            <div class="dayNameContainer" class:currentDay={+day.date === +currentDate}>
              <p style="margin: 0;">{day.name}</p>
            </div>
          </div>
          {#if events}
            <div class="dayEvents">
              {#each events.filter(e => e.event_date === day.isodate) as event (event.id)}
                <div class="eventContainer">
                  <p>{event.event_name}</p>
                </div>
              {/each}
            </div>
          {/if}
        </button>
      {/each}
    </div>
  </div>
</div>

<style>
  #calendarView {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    min-height: 0;
    gap: 10px;
    width: calc(100vw - 85px);
    padding: 10px;
    overflow-y: auto;
    overflow-x: hidden;
    justify-content: center;
  }

  #calendar {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    background: transparent;
    padding: 5px 20px 20px;
  }

  #calendarControls {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    justify-content: center;
    align-items: center;
    max-height: 40px;
    margin: 0;
    margin-bottom: 10px;
    gap: 20px;
    font-size: 24px;
    font-weight: 800;
  }

  #date {
    width: 300px;
    margin: 0;
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
    text-align: center;
    font-weight: 800;
    color: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
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
    gap: 5px;
  }

  .dayContainer .dayEvents .eventContainer {
    display: flex;
    flex: 1 1 0;
    align-items: center;
    width: 80px;
    height: 20px;
    background: red;
    padding: 5px;
  }

  .dayContainer .dayEvents .eventContainer p {
    margin: 0;
    color: #f6f6f6;
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

  #addEventOverlay {
    position: fixed;
    display: flex;
    flex-direction: column;
    align-items: center;
    z-index: 10000;
    top: 0;
    left: 0;
    bottom: 20px;
    width: 100vw;
    height: 100vh - 20px;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(15px);
  }

  #addEventContainer {
    position: relative;
    inset: 0;
    width: 50%;
    height: 80%;
    margin: auto;
    overflow: auto;
    background: #0f0f0f;
    border-radius: 8px;
  }

  #addEventInfo {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    height: 275px;
    margin: 20px;
    background: #222;
    border-radius: 8px;
  }

  #addEventInfo p {
    margin: 0;
    padding: 20px 20px 5px;
    text-align: left;
  }

  #addEventInfo input {
    margin: 0 20px;
    max-height: 30px;
    height: 100%;
    max-width: 250px;
    width: 100%;
    background: #151515;
    border-radius: 8px;
    padding: 2px 10px;
    color: #f6f6f6;
  }
</style>
