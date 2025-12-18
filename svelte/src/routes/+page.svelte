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
  import { dndzone, type DndEvent } from 'svelte-dnd-action';
  import { slide } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import LoaderOverlay from '../components/loaderOverlay.svelte';
  import ComponentNote from '../components/componentNote.svelte';

  import type { Note, Tab } from '../types/types';
  import './app.css';
  import 'overlayscrollbars/overlayscrollbars.css';

  let notes = $state<Note[]>([]);
  let tabs = $state<Tab[]>([]);
  let uiVisibility = $state({ tabBar: true, });
  let topLevelNotes = $state<Note[]>([]);
  let previewNotes = $state<Note[] | null>(null);
  let previewTabs = $state<Tab[] | null>(null);

  let noteOpenStates = $state<Record<number, boolean>>({});
  setContext('noteOpenStates', () => noteOpenStates);
  setContext('uiVisibility', () => uiVisibility);
  setContext('notes', () => notes);

  let store = $state<Store>();
  let hydrated = $state(false);

  let contextMenu: ContextMenu;

  let currentTabId = $state<number | null>(null);
  let currentTabName = $state<string | null>(null);
  let previousTabId = $state<number | null>(null);
  let previousTabName = $state<string | null>(null);
  let editingTabId = $state<number | null>(null);
  let contextTabId = $state<number | null>(null);
  let contextTabName = $state<string | null>(null);
  let editingTabName = $state('');

  let noteType = $state('basic');

  let statusBar!: HTMLSpanElement;

  let flipDurationMs = $state<number>(0);

  onMount(() => {
    void (async () => {
      store = await load('ui-state.json');

      flipDurationMs = 250;

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
      noteType = 'basic';

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
      element.style.outline = '#723fffd0 solid 2px';
      element.style.zIndex = '1000';
    }
  }

  let tabBarIsOpen = $derived(uiVisibility.tabBar);

  function tabBarToggle() {
    uiVisibility.tabBar = !uiVisibility.tabBar;
  }

</script>

<main class="container">
  {#if showOverlay}
    <LoaderOverlay />
  {/if}

  <div id="menuBar">
    <small>Create Note in Current Tab</small>
    <select bind:value={noteType}>
      <option value="basic">Basic</option>
      <option value="categorical">Categorical</option>
    </select>
    <button onclick={addNote} disabled={!currentTabId}>Create Note</button>
    <button onclick={openLogs}>Open logs</button>
    <button onclick={backupDatabase}>Backup Database</button>
  </div>

  <div id="middle" class:enlarged={!tabBarIsOpen}>
    <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
      <div id="noteContainer" use:dndzone={{
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
        {#if currentTabId}
          {#key topLevelNotes.map(n => n.id).join('-')}
            {#each (previewNotes ?? topLevelNotes) as note (note.id)}
              <div animate:flip={{ duration: flipDurationMs }}>
                <ComponentNote
                  {note} {notes} {noteOpenStates}
                  setStatus={(msg) => (statusBar.textContent = msg)}
                  reloadNotes={() => loadNotes(currentTabId)}
                ></ComponentNote>
              </div>
            {/each}
          {/key}
        {:else}
          <p>No tabs available.</p>
        {/if}
      </div>
    </OverlayScrollbarsComponent>
  </div>

  {#if tabBarIsOpen}
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
    <button id="toggleTabBar" onclick={tabBarToggle}>{tabBarIsOpen ? 'v' : '^'}</button>
  </div>

  <ContextMenu bind:this={contextMenu}>
    <Item on:click={onRemoveTab}>Remove Tab</Item>
  </ContextMenu>
</main>

<style>

:global {
  .os-theme-dark {
    --os-handle-bg: #555;
    --os-handle-bg-hover: #aaa;
    --os-handle-bg-active: #aaa;
  }
}

</style>
