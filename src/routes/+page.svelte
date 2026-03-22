<script lang="ts">
  import { onMount, setContext } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import type { Store } from '@tauri-apps/plugin-store';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import Home from '../components/Home.svelte';
  import LoaderOverlay from '../components/loaderOverlay.svelte';
  import NotesView from '../components/notesView.svelte';
  import TimerView from '../components/timerView.svelte';
  import CalendarView from '../components/calendarView.svelte';

  import './style.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  setContext('setDeleteEventId', setDeleteEventId);
  setContext('deleteNoteContext', { getDeleteNoteId: () => deleteNoteId, setDeleteNoteId });

  const views = [
    { name: 'Home', src: 'home.svg' },
    { name: 'notesView', src: 'note.svg' },
    { name: 'timerView', src: 'clock.svg' },
    { name: 'calendarView', src: 'calendar.svg' },
  ];

  let uiVisibility = $state({ runningTimer: true, });
  let currentTabId = $state<number | null>(null);
  let currentTabName = $state<string | null>(null);

  let store = $state<Store | null>(null);
  let currentView = $state<string | 'Home'>('Home');
  let statusBar = $state<HTMLSpanElement>()!;
  let deleteNoteId = $state<number | null>(null);
  let deleteEventId = $state<number | null>(null);
  let timerNotifBottom = $state<number>(25);
  let noteHeightMultiplier = $state<"larger" | "smaller" | null>(null);
  let noteColumns = $state<number | null>(null);
  let noteGap = $state<number | null>(null);
  let runningTimerIsOpen = $derived(uiVisibility.runningTimer);

  let displayMinutes = $state(0);
  let displaySeconds = $state(0);
  let isRunningTimerFinished = $state(false);
  let setTimerMessage = $state('');

  // Helper/wrapper functions

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
      store.set('noteHeightMultiplier', noteHeightMultiplier);
      store.set('noteColumns', noteColumns);
      store.set('noteGap', noteGap);
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

</script>

{#if showOverlay}
  <LoaderOverlay />
{/if}

{#if isRunningTimerFinished}
  <div class="notificationContainer" style="bottom: {timerNotifBottom}px;" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
    <div class="notificationContent">
      <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
        <div style="padding-right: 10px;">
          <p>{setTimerMessage || 'Timer complete!'}</p>
        </div>
      </OverlayScrollbarsComponent>
      <div class="spacer" style="border-bottom: 1px solid #444;"></div>
      <div class="notificationButtonContainer">
        <button class="primary-button" onclick={() => isRunningTimerFinished = false}>OK</button>
      </div>
    </div>
  </div>
{/if}

<div style="position: fixed; inset: 0; overflow: hidden;">
  <div id="navigationBar">
    {#each views as view}
      <button class="primary-button" class:selected={currentView === view.name} onclick={() => currentView = view.name}>
        <img class="navigationIcons" src={view.src} alt="icon">
      </button>
    {/each}
  </div>

  <div id="Content">
    {#if currentView === 'timerView'}
      <TimerView setStatus={(msg) => (statusBar.textContent = msg)} />
    {:else if currentView === 'calendarView'}
      <CalendarView setStatus={(msg) => (statusBar.textContent = msg)} />
    {:else if currentView === 'notesView'}
      <NotesView setStore={setStore} {store} setCurrentTabId={setCurrentTabId} {currentTabId} setCurrentTabName={setCurrentTabName} {currentTabName} setStatus={(msg) => (statusBar.textContent = msg)} bind:noteHeightMultiplier bind:noteColumns bind:noteGap />
    {:else}
      <Home setStatus={(msg) => (statusBar.textContent = msg)}/>
    {/if}
  </div>

  <div id="statusBar">
    <span bind:this={statusBar}></span>
  </div>

  <div id="runningTimerContainer">
    <button id="runningTimerToggle" class:closed={!runningTimerIsOpen} onclick={runningTimerToggle}>
        <img id="runningTimerArrow" class:pointleft={!runningTimerIsOpen} src="arrow.svg" alt="runningTimerToggleIcon">
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

<style>

#navigationBar {
  position: fixed;
  left: 0;
  top: 0;
  bottom: 20px;
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  max-width: 85px;
  width: 100%;
  gap: 10px;
  padding: 15px;
}

#navigationBar button {
  max-width: 55px;
  max-height: 55px;
  padding: 0;
}

#navigationBar button.selected {
  border: 1px solid #723fffd0;
}

#Content {
  position: fixed;
  bottom: 20px;
  left: 85px;
  display: flex;
  width: calc(100vw - 85px);
  height: calc(100% - 20px);
  border-left: 1px solid #444;
}

#statusBar {
  position: fixed;
  display: flex;
  flex-direction: row;
  bottom: 0;
  left: 0;
  right: 0;
  justify-content: flex-start;
  width: 100%;
  height: 20px;
  align-items: center;
  padding: 2px 0 2px 10px;
  font-size: 11px;
  border-top: 1px solid #444;
}

#runningTimerContainer, #timesContainer {
  display: flex;
  flex: 1 1 0;
  flex-direction: row;
  justify-self: right;
  justify-content: flex-end;
  align-items: center;
  max-width: 150px;
  max-height: 70px;
  height: 100%;
  width: 100%;
  gap: 10px;
  z-index: 1;
  user-select: none;
}

#timesContainer {
  padding-right: 4px;
}

#runningTimerContainer button {
  position: fixed;
  right: 129px;
  display: flex;
  justify-content: center;
  align-items: center;
  max-width: 20px;
  height: 70px;
  width: 100%;
  background-color: transparent;
  border: 0;
  padding: 0;
  transition: right 0.4s cubic-bezier(0.645, 0.045, 0.355, 1.000);
  z-index: 2;
}

#runningTimerContainer button.closed {
  right: 0px;
  padding-right: 10px;
}

#runningTimerArrow {
  rotate: -90deg;
  max-width: 20px;
  max-height: 20px;
  height: 100%;
  filter: brightness(1) invert(0.7);
  transition: rotate 0.2s, transform 0.2s;
  z-index: 2;
}

#runningTimerArrow.pointleft {
  rotate: 90deg;
}

#runningTimerArrow:hover {
  cursor: pointer;
  transform: scale(1.2);
}

#runningDisplayMinutes, #runningDisplaySeconds {
  display: flex;
  flex: 1 1 0;
  max-width: 50px;
  width: 100%;
  max-height: 62px;
  height: 100%;
  justify-content: center;
  align-items: center;
  background-color: #222;
  border-radius: 8px;
  pointer-events: none;
}

#runningDisplayMinutes p, #runningDisplaySeconds p {
  margin: 0;
  font-size: 24px;
}

.navigationIcons {
  justify-self: center;
  max-width: 40px;
  width: 100%;
  max-height: 40px;
  height: 100%;
  filter: brightness(0) invert(0.7);
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
