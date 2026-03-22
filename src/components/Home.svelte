<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { Note, CalendarEvent } from '../types/types';
  import { onMount } from 'svelte';

  let {
    setStatus,
  }: {
    setStatus: (msg: string) => void;
  } = $props();

  const monthAbbreviations = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  const today = new Date();
  const yearMonth = $derived.by(() => `${String(today.getFullYear())}-${String(today.getMonth() + 1).padStart(2, '0')}`);
  let upcomingEventsRange = $state<number>(7);

  let calendarEvents = $state<CalendarEvent[]>([]);
  let nextWeekDays = $state<string[]>([]);
  const nextWeekEvents = $derived.by(() => { return nextWeekDays.flatMap((day) => calendarEvents.filter(e => e.event_date === day)); });
  const eventDatesMap = $derived.by(() => {
    const map = new Map<string, string[]>();

    nextWeekEvents.forEach(e => {
      const date = new Date(e.event_date);
      const year = date.getFullYear();
      const month = monthAbbreviations[date.getMonth()];
      const day = date.getDate();

      const formattedDate = `${month} ${day}, ${year}`;

      if (!map.has(e.event_date)) map.set(e.event_date, []);

      const array = map.get(e.event_date);
      if (array && !array.includes(formattedDate)) array.push(formattedDate);
    });

    return map;
  });

  let recentNotes = $state<Note[]>([]);
  
  onMount(() => {
    getCalendarEvents();
    getRecentNotes();
  })

  async function getRecentNotes() {
    try {
      const data = await invoke<Note[]>('get_notes');
      recentNotes = data.sort((a, b) => +new Date(b.updated_at) - +new Date(a.updated_at)).slice(0, 4);

    } catch (error) {
      console.log("Failed to fetch recent notes:", error);
      setStatus(`Failed to fetch recent notes: ${error}`);
    }
  }

  async function getCalendarEvents() {
    try {
      const data = await invoke<CalendarEvent[]>('get_events', { yearMonth: yearMonth, onwards: true });
      calendarEvents = data;

      calcNextWeekDays();

    } catch (error) {
      console.log("Failed to fetch calendar events:", error);
      setStatus(`Failed to fetch calendar events: ${error}`);
    }
  }

  function calcNextWeekDays() {
    let comingDays = [];
    
    for (let i = 0; i <= upcomingEventsRange; i++) {
      const nextDay = new Date(today);
      nextDay.setDate(today.getDate() + i);

      const year = nextDay.getFullYear();
      const month = String(nextDay.getMonth() + 1).padStart(2, '0');
      const day = String(nextDay.getDate()).padStart(2, '0');

      comingDays.push(`${year}-${month}-${day}`);
    }
    nextWeekDays = comingDays;
  }

  function secondsToHoursMinutes(value: number) {
    const hours = Math.floor(value / 3600);
    const minutes = Math.floor((value % 3600) / 60);
    return `${hours}:${String(minutes).padStart(2, '0')}`;
  }

  function stripHtml(html: string | undefined): string {
    if (!html) return '';
    const doc = new DOMParser().parseFromString(html, 'text/html');
    return doc.body.textContent?.trim();
  }
</script>

<div id="home">
  <div id="upcomingEventsContainer">
    <h2 style="align-self: flex-start; margin-left: 14px;">Upcoming Events</h2>
    <div style="display: flex; flex-direction: column; align-self: flex-start; align-items: center; margin: 0 0 18px 14px;">
      <span style="font-size: 11px;">Day range</span>
      <input id="eventsRangeInput" bind:value={upcomingEventsRange} onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); calcNextWeekDays(); }}} />
    </div>
    <div id="upcomingEventsList">
      {#each nextWeekEvents as event (event.id)}
        <div class="upcomingEvent">
          <div style="display: flex; flex-direction: column; align-items: flex-start; width: 100%; overflow: hidden;">
            <span class:sliding={event.event_name.length > 50} style="color: {event.color};">{event.event_name}</span>
            <div style="display: flex; justify-content: space-between; width: 100%;">
              <span>{secondsToHoursMinutes(event.event_start)} - {secondsToHoursMinutes(event.event_end)}</span>
              <span>{eventDatesMap.get(event.event_date)}</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
  <div id="recentNotesContainer">
    <h2 style="align-self: flex-start; margin-left: 14px;">Latest edited notes</h2>
    <div id="recentNotesList">
      {#each recentNotes as note (note.id)}
        <div class="recentNote">
          <div style="border-bottom: 1px solid #444;"><h3 style="margin: 12px 0;">{stripHtml(note.title)}</h3></div>
          <div id="recentNoteContentOuter">
            <div id="recentNoteContent">
              <p style="margin: 0;">{stripHtml(note.content)}</p>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  #home {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    margin: 5rem;
    gap: 20px;
    padding: 10px;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  #home h2 {
    user-select: none;
  }

  #eventsRangeInput {
    max-width: 40px;
    outline: none;
    border: 1px solid #444;
    background-color: transparent;
    color: #f6f6f6;
    padding: 4px;
  }

  #upcomingEventsContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    justify-content: flex-start;
    align-items: center;
    max-width: 35%;
    max-height: 100%;
    padding: 6px;
    background-color: #151515;
    border-radius: 12px;
  }

  #upcomingEventsList {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    align-items: center;
    width: 100%;
    height: 100%;
    gap: 12px;
    border-top: 1px solid #444;
    padding: 14px 8px 14px 14px;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-gutter: stable;
  }

  .upcomingEvent {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 60px;
    padding: 6px 12px;
    background-color: #222;
    border-radius: 8px;
    overflow-x: hidden;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  .upcomingEvent span {
    font-weight: 600;
    max-width: 100%;
  }

  :hover.sliding {
    animation: slideText 3s linear infinite;
  }

  @keyframes slideText {
    0% {
      transform: translateX(0);
    }
    100% {
      transform: translateX(-120%);
    }
  }

  #recentNotesContainer {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    justify-content: flex-start;
    align-items: center;
    max-width: 65%;
    max-height: 100%;
    border-radius: 12px;
    background-color: #151515;
    padding: 4px;
  }

  #recentNotesList {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    width: 100%;
    height: 100%;
    gap: 18px;
    padding: 18px 8px 14px 14px;
    border-top: 1px solid #444;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  .recentNote {
    display: flex;
    flex-direction: column;
    width: calc(50% - 9px);
    height: calc(50% - 9px);
    border-radius: 8px;
    background-color: #222;
    padding: 18px 4px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  }

  #recentNoteContentOuter {
    height: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable;
    padding: 0 12px 0 18px;
    margin: 12px 0;
  }

  #recentNoteContent {
    height: 100%;
    text-align: left;
    overflow-y: auto;
    padding: 2px;
    border: 1px solid #444;
    border-radius: 8px;
  }

  @media (max-width: 1100px) {
    #home {
      flex-direction: column;
    }

    #recentNotesContainer, #upcomingEventsContainer {
      max-width: 100%;
    }

    .recentNote {
      width: 100%;
      height: 100%;
    }
  }

</style>