<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount, getContext } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { flip } from 'svelte/animate';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { load } from '@tauri-apps/plugin-store';
  import type { Store } from '@tauri-apps/plugin-store';
  import { dndzone, type DndEvent, dragHandleZone } from 'svelte-dnd-action';
  import { slide, fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import ComponentNote from '../components/componentNote.svelte';

  import type { Note, Tab } from '../types/types';
  import '../routes/app.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  const deleteNoteContext = getContext<{ getDeleteNoteId: () => number | null, setDeleteNoteId: (id: number | null) => void }>('deleteNoteContext');

  let {
    currentTabId,
    setCurrentTabId,
    currentTabName,
    setCurrentTabName,
    noteOpenStates,
    setStatus,
    setStore,
    store,
  }: {
    currentTabId: number | null;
    setCurrentTabId: (id: number | null) => void;
    currentTabName: string | null;
    setCurrentTabName: (name: string | null) => void;
    noteOpenStates: Record<number, boolean>;
    setStatus: (msg: string) => void;
    setStore: (s: Store) => void;
    store: Store | null;
  } = $props();

  let notes = $state<Note[]>([]);
  let currentTabNotes = $state<Note[]>([]);
  let tabs = $state<Tab[]>([]);
  let uiVisibility = $state({ tabBar: true, });
  let topLevelNotes = $derived.by(() => { return currentTabNotes.filter(n => n.parent_id === null).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0)) });
  let foundNotes = $derived.by(() => { return notes.filter(n => stripHtml(n.title).toLowerCase() === searchable?.trim().toLowerCase()) });
  let previewNotes = $state<Note[] | null>(null);
  let previewTabs = $state<Tab[] | null>(null);
  let zoomedNoteId = $state<number | null>(null);
  let deleteNoteId = $derived(deleteNoteContext.getDeleteNoteId());

  let hydrated = $state(false);
  let noteType = $state('basic');
  let contextMenu = $state<ContextMenu>()!;

  let previousTabId = $state<number | null>(null);
  let previousTabName = $state<string | null>(null);
  let editingTabId = $state<number | null>(null);
  let contextTabId = $state<number | null>(null);
  let contextTabName = $state<string | null>(null);
  let editingTabName = $state('');

  let searchInput = $state<HTMLInputElement>();
  let searchable = $state<string | null>(null);
  let isSearching = $state<boolean>(false);

  const flipDurationMs = 200;

  onMount(() => {
    void (async () => {
      store = await load('ui-state.json');

      await loadTabs();
      if (tabs.length === 0) {
        const newTab = await invoke<Tab>('create_tab', { name: 'Untitled' });
        tabs = [...tabs, newTab];
        setCurrentTabId(newTab.id);
        setCurrentTabName(newTab.name);
        setStatus("No prior tabs found. Creating tab");
      }
      setStatus(`Tabs loaded successfully. Loaded the last tab you left on: ${currentTabName}`);

      const savedTabId = await store.get<number>('currentTabId') ?? null;
      const savedTabName = await store.get<string>('currentTabName') ?? null;
      const savedOpenStates = await store.get<Record<number, boolean>>('noteOpenStates') ?? {};

      if (savedTabId !== null && savedTabName !== null && tabs.some(t => t.id === savedTabId && t.name === savedTabName)) {
        try {
          setCurrentTabId(savedTabId);
          setCurrentTabName(savedTabName);

        } catch (error) {
          console.error(`Failed to fetch the tab ${currentTabName} from previous session:`, error);
          setStatus(`Failed to fetch the tab ${currentTabName} from previous session: ${error}`);
        }
      }

      if (savedOpenStates) {
        try {
          Object.assign(noteOpenStates, savedOpenStates);

          hydrated = true;
        } catch (error) {
          console.error("Failed to set note states to their previous states:", error);
          setStatus(`Failed to set notes' states to their previous states: ${error}`);
        }
      }
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
        setStore(store);
      } catch (error) {
        console.error("Failed to update the store with the currently selected tab's ID and name:", error);
        setStatus(`Failed to update the store with the currently selected tab's ID and name: ${error}`);
      }
    }
  });

  $effect(() => {
    if (!store || !hydrated) return;

    void (async () => {
      try {
        await store.set('noteOpenStates', noteOpenStates);
        await store.save();
        setStore(store);
      } catch (error) {
        console.error("Failed to save note state change:", error);
        setStatus(`Failed to save note state change: ${error}`);
      }
    })();
  });

  $effect(() => {
    if (currentTabId !== null) {
      loadNotes(currentTabId);
    }
  });

  function stripHtml(html: string | undefined): string {
    if (!html) return '';
    const doc = new DOMParser().parseFromString(html, 'text/html');
    return doc.body.textContent?.trim() || '';
  }

  async function openLogs() {
    const logDir = await appLogDir();
    await openPath(logDir);
  }

  async function backupDatabase() {
    try {
      await invoke('backup_database');
      setStatus("Backup successful");
    } catch (error) {
      console.error("Database backup failed:", error);
      setStatus(`Database backup failed: ${error}`);
    }
  }

  async function loadNotes(tabId: number | null) {
    try {
      const data = await invoke<Note[]>('get_notes', { tabId: tabId });
      currentTabNotes = data;

      setStatus(`Notes loaded on tab ${currentTabName} successfully`);
    } catch (error) {
      console.error("get_notes failed:", error);
      setStatus(`Failed to load notes: ${error}`);
    }
  }

  async function addNote() {
    try {
      const newNote = await invoke<Note>('create_note', { title: 'Untitled', content: '', tabId: currentTabId, parentId: null, noteType: noteType });
      noteOpenStates[newNote.id] = true;

      await loadNotes(currentTabId);

      setStatus("Created note successfully");
    } catch (error) {
      console.error("create_note failed:", error);
      setStatus(`Failed to create note: ${error}`);
    }
  }

  async function loadTabs() {
    try {
      const data = await invoke<Tab[]>('get_tabs');
      tabs = data
        .sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0));

    } catch (error) {
      console.error("get_tabs failed:", error);
      setStatus(`Failed to load tabs: ${error}`);
    }
  }

  async function addTab() {
    try {
      const newTab = await invoke<Tab>('create_tab', { name: 'New Tab' });
      tabs = [...tabs, newTab];
      setCurrentTabId(newTab.id);
      setCurrentTabName(newTab.name);

      setStatus(`Added tab ${currentTabName} successfully`);
    } catch (error) {
      console.error("create_tab failed:", error);
      setStatus(`Failed to create tab: ${error}`);
    }
  }

  async function onRemoveTab() {
    try {
      if (contextTabId !== null) {
        await invoke('delete_tab', { id: contextTabId });
        await loadTabs();

        if (currentTabId === contextTabId && tabs.length > 0) {
          setCurrentTabId(tabs[0].id);
          setCurrentTabName(tabs[0].name);
        } else if (tabs.length === 0) {
          setCurrentTabId(null);
          setCurrentTabName(null);
        }
        await loadNotes(currentTabId);
        contextTabId = null;

        setStatus(`Deleted tab ${contextTabName} successfully`);
      }
    } catch (error) {
      console.error("delete_tab failed:", error);
      setStatus(`Failed to delete tab: ${error}`);
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

      setStatus(`Updated tab name to ${editingTabName} successfully`);
    } catch (error) {
      console.error("update_tab failed:", error)
      setStatus(`Error updating tab: ${error}`);
    }
  }

  function cancelRename() {
    editingTabId = null;
  }

  function selectTab(tabId: number, tabName: string) {
    setCurrentTabId(tabId);
    setCurrentTabName(tabName);
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

    newItems.forEach((item, index) => {
      item.order_id = index + 1;
    });
    topLevelNotes = [...newItems];

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

        setStatus("Notes reordered successfully");
        break;
      } catch (error) {
        console.error("Failed to reorder notes:", error);

        if (attempt >= maxRetries) {
          await loadNotes(currentTabId);

          setStatus(`Failed to reorder notes! Retrying. Error: ${error}`);
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

    newItems.forEach((item, index) => {
      item.order_id = index + 1;
    });
    tabs = [...newItems];

    const savedCurrentTabId = currentTabId;

    if (savedCurrentTabId !== null) {
      const currentTab = newItems.find(t => t.id === savedCurrentTabId);
      if (currentTab) {
        setCurrentTabId(currentTab.id);
        setCurrentTabName(currentTab.name);
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

        setStatus("Tabs reordered successfully");
        break;
      } catch (error) {
        console.error("Failed to reorder tabs:", error);
        if (attempt >= maxRetries) {
          await loadTabs();

          setStatus(`Failed to reorder tabs: ${error}`);
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
      notes = data;
    } catch (error) {
      console.error("Failed to fetch all notes:", error);
      setStatus(`Failed to fetch all notes: ${error}`);
    }
  }

  async function searchNotes() {
    await getAllNotes();
    isSearching = true;
    searchable = searchInput!.value;

    if (foundNotes.length <= 0) {
      setStatus("No matches found on search");
      isSearching = false;
    } else if (foundNotes.length > 0) {
      setStatus("Search completed");
    }
  }

  async function closeNotesSearch() {
    isSearching = false;
    searchable = null;
    foundNotes = [];

    await loadNotes(currentTabId);

    setStatus("Search closed");
  }

  let tabBarIsOpen = $derived(uiVisibility.tabBar);

  function tabBarToggle() {
    uiVisibility.tabBar = !uiVisibility.tabBar;
  }

  function zoomNote(id: number) {
    zoomedNoteId = id;
  }

  function closeZoom() {
    zoomedNoteId = null;
    setStatus("Zoomed note closed");
  }

  async function confirmDeleteNote() {
    try {
      const plainTitle = stripHtml(currentTabNotes.find(n => n.id === deleteNoteId)?.title) || 'Untitled';
      await invoke('delete_note', { id: deleteNoteId });
      deleteNoteContext.setDeleteNoteId(null);

      if (isSearching) {
        await getAllNotes();
      } else {
        await loadNotes(currentTabId);
      }

      setStatus(`Deleted note ${plainTitle} successfully`);
    } catch (error) {
      console.error('delete_note failed:', error);
      setStatus(`Failed to delete note: ${error}`);
    }
  }

</script>

{#if deleteNoteId}
  <div id="deleteNotificationContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
    <div id="deleteNotificationContent">
      <div id="deleteNotificationMessage">
        <p>Are you sure you want to delete this note?</p>
        <p><strong>{stripHtml(currentTabNotes.find(n => n.id == deleteNoteId)?.title)}</strong></p>
      </div>
      <div class="mainViewTimerSpacer"></div>
      <div id="deleteNotificationButtons">
        <button onclick={confirmDeleteNote}>Confirm</button>
        <button onclick={()=> deleteNoteContext.setDeleteNoteId(null)}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

{#if zoomedNoteId}
  <div role="button" tabindex="0" class="zoomedNoteOverlay" transition:fly={{ y: -100, duration: 200, easing: cubicInOut }} onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); closeZoom(); }}}>
    <div class="zoomedNoteContent">
      <p class="warnMessage">The close button does not save any changes made to the note. Hitting Escape will close the note without saving. Please remember to save the changes in the note edit mode before exiting the zoom.</p>
      <button class="zoomedNoteCloseBtn" onclick={closeZoom}>Close without saving</button>
      <ComponentNote
        note={currentTabNotes.find(n => n.id === zoomedNoteId)!}
        {currentTabNotes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
        reloadNotes={() => loadNotes(currentTabId)}
      ></ComponentNote>
    </div>
  </div>
{/if}

<div id="notesView" style="height: 100%; width: 100%;">
  <div style="display: flex; flex-direction: row; height: 70px; margin-left: 10px;">
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
      <div id="searchBarInputContainer">
        <input id="searchBarInput" bind:this={searchInput} onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchNotes(); } if (e.key === 'Escape') { e.preventDefault(); closeNotesSearch(); }}}>
        {#if !isSearching}
          <button onclick={() => {searchInput!.value = ''; searchable = null; setStatus("Search cleared") }}>
            <img id="clearSearchIcon" src="close.svg" alt="clearSearchIcon">
          </button>
        {/if}
      </div>
      {#if foundNotes!.length > 0}
        <button transition:fly={{ y: -100, duration: 200, easing: cubicInOut }} id="searchBarCloseBtn" onclick={closeNotesSearch}>
          <img id="closeIcon" src="close.svg" alt="CloseIcon">
        </button>
      {/if}
    </div>
  </div>

  <div id="middle" class:enlarged={!tabBarIsOpen}>
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
                  {note} {currentTabNotes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
                  reloadNotes={() => loadNotes(currentTabId)}
                ></ComponentNote>
              </div>
            {/each}
          {:else}
            {#if currentTabId}
              {#key topLevelNotes.map(n => n.id).join('-')}
                {#each (previewNotes ?? topLevelNotes) as note (note.id)}
                  <div style="display: flex; flex: 1 1 0;" animate:flip={{ duration: flipDurationMs }}>
                    <ComponentNote
                      {note} {currentTabNotes} {noteOpenStates} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
                      reloadNotes={() => loadNotes(currentTabId)}
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
  </div>

  {#if tabBarIsOpen}
    <div id="tabBar" transition:slide={{ delay: 100, duration: 200, easing: cubicInOut }} style="z-index: 1;">
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
</div>

<button id="toggleTabBar" onclick={tabBarToggle} class:enlarged={!tabBarIsOpen}>
  {#if tabBarIsOpen}
    <img class="arrowDown-icon" src="down-arrow.svg" alt="arrowDownIcon">
  {:else}
    <img class="arrowUp-icon" src="up-arrow.svg" alt="arrowUpIcon">
  {/if}
</button>

<ContextMenu bind:this={contextMenu}>
  <Item on:click={onRemoveTab}>Remove Tab</Item>
</ContextMenu>

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
