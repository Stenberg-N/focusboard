<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount, setContext } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writable } from 'svelte/store';
  import { listen } from '@tauri-apps/api/event';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import 'overlayscrollbars/overlayscrollbars.css';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { load } from '@tauri-apps/plugin-store';
  import type { Store } from '@tauri-apps/plugin-store';
  import LoaderOverlay from '../lib/loaderOverlay.svelte';
  import BasicNote from '../lib/basicNote.svelte';

  const notes = writable<Note[]>([]);
  const tabs = writable<Tab[]>([]);

  let store: Store;

  const noteOpenStates = writable<Record<number, boolean>>({});
  setContext('noteOpenStates', noteOpenStates);

  let contextMenu: ContextMenu;

  let currentTabId: number | null = null;
  let currentTabName: string | null = null;
  let previousTabId: number | null = null;
  let previousTabName: string | null = null;
  let editingTabId: number | null = null;
  let contextTabId: number | null = null;
  let contextTabName: string | null = null;
  let editingTabName: string = '';

  let newTitle: string = '';
  let newContent: string = '';

  let noteType: string = 'basic';

  type Note = {
    id: number;
    title: string;
    content: string;
    tab_id: number | null;
    note_type: string;
    created_at: string;
    updated_at: string;
  };

  type Tab = {
    id: number;
    name: string;
    created_at: string;
    updated_at: string;
  }

  let statusBar!: HTMLSpanElement;

  onMount(async () => {
    store = await load('ui-state.json');

    await loadTabs();
    if ($tabs.length === 0) {
      const newTab = await invoke<Tab>('create_tab', { name: 'Untitled' });
      tabs.update((t: Tab[]) => [...t, newTab]);
      currentTabId = newTab.id;
      currentTabName = newTab.name;
    }

    const savedTabId = await store.get<number>('currentTabId') ?? null;
    const savedTabName = await store.get<string>('currentTabName') ?? null;
    const savedOpenStates = await store.get<Record<number, boolean>>('noteOpenStates');

    if (savedTabId !== null && savedTabName !== null && $tabs.some(t => t.id === savedTabId && t.name === savedTabName)) {
      try {
        currentTabId = savedTabId;
        currentTabName = savedTabName;
        await loadNotes();
      } catch (error) {
        console.error(`Failed to fetch the tab ${currentTabName} from previous session:`, error);
        if (statusBar) {
          statusBar.textContent = `Failed to fetch the tab ${currentTabName} from previous session: ${error}`;
        }
      }
    }

    if (savedOpenStates) {
      try {
        noteOpenStates.set(savedOpenStates);
      } catch (error) {
        console.error("Failed to set note states to their previous states:", error);
        if (statusBar) {
          statusBar.textContent = `Failed to set notes' states to their previous states: ${error}`;
        }
      }
    }

    noteOpenStates.subscribe(async (states) => {
      if (store) {
        try {
          await store.set('noteOpenStates', states);
          await store.save();
        } catch (error) {
          console.error("Failed to save note state change:", error);
          if (statusBar) {
            statusBar.textContent = `Failed to save note state change: ${error}`;
          }
        }
      }
    });

    if (statusBar) statusBar.textContent = "App setup complete";
  });

  $: if (currentTabId !== previousTabId && currentTabId !== null && currentTabName !== previousTabName && currentTabName !== null && store) {
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

  let showOverlay = false;

  listen('app-closing', async () => {
    if (store) {
      store.set('currentTabId', currentTabId);
      store.set('noteOpenStates', $noteOpenStates);
      store.set('currentTabName', currentTabName);
      store.save();
    }
    showOverlay = true;
  });

  function insertNewLineAtCursor(textarea: HTMLTextAreaElement) {
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const value = textarea.value;

    textarea.value = value.slice(0, start) + '\n' + value.slice(end);

    textarea.selectionStart = textarea.selectionEnd = start + 1;

    textarea.dispatchEvent(new Event('input'));
  }

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

  function clearStaleOpenStates(validNoteIds: number[]) {
    noteOpenStates.update(states => {
      const newStates: Record<number, boolean> = {};
      for (const id of validNoteIds) {
        newStates[id] = states[id] ?? true;
      }
      return newStates;
    });
  }

  async function loadNotes() {
    try {
      const data = await invoke<Note[]>('get_notes', { tabId: currentTabId });
      notes.set(data);

      const validIds = data.map(n => n.id);
      clearStaleOpenStates(validIds);

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
      if (!newTitle) return;
      const newNote = await invoke<Note>('create_note', { title: newTitle, content: newContent, tabId: currentTabId, noteType: noteType });
      notes.update((n: Note[]) => [...n, newNote]);
      newTitle = '';
      newContent = '';
      noteType = 'basic';
      await loadNotes();
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
      tabs.set(data);
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
      tabs.update((t: Tab[]) => [...t, newTab]);
      currentTabId = newTab.id;
      currentTabName = newTab.name;
      await loadNotes();
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
        if (currentTabId === contextTabId && $tabs.length > 0) {
          currentTabId = $tabs[0].id;
          currentTabName = $tabs[0].name;
        } else if ($tabs.length === 0) {
          currentTabId = null;
          currentTabName = null;
        }
        await loadNotes();
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

  let inputElement!: HTMLInputElement;

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
      tabs.update((t: Tab[]) => [...t]);
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
    loadNotes();
  }

  function handleContextMenu(tabId: number, tabName: string, event: MouseEvent) {
    event.preventDefault();
    contextTabId = tabId;
    contextTabName = tabName;
    contextMenu.show(event);
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
    <input bind:value={newTitle} placeholder="Title" />
    <textarea
      bind:value={newContent}
      placeholder="Content"
      on:keydown={(e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          addNote();
        }
        if (e.key === 'Enter' && e.shiftKey) {
          e.preventDefault();
          insertNewLineAtCursor(e.currentTarget);
        }
      }}>
    </textarea>
    <button on:click={addNote} disabled={!currentTabId}>Save</button>
    <button on:click={openLogs}>Open logs</button>
    <button on:click={backupDatabase}>Backup Database</button>
  </div>

  <div id="middle">
    <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
      <div id="noteContainer">
        {#if currentTabId}
          {#each $notes as note (note.id)}
            {#if note.note_type === 'basic'}
              <BasicNote
                {note}
                  setStatus={(msg) => (statusBar.textContent = msg)}
                  reloadNotes={loadNotes}
              ></BasicNote>
            {:else if note.note_type === 'categorical'}
              placeholder
            {/if}
          {/each}
        {:else}
          <p>No tabs available.</p>
        {/if}
      </div>
    </OverlayScrollbarsComponent>
  </div>

  <div id="tabBar">
    <button on:click={addTab}>Add Tab</button>
    {#each $tabs as tab (tab.id)}
      <button
        role="textbox"
        tabindex="0"
        class="tab"
        class:selected={currentTabId === tab.id}
        on:click={() => selectTab(tab.id, tab.name)}
        on:dblclick={() => startRename(tab)}
        on:contextmenu={(event) => handleContextMenu(tab.id, tab.name, event)}
      >
        {#if editingTabId === tab.id}
          <input
            bind:this={inputElement}
            bind:value={editingTabName}
            on:blur={() => saveRename(tab)}
            on:keydown={(e) => {
              if (e.key === 'Enter') saveRename(tab);
              if (e.key === 'Escape') cancelRename();
            }}
          />
        {:else}
          {tab.name || 'Untitled'}
        {/if}
      </button>
    {/each}
  </div>

  <div id="statusBar">
    <span bind:this={statusBar}></span>
  </div>

  <ContextMenu bind:this={contextMenu}>
    <Item on:click={onRemoveTab}>Remove Tab</Item>
  </ContextMenu>
</main>

<style>
:root {
  * { box-sizing: border-box; }
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #f6f6f6;
  background-color: #222;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  height: 100%;
  width: 100%;
  overflow: hidden;
  margin: 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  text-align: center;
}

#menuBar {
  position: fixed;
  top: 0;
  right: 0;
  left: 0;
  display: flex;
  flex-direction: row;
  align-items: center;
  height: 70px;
  border: 1px solid yellow;
}

#middle {
  position: fixed;
  top: 70px;
  bottom: 50px;
  left: 0;
  right: 0;
  overflow: hidden;
  padding: 0 20px 0 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  border: 1px solid teal;
}

#noteContainer {
  display: grid;
  flex: 1 1 0;
  min-height: 0;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  width: 100%;
  max-width: calc(100vw - 100px);
  padding: 10px;
  border: 1px solid cyan;
  overflow-y: auto;
  overflow-x: hidden;
}

#tabBar {
  position: fixed;
  bottom: 20px;
  left: 0;
  right: 0;
  display: flex;
  flex-direction: row;
  justify-content: left;
  width: 100%;
  height: 30px;
  gap: 5px;
  border: 1px solid red;
  background-color: #222;
}

.tab {
  text-align: left;
  direction: ltr;
  cursor: pointer;
  min-width: 10px;
  border: 1px solid green;
  padding: 3px;
}

.tab.selected {
  background-color: #444;
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
  background-color: #222;
  align-items: center;
  padding: 2px 0 2px 10px;
  font-size: 11px;
}

:global {
  .os-theme-dark {
    --os-handle-bg: #555;
    --os-handle-bg-hover: #aaa;
    --os-handle-bg-active: #aaa;
  }
}

</style>
