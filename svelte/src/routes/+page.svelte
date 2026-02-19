<script lang="ts">
  import { onMount, setContext } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import type { Store } from '@tauri-apps/plugin-store';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import LoaderOverlay from '../components/loaderOverlay.svelte';
  import NotesView from '../components/notesView.svelte';
  import TimerView from '../components/timerView.svelte';
  import CalendarView from '../components/calendarView.svelte';

  import './app.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  setContext('setDeleteEventId', setDeleteEventId);
  setContext('deleteNoteContext', { getDeleteNoteId: () => deleteNoteId, setDeleteNoteId });

  let uiVisibility = $state({ runningTimer: true, });
  let noteOpenStates = $state<Record<number, boolean>>({});
  let currentTabId = $state<number | null>(null);
  let currentTabName = $state<string | null>(null);

  let store = $state<Store | null>(null);
  let currentView = $state<'timerView' | 'calendarView' | 'notesView' | 'Home'>('Home');
  let statusBar = $state<HTMLSpanElement>()!;
  let deleteNoteId = $state<number | null>(null);
  let deleteEventId = $state<number | null>(null);
  let timerNotifBottom = $state<number>(25);

  let displayMinutes = $state(0);
  let displaySeconds = $state(0);
  let isRunningTimerFinished = $state(false);
  let setTimerMessage = $state('');

  onMount(() => {
    void (async () => {
      statusBar.textContent = "App setup complete";

      updateFromTimer();
      const currentlyRunningTimer = setInterval(updateFromTimer, 1000);
      return () => clearInterval(currentlyRunningTimer);
    })();
  });

  let showOverlay = $state<boolean>(false);

  listen('app-closing', async () => {
    if (store) {
      store.set('currentTabId', currentTabId);
      store.set('currentTabName', currentTabName);
      store.set('noteOpenStates', noteOpenStates);
      store.save();
    }
    showOverlay = true;
  });

  $effect(() => {
    if (currentView === 'notesView') {
      if (deleteNoteId) timerNotifBottom = 210;
      else timerNotifBottom = 55;
    } else {
      if (deleteEventId) timerNotifBottom = 180;
      else timerNotifBottom = 25;
    }
  })

  function updateFromTimer() {
    const stored = localStorage.getItem('runningTimer');
    if (!stored) {
      displayMinutes = 0;
      displaySeconds = 0;
      return;
    }

    const { endAt, isRunning, setMessage } = JSON.parse(stored);
    if (!isRunning) return;

    const remainingTime = Math.ceil((endAt - Date.now()) / 1000);

    if (remainingTime <= 1) {
      setTimeout(() => {
        isRunningTimerFinished = true;
      }, 1000);
    }

    displayMinutes = Math.floor(remainingTime / 60);
    displaySeconds = remainingTime % 60;
    setTimerMessage = setMessage;
  }

  let runningTimerIsOpen = $derived(uiVisibility.runningTimer);

  function runningTimerToggle() {
    uiVisibility.runningTimer = !uiVisibility.runningTimer;
  }

  function setCurrentTabId(id: number | null) {
    currentTabId = id;
  }

  function setCurrentTabName(name: string | null) {
    currentTabName = name;
  }

  function setDeleteNoteId(id: number | null) {
    deleteNoteId = id;
  }

  function setDeleteEventId(id: number | null) {
    deleteEventId = id;
  }

  function setStore(s: Store) {
    store = s;
  }

</script>

<main class="container">
  {#if showOverlay}
    <LoaderOverlay />
  {/if}

  {#if isRunningTimerFinished}
    <div id="mainViewTimerFinishedContainer" style="bottom: {timerNotifBottom}px;" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
      <div id="mainViewTimerNotificationContainer">
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <div id="mainViewTimerMessageContainer">
            <p>{setTimerMessage || 'Timer complete!'}</p>
          </div>
        </OverlayScrollbarsComponent>
        <div class="mainViewTimerSpacer"></div>
        <button onclick={() => isRunningTimerFinished = false}>OK</button>
      </div>
    </div>
  {/if}

  <div style="position: fixed; inset: 0; overflow: hidden;">
    <div id="navigationBar">
      <button class:selected={currentView === 'Home'} onclick={() => { currentView = 'Home'; statusBar.textContent = "Home loaded successfully" }}>Home</button>
      <button id="noteViewBtn" class:selected={currentView === 'notesView'} onclick={() => currentView = 'notesView'}>
        <img id="noteIcon" src="note.svg" alt="noteIcon">
      </button>
      <button id="timerViewBtn" class:selected={currentView === 'timerView'} onclick={() => currentView = 'timerView'}>
        <img id="clockIcon" src="clock.svg" alt="clockIcon">
      </button>
      <button id="calendarViewBtn" class:selected={currentView === 'calendarView'} onclick={() => currentView = 'calendarView'}>
        <img id="calendarIcon" src="calendar.svg" alt="calendarIcon">
      </button>
    </div>

    <div id="Content" style="position: fixed; bottom: 20px; left: 85px; width: calc(100vw - 85px); height: calc(100% - 20px); border-left: 1px solid #444; display: flex;">
      {#if currentView === 'timerView'}
        <TimerView setStatus={(msg) => (statusBar.textContent = msg)} />
      {:else if currentView === 'calendarView'}
        <CalendarView setStatus={(msg) => (statusBar.textContent = msg)} />
      {:else if currentView === 'notesView'}
        <NotesView setStore={setStore} {store} setCurrentTabId={setCurrentTabId} {currentTabId} setCurrentTabName={setCurrentTabName} {currentTabName} {noteOpenStates} setStatus={(msg) => (statusBar.textContent = msg)} />
      {:else}
        <div>Home</div>
      {/if}
    </div>

    <div id="statusBar">
      <span bind:this={statusBar}></span>
    </div>

    <div id="runningTimerContainer">
      <button id="runningTimerToggle" class:closed={!runningTimerIsOpen} onclick={runningTimerToggle}>
          <img id="runningTimerArrow" class:pointleft={!runningTimerIsOpen} src="down-arrow.svg" alt="runningTimerToggleIcon">
      </button>
      {#if runningTimerIsOpen}
        <div id="timesContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
          <div id="runningDisplayMinutes">
            <p>{displayMinutes.toString().padStart(2, '0')}</p>
          </div>
          <div id="runningDisplaySeconds">
            <p>{displaySeconds.toString().padStart(2, '0')}</p>
          </div>
        </div>
      {/if}
    </div>
  </div>
</main>

<style>

:global {
  .os-theme-dark {
    --os-handle-bg: #888;
    --os-handle-bg-hover: #ccc;
    --os-handle-bg-active: #ccc;
    --os-track-bg: #444;
  }
}

</style>
