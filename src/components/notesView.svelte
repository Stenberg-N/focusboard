<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount, getContext } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { flip } from 'svelte/animate';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { load } from '@tauri-apps/plugin-store';
  import type { Store } from '@tauri-apps/plugin-store';
  import { dndzone, type DndEvent, dragHandleZone } from 'svelte-dnd-action';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import ComponentNote from '../components/componentNote.svelte';

  import type { Note, Tab } from '../types/types';
  import '../routes/style.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  const deleteNoteContext = getContext<{ getDeleteNoteId: () => number | null, setDeleteNoteId: (id: number | null) => void }>('deleteNoteContext');

  let {
    currentTabId,
    setCurrentTabId,
    currentTabName,
    setCurrentTabName,
    setStatus,
    setStore,
    store,
    noteHeightMultiplier = $bindable(),
    noteColumns = $bindable(),
    noteGap = $bindable(),
  }: {
    currentTabId: number | null;
    setCurrentTabId: (id: number | null) => void;
    currentTabName: string | null;
    setCurrentTabName: (name: string | null) => void;
    setStatus: (msg: string) => void;
    setStore: (s: Store) => void;
    store: Store | null;
    noteHeightMultiplier: "smaller" | "larger" | null;
    noteColumns: number | null;
    noteGap: number | null;
  } = $props();

  let notes = $state<Note[]>([]);
  let currentTabNotes = $state<Note[]>([]);
  let tabs = $state<Tab[]>([]);
  let topLevelNotes = $derived(currentTabNotes);
  let foundNotes = $derived.by(() => { return notes.filter(n => stripHtml(n.title).toLowerCase() === searchable?.trim().toLowerCase()) });
  let previewNotes = $state<Note[] | null>(null);
  let previewTabs = $state<Tab[] | null>(null);
  let zoomedNoteId = $state<number | null>(null);
  let deleteNoteId = $derived(deleteNoteContext.getDeleteNoteId());

  let contextMenu = $state<ContextMenu>()!;

  let editingTabId = $state<number | null>(null);
  let contextTabId = $state<number | null>(null);
  let contextTabName = $state<string | null>(null);
  let editingTabName = $state('');

  let searchInput = $state<HTMLInputElement>();
  let searchable = $state<string | null>(null);
  let isSearching = $state<boolean>(false);

  const flipDurationMs = 200;
  let windowHeight = $state<number>(0);
  let noteRowHeight = $state<number>(0);

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

      const savedTabId = await store.get<number>('currentTabId') ?? null;
      const savedTabName = await store.get<string>('currentTabName') ?? null;
      const savedNoteHeightMultiplier = await store.get<"smaller" | "larger">('noteHeightMultiplier') ?? "smaller";
      const savedNoteColumns = await store.get<number | null>('noteColumns') ?? 4;
      const savedNoteGap = await store.get<number | null>('noteGap') ?? 1;

      if (savedTabId !== null && savedTabName !== null && tabs.some(t => t.id === savedTabId && t.name === savedTabName)) {
        try {
          setCurrentTabId(savedTabId);
          setCurrentTabName(savedTabName);

        } catch (error) {
          console.error(`Failed to fetch the tab ${currentTabName} from previous session:`, error);
          setStatus(`Failed to fetch the tab ${currentTabName} from previous session: ${error}`);
        }
      }

      if (savedNoteHeightMultiplier) {
        try {
          noteHeightMultiplier = savedNoteHeightMultiplier;
        } catch (error) {
          console.error("Failed to set note height multiplier:", error);
        }
      }

      if (savedNoteColumns) {
        try {
          noteColumns = savedNoteColumns;
        } catch (error) {
          console.error("Failed to set note columns:", error);
        }
      }

      if (savedNoteGap) {
        try {
          noteGap = savedNoteGap;
        } catch (error) {
          console.log("Failed to set note gap:", error);
        }
      }
    })();
  });

  $effect(() => {
    if (currentTabId !== null && currentTabName !== null && store) {
      try {
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
    if (currentTabId !== null) {
      loadNotes(currentTabId);
    }
  });

  $effect(() => {
    if (noteHeightMultiplier !== null && store) {
      store.set('noteHeightMultiplier', noteHeightMultiplier);
      store.save();
      setStore(store);
    }
  });

  $effect(() => {
    if (noteColumns !== null && store) {
      store.set('noteColumns', noteColumns);
      store.save;
      setStore(store);
    }
  });

  $effect(() => {
    if (noteGap !== null && store) {
      store.set('noteGap', noteGap);
      store.save;
      setStore(store);
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined') windowHeight = window.innerHeight;
  });

  $effect(() => {
    
    if (noteHeightMultiplier === 'smaller' && noteGap) noteRowHeight = (windowHeight - (140 + (10 * noteGap))) / 2;
    else if (noteHeightMultiplier === 'larger') noteRowHeight = windowHeight - 140;
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
      await invoke<Note>('create_note', { title: 'Untitled', content: '', tabId: currentTabId });

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
      setCurrentTabId(tab.id);
      setCurrentTabName(tab.name);

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

        const content: HTMLElement | null = innerNote.querySelector('.noteContent');
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

<svelte:window bind:innerHeight={windowHeight} />

{#if deleteNoteId}
  <div class="notificationContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
    <div class="notificationContent">
      <div>
        <p>Are you sure you want to delete this note?</p>
        <p><strong>{stripHtml(currentTabNotes.find(n => n.id == deleteNoteId)?.title)}</strong></p>
      </div>
      <div class="spacer" style="border-bottom: 1px solid #444;"></div>
      <div class="notificationButtonContainer">
        <button class="primary-button" onclick={confirmDeleteNote}>Confirm</button>
        <button class="primary-button" onclick={()=> deleteNoteContext.setDeleteNoteId(null)}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

{#if zoomedNoteId}
  <div role="button" tabindex="0" class="zoomedNoteOverlay" onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); closeZoom(); }}}>
    <div class="zoomedNoteContent">
      <p class="warnMessage">The close button does not save any changes made to the note. Hitting Escape will close the note without saving. Please remember to save the changes in the note edit mode before exiting the zoom.</p>
      <button id="zoomedNoteCloseBtn" class="primary-button" onclick={closeZoom}>Close without saving</button>
      <ComponentNote
        note={currentTabNotes.find(n => n.id === zoomedNoteId)!}
        zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
        reloadNotes={() => loadNotes(currentTabId)}
      ></ComponentNote>
    </div>
  </div>
{/if}

<div id="notesView" style="height: 100%; width: 100%;">
  <div id="menuBar">
    <h2>Notes</h2>
    <div id="notesMenuBarControls">
      <div style="margin-left: 10px; min-width: 70px;">
        <span>Height</span>
        <select bind:value={noteHeightMultiplier} style="margin: 0;">
          <option value="smaller">Normal</option>
          <option value="larger">Larger</option>
        </select>
      </div>
      <div style="min-width: 38px;">
        <span>Columns</span>
        <select bind:value={noteColumns} style="margin: 0;">
          {#each Array.from({ length: 5}, (_, i) => i) as index}
            <option value={index+1}>{index+1}</option>
          {/each}
        </select>
      </div>
      <div style="min-width: 38px;">
        <span>Gap size</span>
        <select bind:value={noteGap} style="margin: 0;">
          {#each Array.from({ length: 4}, (_, i) => i) as index}
            <option value={index+1}>{index+1}0</option>
          {/each}
        </select>
      </div>
      <button class="primary-button" onclick={addNote} disabled={!currentTabId}>Add note</button>
      <button class="primary-button" onclick={openLogs}>Logs</button>
      <button class="primary-button" onclick={backupDatabase}>Backup database</button>
    </div>
    <div id="searchBarContainer">
      <button id="searchBarBtn" class="primary-button" onclick={searchNotes}>
        <img id="searchIcon" src="search.svg" alt="searchIcon">
      </button>
      <input id="searchBarInput" bind:this={searchInput} onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); searchNotes(); } if (e.key === 'Escape') { e.preventDefault(); closeNotesSearch(); }}}>
      {#if !isSearching}
        <button id="clearSearchBtn" class="primary-button" onclick={() => {searchInput!.value = ''; searchable = null; setStatus("Search cleared") }}>
          <img id="clearSearchIcon" src="close.svg" alt="clearSearchIcon">
        </button>
      {/if}
      {#if foundNotes!.length > 0}
        <button id="searchBarCloseBtn" class="primary-button" transition:fly={{ y: -100, duration: 200, easing: cubicInOut }} onclick={closeNotesSearch}>
          <img id="closeIcon" src="close.svg" alt="CloseIcon">
        </button>
      {/if}
    </div>
  </div>

  <div id="middle">
    <div id="innerNoteContainer" style="grid-auto-rows: {noteRowHeight}px; grid-template-columns: repeat({noteColumns}, minmax(312px, 1fr)); gap: {10 * noteGap!}px;" use:dragHandleZone={{
      items: previewNotes ?? topLevelNotes,
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
              {note} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
              reloadNotes={() => loadNotes(currentTabId)}
            ></ComponentNote>
          </div>
        {/each}
      {:else}
        {#if currentTabNotes.length > 0}
          {#key topLevelNotes.map(n => n.id).join('-')}
            {#each (previewNotes ?? topLevelNotes) as note (note.id)}
              <div style="display: flex; flex: 1 1 0;" animate:flip={{ duration: flipDurationMs }}>
                <ComponentNote
                  {note} zoomedNote={zoomNote} zoomedNoteId={zoomedNoteId} setStatus={setStatus} isSearching={isSearching} getAllNotes={getAllNotes}
                  reloadNotes={() => loadNotes(currentTabId)}
                ></ComponentNote>
              </div>
            {/each}
          {/key}
        {:else}
          <div style="position: fixed; inset: 70px 0 50px 85px; display: flex; align-items: center; justify-content: center;">
            <span style="user-select: none; font-size: 24px;">No notes yet.</span>
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <div id="tabBar" style="z-index: 1;">
    <button id="buttonAddTab" class="primary-button" onclick={addTab}>Add Tab</button>
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
</div>

<ContextMenu bind:this={contextMenu}>
  <Item on:click={onRemoveTab}>Remove Tab</Item>
</ContextMenu>

<style>

#menuBar {
  position: fixed;
  z-index: 1;
  left: 85px;
  right: 0;
  display: flex; 
  flex-direction: row;
  height: 70px;
  margin-left: 10px;
  background-color: rgba(15,15,15,0.8);
  backdrop-filter: blur(20px);
}

#menuBar h2 {
  width: 70px;
  margin-left: 15px;
  text-align: left;
  user-select: none;
}

#notesMenuBarControls {
  display: flex;
  flex: 1 1 0;
  max-width: 425px;
  flex-direction: row;
  align-items: center;
  gap: 5px;
  user-select: none;
}

#notesMenuBarControls div {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  height: 100%;
  user-select: none;
}

#notesMenuBarControls span {
  font-size: 10px;
  height: 15px;
  line-height: 1;
  padding: 2px;
}

#notesMenuBarControls button {
  min-width: 73px;
}

#notesMenuBarControls select {
  background-color: #222;
  color: #f6f6f6;
  height: 40px;
  max-width: 100px;
  width: 100%;
  outline: 0;
  border: 0;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: box-shadow 0.2s;
  user-select: none;
}

#notesMenuBarControls select:hover {
  background-color: #333;
  cursor: pointer;
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

#notesMenuBarControls select option {
  background-color: #222;
}

#searchBarContainer {
  position: relative;
  display: flex;
  flex-direction: row;
  align-self: center;
  align-items: center;
  justify-content: left;
  max-width: 500px;
  width: 100%;
  max-height: 40px;
  height: 100%;
  margin: 0 175px 0 25px;
}

#searchBarBtn {
  width: 60px;
  height: 40px;
  background-color: #222;
  border-radius: 20px 0 0 20px;
  border: 1px solid #444;
  border-right: none;
}

#searchIcon {
  width: 25px;
  height: 25px;
  filter: brightness(0) invert(0.7);
  user-select: none;
}

#searchBarCloseBtn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  margin-left: 10px;
  border: 1px solid #444;
}

#closeIcon {
  height: 18px;
  width: 18px;
  filter: brightness(0) invert(0.7);
  user-select: none;
}

#clearSearchBtn, #searchBarInput {
  border: 0;
  outline: 0;
}

#clearSearchBtn {
  position: absolute;
  top: 50%;
  right: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10px;
  width: 10px;
  background-color: transparent;
  box-shadow: none;
  transform: translateY(-50%);
}

#clearSearchIcon {
  position: relative;
  height: 10px;
  width: 10px;
  filter: brightness(0) invert(0.7);
  user-select: none;
}

#clearSearchIcon:hover {
  filter: brightness(0) invert(0.9);
}

#searchBarInput {
  width: 100%;
  height: 100%;
  padding-left: 10px;
  background: #222;
  border: 1px solid #444;
  color: #f6f6f6;
  font-size: 16px;
  border-radius: 0 20px 20px 0;
}

#searchBarInput:focus {
  border-color: #723fffd0;
}

#searchBarBtn:hover, #searchBarCloseBtn:hover {
  cursor: pointer;
  background-color: #333;
  transform: unset;
}

#searchBarCloseBtn:hover {
  transform: translateY(-4px);
}

#middle {
  position: fixed;
  top: 0px;
  bottom: 50px;
  left: 85px;
  right: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.notificationContainer {
  bottom: 55px;
}

#innerNoteContainer {
  display: grid;
  width: calc(100vw - 85px);
  height: 100vh;
  padding: 80px 10px 10px 16px;
  overflow-y: auto;
  overflow-x: hidden;
  scrollbar-gutter: stable;
}

#innerNoteContainer::-webkit-scrollbar-track {
  margin-top: 70px;
}

#tabBar {
  position: fixed;
  bottom: 20px;
  left: 85px;
  right: 0;
  display: flex;
  flex-direction: row;
  justify-content: left;
  width: 100%;
  height: 30px;
  gap: 20px;
  border-top: 1px solid #444;
}

#buttonAddTab {
  width: 70px;
  font-size: 12px;
  border-radius: 0 6px 6px 0;
  max-height: 24.5px;
  height: 100%;
  align-self: center;
  margin-left: 1px;
  box-shadow: unset;
  transform: unset;
}

#buttonAddTab:hover {
  transform: unset;
  box-shadow: unset;
}

#tabList {
  display: flex;
  margin: 0;
  gap: 5px;
  width: 100%;
  align-items: center;
}

.tab {
  text-align: left;
  direction: ltr;
  min-width: 10px;
  padding: 5px;
  color: #f6f6f6;
  text-align: center;
  background-color: #222;
  border: none;
  border-radius: 6px;
  max-height: 24.5px;
  height: 100%;
  transition: transform 0.2s;
}

.tab:hover {
  background-color: #333;
  transform: translateY(-2px);
}

.tab.selected {
  outline: 1px solid #723fffd0;
}

.tab.editing {
  padding: 0;
}

.tab input {
  max-width: 120px;
  width: 100%;
  background-color: transparent;
  color: #f6f6f6;
  border: none;
  outline: none;
  padding-left: 5px;
}

.zoomedNoteOverlay {
  position: fixed;
  display: flex;
  align-items: center;
  z-index: 10000;
  top: 0;
  left: 0;
  bottom: 20px;
  width: 100vw;
  height: 100vh - 20px;
  background-color: #0f0f0f;
}

.zoomedNoteContent {
  position: relative;
  inset: 0;
  width: 50%;
  height: 80%;
  margin: auto;
  overflow: auto;
  padding: 20px;
}

.zoomedNoteContent .warnMessage {
  position: fixed;
  top: 0;
  left: 160px;
  right: 160px;
  display: flex;
  word-break: break-all;
  align-items: center;
  justify-self: center;
  max-height: 48px;
  font-size: 14px;
  margin: 0;
  margin-top: 10px;
  padding: 10px;
  border: 1px solid #723fffd0;
  border-radius: 8px;
}

#zoomedNoteCloseBtn {
  position: fixed;
  top: 10px;
  left: 10px;
  justify-self: center;
  max-width: unset;
  width: 140px;
  max-height: unset;
  height: 45px;
  border-radius: 6px;
}

</style>
