<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount, setContext } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { flip } from 'svelte/animate';
  import { listen } from '@tauri-apps/api/event';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { load } from '@tauri-apps/plugin-store';
  import type { Store } from '@tauri-apps/plugin-store';
  import { dndzone, type DndEvent, dragHandleZone } from 'svelte-dnd-action';
  import { slide, fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import LoaderOverlay from '../components/loaderOverlay.svelte';
  import ComponentNote from '../components/componentNote.svelte';
  import TimerView from '../components/timerView.svelte';

  import type { Note, Tab } from '../types/types';
  import './app.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  let notes = $state<Note[]>([]);
  let tabs = $state<Tab[]>([]);
  let uiVisibility = $state({ tabBar: true, runningTimer: true, });
  let topLevelNotes = $derived.by(() => {return notes.filter(n => n.parent_id === null).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0)) });
  let searchedNotes = $state<Note[]>([]);
  let foundNotes = $derived.by(() => {return searchedNotes.filter(n => n.title === searchable) });
  let previewNotes = $state<Note[] | null>(null);
  let previewTabs = $state<Tab[] | null>(null);
  let zoomedNoteId = $state<number | null>(null);
  let timerViewVisible = $state<boolean>(false);
  let noteOpenStates = $state<Record<number, boolean>>({});

  setContext('noteOpenStates', () => noteOpenStates);
  setContext('uiVisibility', () => uiVisibility);
  setContext('notes', () => notes);

  let displayMinutes = $state(0);
  let displaySeconds = $state(0);
  let isRunningTimerFinished = $state(false);
  let setTimerMessage = $state('');

  let store = $state<Store>();
  let hydrated = $state(false);

  let contextMenu = $state<ContextMenu>()!;

  let currentTabId = $state<number | null>(null);
  let currentTabName = $state<string | null>(null);
  let previousTabId = $state<number | null>(null);
  let previousTabName = $state<string | null>(null);
  let editingTabId = $state<number | null>(null);
  let contextTabId = $state<number | null>(null);
  let contextTabName = $state<string | null>(null);
  let editingTabName = $state('');

  let noteType = $state('basic');

  let statusBar = $state<HTMLSpanElement>()!;

  let searchInput = $state<HTMLInputElement>();
  let searchable = $state<string | null>(null);
  let isSearching = $state<boolean>(false);

  let flipDurationMs = $state<number>(0);

  onMount(() => {
    void (async () => {
      store = await load('ui-state.json');

      flipDurationMs = 200;

      await loadTabs();
      if (tabs.length === 0) {
        const newTab = await invoke<Tab>('create_tab', { name: 'Untitled' });
        tabs = [...tabs, newTab];
        currentTabId = newTab.id;
        currentTabName = newTab.name;
      }

      const savedTabId = await store.get<number>('currentTabId') ?? null;
      const savedTabName = await store.get<string>('currentTabName') ?? null;
      const savedOpenStates = await store.get<Record<number, boolean>>('noteOpenStates');

      if (savedTabId !== null && savedTabName !== null && tabs.some(t => t.id === savedTabId && t.name === savedTabName)) {
        try {
          currentTabId = savedTabId;
          currentTabName = savedTabName;

        } catch (error) {
          console.error(`Failed to fetch the tab ${currentTabName} from previous session:`, error);
          if (statusBar) {
            statusBar.textContent = `Failed to fetch the tab ${currentTabName} from previous session: ${error}`;
          }
        }
      }

      if (savedOpenStates) {
        try {
          Object.assign(noteOpenStates, savedOpenStates);

          hydrated = true;
        } catch (error) {
          console.error("Failed to set note states to their previous states:", error);
          if (statusBar) {
            statusBar.textContent = `Failed to set notes' states to their previous states: ${error}`;
          }
        }
      }

      if (statusBar) statusBar.textContent = "App setup complete";

      updateFromTimer();
      const currentlyRunningTimer = setInterval(updateFromTimer, 1000);
      return () => clearInterval(currentlyRunningTimer);
    })();
  });

  $effect(() => {
    if (currentTabId !== previousTabId && currentTabId !== null && currentTabName !== null && store) {
      try {
        previousTabId = currentTabId;
        previousTabName = currentTabName;
        store.set('currentTabId', currentTabId);
        store.set('currentTabName', currentTabName);
        store.save();
      } catch (error) {
        console.error("Failed to update the store with the currently selected tab's ID and name:", error);
        if (statusBar) {
          statusBar.textContent = `Failed to update the store with the currently selected tab's ID and name: ${error}`;
        }
      }
    }
  });

  $effect(() => {
    if (!store || !hydrated) return;

    void (async () => {
      try {
        await store.set('noteOpenStates', noteOpenStates);
        await store.save();
      } catch (error) {
        console.error("Failed to save note state change:", error);
        if (statusBar) {
          statusBar.textContent = `Failed to save note state change: ${error}`;
        }
      }
    })();
  });

  $effect(() => {
    if (currentTabId !== null) {
      loadNotes(currentTabId);
    }
  });

  $effect(() => {
    topLevelNotes = notes.filter(n => n.parent_id === null).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0))
  });

  $effect(() => {
    foundNotes = searchedNotes.filter(n => n.title === searchable)
  });

  let showOverlay = $state<boolean>(false);

  listen('app-closing', async () => {
    if (store) {
      store.set('currentTabId', currentTabId);
      store.set('noteOpenStates', noteOpenStates);
      store.set('currentTabName', currentTabName);
      store.save();
    }
    showOverlay = true;
  });

  async function openLogs() {
    const logDir = await appLogDir();
    await openPath(logDir);
  }

  async function backupDatabase() {
    try {
      await invoke('backup_database');
      if (statusBar) {
        statusBar.textContent = "Backup successful";
      }
    } catch (error) {
      console.error("Database backup failed:", error);
      if (statusBar) {
        statusBar.textContent = `Database backup failed: ${error}`;
      }
    }
  }

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

  async function loadNotes(tabId: number | null) {
    try {
      const data = await invoke<Note[]>('get_notes', { tabId: tabId });
      notes = data;

      topLevelNotes = notes
        .filter(n => n.parent_id === null)
        .sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0));

      if (statusBar) {
        statusBar.textContent = `Loaded notes on tab ${currentTabName} successfully`;
      }
    } catch (error) {
      console.error("get_notes failed:", error);
      if (statusBar) {
        statusBar.textContent = `Failed to load notes: ${error}`;
      }
    }
  }

  async function addNote() {
    try {
      const newNote = await invoke<Note>('create_note', { title: 'Untitled', content: '', tabId: currentTabId, parentId: null, noteType: noteType });
      noteOpenStates[newNote.id] = true;

      await loadNotes(currentTabId);

      if (statusBar) {
        statusBar.textContent = "Created note successfully";
      }
    } catch (error) {
      console.error("create_note failed:", error);
      if (statusBar) {
        statusBar.textContent = `Failed to create note: ${error}`;
      }
    }
  }

  async function loadTabs() {
    try {
      const data = await invoke<Tab[]>('get_tabs');
      tabs = data
        .sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0));

      if (statusBar) {
        statusBar.textContent = `Loaded tabs successfully. Loaded the last tab you left on: ${currentTabName}`;
      }
    } catch (error) {
      console.error("get_tabs failed:", error);
      if (statusBar) {
        statusBar.textContent = `Failed to load tabs: ${error}`;
      }
    }
  }

  async function addTab() {
    try {
      const newTab = await invoke<Tab>('create_tab', { name: 'New Tab' });
      tabs = [...tabs, newTab];
      currentTabId = newTab.id;
      currentTabName = newTab.name;

      if (statusBar) {
        statusBar.textContent = `Added tab ${currentTabName} successfully`;
      }
    } catch (error) {
      console.error("create_tab failed:", error);
      if (statusBar) {
        statusBar.textContent = `Failed to create tab: ${error}`;
      }
    }
  }

  async function onRemoveTab() {
    try {
      if (contextTabId !== null) {
        await invoke('delete_tab', { id: contextTabId });
        await loadTabs();

        if (currentTabId === contextTabId && tabs.length > 0) {
          currentTabId = tabs[0].id;
          currentTabName = tabs[0].name;
        } else if (tabs.length === 0) {
          currentTabId = null;
          currentTabName = null;
        }
        await loadNotes(currentTabId);
        contextTabId = null;

        if (statusBar) {
          statusBar.textContent = `Deleted tab ${contextTabName} successfully`;
        }
      }
    } catch (error) {
      console.error("delete_tab failed:", error);
      if (statusBar) {
        statusBar.textContent = `Failed to delete tab: ${error}`;
      }
    }
  }

  let inputElement = $state<HTMLInputElement | null>(null);

  function startRename(tab: Tab) {
    editingTabId = tab.id;
    editingTabName = tab.name;

    setTimeout(() => {
      inputElement?.select();
      inputElement?.focus();
    }, 0);
  }

  async function saveRename(tab: Tab) {
    try {
      if (editingTabName.trim() === '') return;
      await invoke('update_tab', { id: tab.id, name: editingTabName });
      tab.name = editingTabName;
      tabs = [...tabs];
      editingTabId = null;
      if (statusBar) {
        statusBar.textContent = `Updated tab name to ${editingTabName} successfully`;
      }
    } catch (error) {
      console.error("update_tab failed:", error)
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
      }
    }
  }

  function cancelRename() {
    editingTabId = null;
  }

  function selectTab(tabId: number, tabName: string) {
    currentTabId = tabId;
    currentTabName = tabName;
  }

  function handleContextMenu(tabId: number, tabName: string, event: MouseEvent) {
    event.preventDefault();
    contextTabId = tabId;
    contextTabName = tabName;
    contextMenu.show(event);
  }

  let pendingNoteUpdate: { ids: number[]; tabId: number | null; parentId: null } | null = null;
  let areNoteSyncing = false;

  function handleDndNote(e: CustomEvent<DndEvent<Note>>) {
    previewNotes = [...e.detail.items] as Note[];
  }

  function handleDndFinalizeNote(e: CustomEvent<DndEvent<Note>>) {
    previewNotes = null;
    const newItems = [...e.detail.items] as Note[];
    topLevelNotes = newItems;

    const orderedIds = newItems.map(n => n.id);

    pendingNoteUpdate = { ids: orderedIds, tabId: currentTabId, parentId: null };

    if (!areNoteSyncing) {
      processPendingNoteUpdate();
    }
  }

  async function processPendingNoteUpdate() {
    if (!pendingNoteUpdate) {
      areNoteSyncing = false;
      return;
    }

    areNoteSyncing = true;

    const currentBatch = pendingNoteUpdate;
    pendingNoteUpdate = null;

    let attempt = 0;
    const maxRetries = 3;

    while (attempt <= maxRetries) {
      try {
        await invoke('reorder_notes', { tabId: currentBatch.tabId, noteIds: currentBatch.ids, parentId: currentBatch.parentId });

        noteOpenStates = { ...noteOpenStates };

        if (statusBar) statusBar.textContent = 'Notes reordered successfully';
        break;
      } catch (error) {
        console.error("Failed to reorder notes:", error);

        if (attempt >= maxRetries) {
          await loadNotes(currentTabId);
          if (statusBar) statusBar.textContent = `Failed to reorder notes! Retrying. Error: ${error}`;
          break;
        }
        await new Promise(r => setTimeout(r, 200 * Math.pow(2, attempt)));
        attempt++;
      }
    }

    areNoteSyncing = false;
    processPendingNoteUpdate();
  }

  let pendingTabUpdate: { ids: number[] } | null = null;
  let areTabsSyncing = false;

  function handleDndTab(e: CustomEvent<DndEvent<Tab>>) {
    previewTabs = [...e.detail.items] as Tab[];
  }

  function handleDndFinalizeTab(e: CustomEvent<DndEvent<Tab>>) {
    previewTabs = null;
    const newItems = [...e.detail.items] as Tab[];
    tabs = newItems;

    const savedCurrentTabId = currentTabId;

    if (savedCurrentTabId !== null) {
      const currentTab = newItems.find(t => t.id === savedCurrentTabId);
      if (currentTab) {
        currentTabId = currentTab.id;
        currentTabName = currentTab.name;
      }
    }

    const orderedIds = newItems.map(n => n.id);
    pendingTabUpdate = { ids: orderedIds };

    if (!areTabsSyncing) {
      processPendingTabUpdate();
    }
  }

  async function processPendingTabUpdate() {
    if (!pendingTabUpdate) {
      areTabsSyncing = false;
      return;
    }

    areTabsSyncing = true;

    const currentBatch = pendingTabUpdate;
    pendingTabUpdate = null;

    let attempt = 0;
    const maxRetries = 3;

    while (attempt <= maxRetries) {
      try {
        await invoke('reorder_tabs', { tabIds: currentBatch.ids });

        if (statusBar) statusBar.textContent = "Tabs reordered successfully";
        break;
      } catch (error) {
        console.error("Failed to reorder tabs:", error);
        if (attempt >= maxRetries) {
          await loadTabs();
          if (statusBar) statusBar.textContent = `Failed to reorder tabs: ${error}`;
          break;
        }
        await new Promise(r => setTimeout(r, 200 * Math.pow(2, attempt)));
        attempt++;
      }
    }

    areTabsSyncing = false;
    processPendingTabUpdate();
  }

  function transformElement(element: HTMLElement | undefined) {
    if (element) {
      const innerNote: HTMLElement | null = element.querySelector('.note');
      element.style.outline = 'none';
      element.style.willChange = 'transform';
      if (innerNote) {
        innerNote.style.willChange = 'transform';
        const currentHeight = element.getBoundingClientRect().height;
        innerNote.style.outline = '2px solid #723fffd0';
        innerNote.style.outlineOffset = '-2px';
        innerNote.style.borderRadius = '8px';
        innerNote.style.boxShadow = '0 8px 20px rgba(0,0,0,0.4)';
        innerNote.style.zIndex = '1000';
        innerNote.style.height = `${currentHeight}px`;
        innerNote.style.transform = 'scale(0.5)';
        innerNote.style.transition = 'transform 0.2s cubic-bezier(0.645, 0.045, 0.355, 1.000)';
        innerNote.style.transformOrigin = 'top-center';

        const content: HTMLElement | null = innerNote.querySelector('.noteContent, .subNotes');
        if (content) content.style.display = 'none';
      }
    }
  }

  async function getAllNotes() {
    try {
      const data = await invoke<Note[]>('get_all_notes');
      searchedNotes = data;
    } catch (error) {
      console.error("Failed to fetch all notes:", error);
      statusBar.textContent = `Failed to fetch all notes: ${error}`;
    }
  }

  async function searchNotes() {
    await getAllNotes();
    isSearching = true;
    searchable = searchInput!.value;

    if (foundNotes.length <= 0) {
      statusBar.textContent = 'No matches found on search';
      isSearching = false;
    } else if (foundNotes.length > 0) {
      statusBar.textContent = 'Search completed';
    }
  }

  async function closeNotesSearch() {
    isSearching = false;
    searchable = null;
    foundNotes = [];

    await loadNotes(currentTabId);

    statusBar.textContent = 'Search closed';
  }

  let tabBarIsOpen = $derived(uiVisibility.tabBar);

  function tabBarToggle() {
    uiVisibility.tabBar = !uiVisibility.tabBar;
  }

  let runningTimerIsOpen = $derived(uiVisibility.runningTimer);

  function runningTimerToggle() {
    uiVisibility.runningTimer = !uiVisibility.runningTimer;
  }

  function zoomNote(id: number) {
    zoomedNoteId = id;
  }

  function closeZoom() {
    zoomedNoteId = null;
    if (statusBar) statusBar.textContent = 'Closed zoomed note';
  }

</script>

<main class="container">
  {#if showOverlay}
    <LoaderOverlay />
  {/if}

  {#if zoomedNoteId}
    <div role="button" tabindex="0" class="zoomedNoteOverlay" transition:fly={{ y: -100, duration: 200, easing: cubicInOut }} onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); closeZoom(); }}}>
      <div class="zoomedNoteContent">
        <p class="warnMessage">The close button does not save any changes made to the note. Hitting Escape will close the note without saving. Please remember to save the changes in the note edit mode before exiting the zoom.</p>
        <button class="zoomedNoteCloseBtn" onclick={closeZoom}>Close without saving</button>
        <ComponentNote
          note={notes.find(n => n.id === zoomedNoteId)!}
          {notes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId}
          setStatus={(msg) => (statusBar.textContent = msg)}
          reloadNotes={() => loadNotes(currentTabId)}
          isSearching={isSearching}
          getAllNotes={getAllNotes}
        ></ComponentNote>
      </div>
    </div>
  {/if}

  {#if isRunningTimerFinished && !timerViewVisible}
    <div id="mainViewTimerFinishedContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
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

  <div id="menuBar">
    {#if timerViewVisible}
      <h2>Timer</h2>
    {:else}
      <h2>Notes</h2>
      <div id="notesMenuBarControls">
        <select bind:value={noteType}>
          <option value="basic">Basic</option>
          <option value="categorical">Categorical</option>
        </select>
        <button onclick={addNote} disabled={!currentTabId}>Create Note</button>
        <button onclick={openLogs}>Open logs</button>
        <button onclick={backupDatabase}>Backup Database</button>
      </div>
      <div id="searchBarContainer">
        <button id="searchBarBtn" onclick={searchNotes}>
          <img id="searchIcon" src="search.svg" alt="searchIcon">
        </button>
        <input id="searchBarInput" bind:this={searchInput} onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchNotes(); } if (e.key === 'Escape') { e.preventDefault(); closeNotesSearch(); }}}>
        {#if foundNotes!.length > 0}
          <button transition:fly={{ y: -100, duration: 200, easing: cubicInOut }} id="searchBarCloseBtn" onclick={closeNotesSearch}>
            <img id="closeIcon" src="close.svg" alt="CloseIcon">
          </button>
        {/if}
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
    {/if}
  </div>

  <div id="middle" class:enlarged={!tabBarIsOpen || timerViewVisible}>
    <div id="navigationBar">
      <button id="timerViewBtn" onclick={() => (timerViewVisible = !timerViewVisible)}>
        {#if timerViewVisible}
          <img id="noteIcon" src="note.svg" alt="noteIcon">
        {:else}
          <img id="clockIcon" src="clock.svg" alt="clockIcon">
        {/if}
      </button>
    </div>

    {#if timerViewVisible}
      <TimerView setStatus={(msg) => (statusBar.textContent = msg)} />
    {:else}
      <div id="noteContainer" class:enlarged={!tabBarIsOpen}>
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <div id="innerNoteContainer" use:dragHandleZone={{
            items: previewNotes ?? topLevelNotes,
            type: 'top-level-note',
            flipDurationMs: flipDurationMs,
            dropTargetStyle: {},
            transformDraggedElement: transformElement,
            morphDisabled: true,
            centreDraggedOnCursor: true }}
            onconsider={handleDndNote}
            onfinalize={handleDndFinalizeNote}
          >
            {#if foundNotes!.length > 0}
              {#each foundNotes as note (note.id)}
                <div style="display: flex; flex: 1 1 0;">
                  <ComponentNote
                    {note} {notes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId}
                    setStatus={(msg) => (statusBar.textContent = msg)}
                    reloadNotes={() => loadNotes(currentTabId)}
                    isSearching={isSearching}
                    getAllNotes={getAllNotes}
                  ></ComponentNote>
                </div>
              {/each}
            {:else}
              {#if currentTabId}
                {#key topLevelNotes.map(n => n.id).join('-')}
                  {#each (previewNotes ?? topLevelNotes) as note (note.id)}
                    <div style="display: flex; flex: 1 1 0;" animate:flip={{ duration: flipDurationMs }}>
                      <ComponentNote
                        {note} {notes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId}
                        setStatus={(msg) => (statusBar.textContent = msg)}
                        reloadNotes={() => loadNotes(currentTabId)}
                        isSearching={isSearching}
                        getAllNotes={getAllNotes}
                      ></ComponentNote>
                    </div>
                  {/each}
                {/key}
              {:else}
                <p>No tabs available.</p>
              {/if}
            {/if}
          </div>
        </OverlayScrollbarsComponent>
      </div>
    {/if}
  </div>

  {#if tabBarIsOpen && !timerViewVisible}
    <div id="tabBar" transition:slide={{ delay: 100, duration: 200, easing: cubicInOut }}>
      <button id="buttonAddTab" onclick={addTab}>Add Tab</button>
      <div id="tabList" use:dndzone={{
        items: previewTabs ?? tabs,
        type: 'tabs',
        flipDurationMs: flipDurationMs,
        dropTargetStyle: {},
        transformDraggedElement: transformElement,
        morphDisabled: true,
        centreDraggedOnCursor: true }}
        onconsider={handleDndTab}
        onfinalize={handleDndFinalizeTab}
      >
        {#key tabs.map(t => t.id).join('-')}
          {#each (previewTabs ?? tabs) as tab (tab.id)}
            <button animate:flip={{ duration: flipDurationMs }}
              role="textbox"
              tabindex="0"
              class="tab"
              class:editing={editingTabId === tab.id}
              class:selected={currentTabId === tab.id}
              onclick={() => selectTab(tab.id, tab.name)}
              ondblclick={() => startRename(tab)}
              oncontextmenu={(event) => handleContextMenu(tab.id, tab.name, event)}
            >
              {#if editingTabId === tab.id}
                <input
                  bind:this={inputElement}
                  bind:value={editingTabName}
                  onblur={() => saveRename(tab)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') saveRename(tab);
                    if (e.key === 'Escape') cancelRename();
                  }}
                />
              {:else}
                {tab.name || 'Untitled'}
              {/if}
            </button>
          {/each}
        {/key}
      </div>
    </div>
  {/if}

  <div id="statusBar">
    <span bind:this={statusBar}></span>
    {#if !timerViewVisible}
      <button id="toggleTabBar" onclick={tabBarToggle}>
        {#if tabBarIsOpen}
          <img class="arrowDown-icon" src="down-arrow.svg" alt="arrowDownIcon">
        {:else}
          <img class="arrowUp-icon" src="up-arrow.svg" alt="arrowUpIcon">
        {/if}
      </button>
    {/if}
  </div>

  <ContextMenu bind:this={contextMenu}>
    <Item on:click={onRemoveTab}>Remove Tab</Item>
  </ContextMenu>
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
