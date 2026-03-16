<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import debounce from 'lodash/debounce';
  import VirtualList from '@sveltejs/svelte-virtual-list';
  import { getContext } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import CalendarEventOverlay from "./CalendarEventOverlay.svelte";

  import type { CalendarDay, CalendarEvent } from "../types/types";
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/style.css';

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
  let eventToSave = $state<string | null>(null);

  let headers = $state<Array<string>>([]);
  let days = $state<CalendarDay[]>([]);

  const setDeleteEventId = getContext<(id: number | null) => void>('setDeleteEventId');
  let deleteEventId = $state<number | null>(null);

  let eventInEdit = $state<CalendarEvent | null>(null);
  let showAddEvent = $state<boolean>(false);
  let displayEventName = $state<string | null>(null);
  let height = $state<number>(0);
  let addEventInfoHeight = $state<number>(0);
  let addEventContainerHeight = $state<number>(0);
  let isOpen = $state<boolean>(false);

  let editEventNameInput = $state<HTMLInputElement | null>(null);
  let editEventStartHoursInput = $state<HTMLInputElement | null>(null);
  let editEventStartMinutesInput = $state<HTMLInputElement | null>(null);
  let editEventEndHoursInput = $state<HTMLInputElement | null>(null);
  let editEventEndMinutesInput = $state<HTMLInputElement | null>(null);

  let eventNameInput = $state<HTMLInputElement | null>(null);
  let addEventSelectedDay = $state<number>(1);
  let eventStartHoursInput = $state<HTMLInputElement | null>(null);
  let eventStartMinutesInput = $state<HTMLInputElement | null>(null);
  let eventEndHoursInput = $state<HTMLInputElement | null>(null);
  let eventEndMinutesInput = $state<HTMLInputElement | null>(null);

  let eventSearchInput = $state<HTMLInputElement | null>(null);
  let searchable = $state<string | null>(null);
  let isSearching = $state<boolean>(false);
  let foundEvents = $derived.by(() => { return events.filter(e => e.event_name.toLowerCase() === searchable?.trim().toLowerCase()) });

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
    if (showAddEvent && !selectedDate) height = 436; else height = 366;
    if (eventInEdit) height = 390;
    if (showAddEvent && eventInEdit && !selectedDate) height = 806; else if (showAddEvent && eventInEdit) height = 736;
  });

  $effect(() => {
    if (!selectedDate) addEventInfoHeight = 310, addEventContainerHeight = 396; else addEventInfoHeight = 240, addEventContainerHeight = 326;
  });

  $effect(() => {
    if (eventInEdit !== null) {
      displayEventName = eventInEdit.event_name;
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
    let firstDay = new Date(year, month, 1).getDay();
    let offset = firstDay === 0 ? 6 : firstDay - 1;
    let daysInCurrentMonth = new Date(year, month + 1, 0).getDate();
    let daysInLastMonth = new Date(year, month, 0).getDate();
    let previousMonth = month == 0 ? 11 : month - 1;

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
    eventInEdit = event;

    tick().then(() => {
      if (editEventStartHoursInput && editEventStartMinutesInput && editEventEndHoursInput && editEventEndMinutesInput) {
        editEventStartHoursInput.value = String(Math.floor(eventInEdit!.event_start / 3600));
        editEventStartMinutesInput.value = String(Math.floor((eventInEdit!.event_start % 3600) / 60));
        editEventEndHoursInput.value = String(Math.floor(eventInEdit!.event_end / 3600));
        editEventEndMinutesInput.value = String(Math.floor((eventInEdit!.event_end % 3600) / 60));
      }
      setStatus("Edit started");
    });
  }

  async function updateEvent() {
    try {
      if (editEventNameInput?.value.trim() == '') {
        setStatus("Event name missing! Please add a name for the event");
        return;
      }

      let timeStart = (Number(editEventStartHoursInput?.value) * 3600) + (Number(editEventStartMinutesInput?.value) * 60);
      let timeEnd = (Number(editEventEndHoursInput?.value) * 3600) + (Number(editEventEndMinutesInput?.value) * 60);

      if (timeStart > timeEnd) {
        setStatus("Invalid event start and/or end times");
        return;
      } else {
        await invoke('update_event', { id: eventInEdit?.id, eventName: editEventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd });
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
    eventInEdit = null;
    setStatus("Edit closed");
  }

  async function deleteEvent(eventId: number) {
    try {
      if (eventId !== null) {
        await invoke('delete_event', { id: eventId });
        await getEvents();
      }

      setStatus(`Deleted event ${events.find(e => e.id === deleteEventId)?.event_name} successfully`);
      deleteEventId = null;
      setDeleteEventId(null)
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
      let timeStart = (Number(eventStartHoursInput?.value) * 3600) + (Number(eventStartMinutesInput?.value) * 60);
      let timeEnd = (Number(eventEndHoursInput?.value) * 3600) + (Number(eventEndMinutesInput?.value) * 60);

      if (!selectedDate) {
        eventToSave = `${year}-${String(month + 1).padStart(2, '0')}-${String(addEventSelectedDay).padStart(2, '0')}`;
       } else {
        eventToSave = selectedDate;
      }

      if (timeStart > timeEnd) {
        setStatus("Invalid event start and/or end times");
        return;
      } else {
        if (eventStartHoursInput && eventStartMinutesInput && eventEndHoursInput && eventEndMinutesInput && eventNameInput) {
        await invoke('insert_event', { eventDate: eventToSave, yearMonth: yearMonth, eventName: eventNameInput?.value, eventStart: timeStart, eventEnd: timeEnd, color: randomColor });
        await getEvents();

        eventStartHoursInput.value = '';
        eventStartMinutesInput.value = '';
        eventEndHoursInput.value = '';
        eventEndMinutesInput.value = '';
        eventNameInput.value = '';
        addEventSelectedDay = 1;

        setStatus("Added event successfully");
        }
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

  function handleTimeInput(command: string, target: HTMLInputElement, value: string) {
    if (!target || !value) return;

    const sanitized = value.replace(/[^0-9]/g, '');
  
    if (!/^\d{0,2}$/.test(sanitized) || (sanitized === '')) { target.value = sanitized; return; }

    let num = Number(sanitized);
    let currentValue = num;

    switch(command) {
      case 'hours':
        currentValue = Math.max(0, Math.min(23, num));
        break;
      case 'minutes':
        currentValue = Math.max(0, Math.min(59, num));
        break;
      default:
        return;
    }

    if (sanitized !== currentValue.toString()) target.value = currentValue.toString();
  }

  function clickOutside(node: HTMLElement) {
    const handleClick = (event: MouseEvent) => {
      if (isOpen && node && !node.contains(event.target as Node)) {
        isOpen = false;
      }
    };
    document.addEventListener('click', handleClick, true);

    return {
      destroy() {
        document.removeEventListener('click', handleClick, true);
      }
    };
  }

  function searchEvents() {
    if (!eventSearchInput) return;
    isSearching = true;
    searchable = eventSearchInput.value;

    if (foundEvents.length <= 0) { setStatus('No matches found'); isSearching = false; }
    else if (foundEvents.length > 0) setStatus('Search completed');
  }

  function closeEventSearch() {
    if (!eventSearchInput) return;

    isSearching = false;
    searchable = null;
    eventSearchInput.value = '';
    foundEvents = [];

    setStatus("Search closed");
  }
</script>

{#if deleteEventId}
  <div class="notificationContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
    <div class="notificationContent">
      <div>
        <p>Are you sure you want to delete this event?</p>
        <p><strong>{events.find(e => e.id === deleteEventId)?.event_name}</strong></p>
      </div>
      <div class="spacer" style="border-bottom: 1px solid #444;"></div>
      <div class="notificationButtonContainer">
        <button class="primary-button" onclick={() => { if (deleteEventId) deleteEvent(deleteEventId); }}>Confirm</button>
        <button class="primary-button" onclick={() => { deleteEventId = null; setDeleteEventId(null); }}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

{#if showAddEvent || eventInEdit}
  <div id="addEditEventContainer" style="height: {height}px;">
    {#if showAddEvent}
      <div id="addEventContainer" style="max-height: {addEventContainerHeight}px;" role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); saveEvent(); } if (e.key === 'Escape') { e.preventDefault(); showAddEvent = false; setStatus("Event cancelled successfully"); }}}>
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <h3>Add event</h3>
          <div id="addEventInfo" style="height: {addEventInfoHeight}px;">
            <div id="addEventNameContainer">
              <p>Event name</p>
              <input bind:this={eventNameInput} />
            </div>
            {#if !selectedDate}
              <div id="addEventSelectDayContainer" class:noRaise={isOpen} use:clickOutside>
                <button id="selectedDay" class="primary-button" class:listOpen={isOpen} onclick={() => { isOpen = !isOpen }}>{addEventSelectedDay}</button>
                  {#if isOpen}
                    <div id="addEventSelectDayList" transition:slide={{ delay: 100, duration: 200, easing: cubicInOut }}>
                      <div style="overflow-y: auto; overflow-x: hidden;">
                        {#each Array.from({ length: new Date(year, month + 1, 0).getDate()}, (_, i) => i+1) as day}
                          <button class="dayOption primary-button" onclick={() => { addEventSelectedDay = day; isOpen = !isOpen; }}>{day}</button>
                        {/each}
                      </div>
                    </div>
                  {/if}
                <div style="height: 24px; align-self: center;">
                  /{month+1}/{year}
                </div>
              </div>
            {/if}
            <div class="eventStartEndContainer">
              <div class="eventStartTimesContainer" class:noRaise={isOpen}>
                <p>Event start</p>
                <div class="eventInputContainer">
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={eventStartHoursInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('hours', target, target.value); }} />
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={eventStartMinutesInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('minutes', target, target.value); }} />
                </div>
              </div>
              <div class="eventEndTimesContainer">
                <p>Event end</p>
                <div class="eventInputContainer">
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={eventEndHoursInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('hours', target, target.value); }} />
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={eventEndMinutesInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('minutes', target, target.value); }} />
                </div>
              </div>
            </div>
          </div>
          <div class="eventFormButtons">
            <button class="primary-button" onclick={saveEvent}>Add</button>
            <button class="primary-button" onclick={() => { showAddEvent = false; setStatus("Event cancelled successfully"); }}>Cancel</button>
          </div>
        </OverlayScrollbarsComponent>
      </div>
    {/if}

    {#if eventInEdit}
      <div id="editEventContainer" class:moved={showAddEvent} role="button" tabindex="0" style="max-height: 348px;" onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); updateEvent(); } if (e.key === 'Escape') { e.preventDefault(); cancelEventUpdate(); }}}>
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <h3>
            Edit event:<br><span style="color: {eventInEdit.color}">{displayEventName}</span>
          </h3>
          <div id="editEventInfo">
            <div id="editEventNameContainer">
              <p>Event name</p>
              <input value={eventInEdit?.event_name} bind:this={editEventNameInput} />
            </div>
            <div class="eventStartEndContainer">
              <div class="eventStartTimesContainer">
                <p>Event start</p>
                <div class="eventInputContainer">
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={editEventStartHoursInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('hours', target, target.value); }} />
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={editEventStartMinutesInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('minutes', target, target.value); }} />
                </div>
              </div>
              <div class="eventEndTimesContainer">
                <p>Event end</p>
                <div class="eventInputContainer">
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={editEventEndHoursInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('hours', target, target.value); }} />
                  <input type="text" inputmode="numeric" maxlength="2" placeholder="00" bind:this={editEventEndMinutesInput} oninput={(e) => { const target = e.target as HTMLInputElement; handleTimeInput('minutes', target, target.value); }} />
                </div>
              </div>
            </div>
          </div>
          <div class="eventFormButtons">
            <button class="primary-button" onclick={updateEvent}>Save</button>
            <button class="primary-button" onclick={cancelEventUpdate}>Close</button>
          </div>
        </OverlayScrollbarsComponent>
      </div>
    {/if}
  </div>
{/if}

<div id="eventList">
  <div id="eventSearchBar" style="position: relative; display: flex; flex-direction: row; height: 40px; width: 100%;">
    <button id="searchBarBtn" class="primary-button" onclick={() => searchEvents()}><img src="search.svg" alt="icon"></button>
      <input id="searchBar" bind:this={eventSearchInput} onkeydown={(e) => { if (e.key === 'Enter') { searchEvents(); } if (e.key === 'Escape') { closeEventSearch(); }}} />
      <button id="stopSearchBtn" class="primary-button" onclick={() => closeEventSearch()}><img src="close.svg" alt="icon"></button>
  </div>
  {#if eventsMap.size <= 0}
    <span style="font-size: 18px; font-weight: bold; user-select: none;">No events yet</span>
  {:else}
    <VirtualList items={searchable && foundEvents.length > 0 ? foundEvents : (selectedDate ? eventsMap.get(selectedDate) ?? [] : (Array.from(eventsMap.values()).flat() || []))} let:item>
      <div class="listedEvent">
        <div class="listedEventInfo" style="background-color: {item.color}; color: {brightColors.some(c => c === item.color) ? 'black' : '#f6f6f6'}">
          <div class="eventName">
            <p class:sliding={item.event_name.length >= 12}>{item.event_name}</p>
          </div>
          <div class="spacer"></div>
          <p>{secondsToHoursMinutes(item.event_start)}-{secondsToHoursMinutes(item.event_end)}</p>
        </div>
        <div class="spacer"></div>
        <div class="listedEventControls">
          <button class="primary-button" onclick={() => { deleteEventId = item.id; setDeleteEventId(item.id); }}>Delete</button>
          <button class="primary-button" onclick={() => {startEdit(item); }}>Edit</button>
        </div>
      </div>
    </VirtualList>
  {/if}
</div>

<div id="calendarView">
  <div id="calendarControls">
    <button class="primary-button" style="background-color: transparent;" onclick={() => selectedDate ? prevDay() : prevMonth() }>
      <img id="prevArrowIcon" src="arrow.svg" alt="prevArrow">
    </button>
    <button class="primary-button" style="background-color: transparent;" onclick={() => selectedDate ? nextDay() : nextMonth() }>
      <img id="nextArrowIcon" src="arrow.svg" alt="nextArrow">
    </button>
    {#if selectedDate}
      <p id="date">{selectedDateClean}</p>
    {:else}
      <p id="date">{monthNames[month]} {year}</p>
    {/if}
    <button id="addEvent" class="primary-button" onclick={() => { showAddEvent = true; }}>Add event</button>
    {#if selectedDate}
      <button id="closeOverlay" class="primary-button" onclick={() => { selectedDate = null; showAddEvent = false; eventInEdit = null; }}>Close</button>
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
          <button class="dayContainer primary-button" class:nonCurrent={day.enabled == false} onclick={() => { if (!day.enabled) return; selectedDate = day.isodate; setStatus("Event view opened successfully"); }}>
            <div class="dayInfo">
              <p class="monthAbbreviation">{day.monthabbrev}</p>
              <div class="dayNameContainer" class:currentDay={+day.date === +currentDate}>
                <p style="margin: 0;">{day.name}</p>
              </div>
            </div>
            {#if events.length > 0}
              <div class="dayEvents">
                <VirtualList items={eventsMap.get(day.isodate) || []} let:item>
                  <div class="eventContainer" style="background-color: {item.color};">
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
    background-color: transparent;
    padding: 12px;
    user-select: none;
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
    user-select: none;
  }

  #date {
    width: 300px;
    margin: 0;
    text-align: left;
  }

  #calendarControls button {
    height: 20px;
    width: 20px;
    padding: 0;
    box-shadow: unset;
    transition: transform 0.2s;
    user-select: none;
  }

  #calendarControls button#addEvent, #calendarControls button#closeOverlay {
    max-height: 30px;
    max-width: 80px;
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
    background-color: transparent;
  }

  .header {
    flex: 1 1 0;
    background-color: transparent;
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
    flex-direction: column;
    flex: 1 1 0;
    justify-content: flex-start;
    align-items: unset;
    min-width: calc(100% / 7 - 10px);
    min-height: calc(100% / 6 - 10px);
    max-width: unset;
    max-height: unset;
    padding: 10px;
    gap: 10px;
    text-align: center;
    font-weight: 800;
    color: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s, background-color 0.2s;
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

  .dayContainer.nonCurrent {
    cursor: not-allowed;
    transform: none;
    background-color: #222;
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
    background-color: #888;
  }

  .dayNameContainer.currentDay {
    background-color: #723fffd0;
  }

  @keyframes slideText {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-120%);
    }
  }

  #eventList {
    display: flex;
    flex-direction: column;
    width: 300px;
    background-color: transparent;
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
    background-color: transparent;
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
    background-color: #0f0f0f;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    user-select: none;
  }

  #addEventContainer, #editEventContainer {
    z-index: 10002;
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    max-width: 500px;
    overflow: auto;
    background-color: #151515;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
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
    border-radius: 6px;
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
    background-color: #222;
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
    background-color: #151515;
    border-radius: 8px;
    padding: 2px 10px;
    color: #f6f6f6;
    font-size: 16px;
    border: none;
    outline: none;
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
    background-color: #222;
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
    background-color: #151515;
  }

  .eventStartEndContainer .eventInputContainer input {
    margin: 0;
    max-height: 100vh;
    height: 100%;
    width: 100%;
    background-color: #222;
    border: none;
    outline: none;
    border-radius: 6px;
    padding: 2px 10px;
    color: #f6f6f6;
    text-align: center;
    font-size: 20px;
    font-weight: 800;
  }

  .eventStartEndContainer .eventInputContainer input[type="text"]::-webkit-outer-spin-button, .eventStartEndContainer .eventInputContainer input[type="text"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  #addEventSelectDayContainer {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    align-items: top;
    max-height: 50px;
    height: 100%;
    padding: 10px;
    border-radius: 8px;
    background-color: #222;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
    transition: transform 0.2s, box-shadow 0.2s;
    font-size: 16px;
  }

  #addEventSelectDayContainer:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0,0,0,0.8);
  }

  #addEventSelectDayContainer #selectedDay {
    will-change: transition animation;
    align-self: center;
    width: 56px;
    height: 24px;
    margin: 0 5px;
    background-color: #151515;
    transition: border-radius 400ms cubic-bezier(0.645, 0.045, 0.355, 1.000);
    box-shadow: none;
  }

  #addEventSelectDayContainer #selectedDay:hover {
    transform: none;
    box-shadow: none;
  }

  #addEventSelectDayContainer #selectedDay.listOpen {
    animation: openList 0.4s cubic-bezier(0.645, 0.045, 0.355, 1.000);
    border-radius: 8px 8px 0 0;
  }

  #addEventSelectDayList {
    background-color: #151515;
    display: flex;
    flex-direction: column;
    flex: 1;
    position: fixed;
    align-items: center;
    z-index: 10003;
    color: #f6f6f6;
    width: 56px;
    height: 180px;
    overflow: hidden;
    border-radius: 0 0 8px 8px;
    transform: translateX(5px) translateY(24px);
    scrollbar-gutter: stable;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .eventStartTimesContainer.noRaise, #addEventSelectDayContainer.noRaise {
    transform: none;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .dayOption {
    height: 30px;
    width: 40px;
    margin: 5px 5px 5px 0;
    border-radius: 0 8px 8px 0;
  }

  .dayOption:hover {
    transform: translateY(-2px);
  }

  #searchBar {
    width: 100%;
    margin-right: 11px;
    outline: none;
    border: none;
    background-color: #222;
    border-radius: 0 20px 20px 0;
    border: 1px solid #444;
    color: #f6f6f6;
    font-size: 16px;
    padding-left: 5px;
  }

  #searchBar:focus {
    border-color: #723fffd0;
  }

  #searchBarBtn {
    max-width: 40px;
    border-radius: 20px 0 0 20px;
    border: 1px solid #444;
    border-right: none;
  }

  #searchBarBtn img {
    width: 20px;
    height: 20px;
    filter: brightness(0) invert(0.7);
  }

  #searchBarBtn:hover {
    transform: unset;
  }

  #stopSearchBtn {
    position: absolute;
    top: 50%;
    right: 25px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 10px;
    height: 10px;
    background-color: transparent;
    box-shadow: none;
    transform: translateY(-50%); 
  }

  #stopSearchBtn img {
    position: relative;
    width: 10px;
    height: 10px;
    filter: brightness(0) invert(0.7);
  }

  #stopSearchBtn img:hover {
    filter: brightness(0) invert(0.9);
  }

  @keyframes openList {
    0% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(5px);
    }
    100% {
      transform: translateY(0);
    }
  }

</style>
