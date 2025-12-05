<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writable } from 'svelte/store';
  import { listen } from '@tauri-apps/api/event';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import 'overlayscrollbars/overlayscrollbars.css';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import LoaderOverlay from '../lib/loaderOverlay.svelte';
  import BasicNote from '../lib/basicNote.svelte';

  const notes = writable<Note[]>([]);
  const tabs = writable<Tab[]>([]);

  let contextMenu: ContextMenu;

  let currentTabId: number | null = null;
  let editingTabId: number | null = null;
  let contextTabId: number | null = null;
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
    await loadTabs();
    if ($tabs.length === 0) {
      const newTab = await invoke<Tab>('create_tab', { name: 'Untitled' });
      tabs.update((t: Tab[]) => [...t, newTab]);
      currentTabId = newTab.id;
    } else {
      currentTabId = $tabs[0].id;
    }
    await loadNotes();
    if (statusBar) {
      statusBar.textContent = "App setup complete";
    }
  });

  let showOverlay = false;

  listen('app-closing', () => {
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
      console.error("database backup failed:", error);
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
      }
    }
  }

  async function loadNotes() {
    try {
      const data = await invoke<Note[]>('get_notes', { tabId: currentTabId });
      notes.set(data);
      if (statusBar) {
        statusBar.textContent = "Loaded notes successfully";
      }
    } catch (error) {
      console.error("get_notes failed:", error);
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
      }
    }
  }

  async function addNote() {
    try {
      if (!newTitle) return;
      await invoke('create_note', { title: newTitle, content: newContent, tabId: currentTabId, noteType: noteType });
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
        statusBar.textContent = `Error: ${error}`;
      }
    }
  }

  async function loadTabs() {
    try {
      const data = await invoke<Tab[]>('get_tabs');
      tabs.set(data);
      if (statusBar) {
        statusBar.textContent = "Loaded tabs successfully";
      }
    } catch (error) {
      console.error("get_notes failed:", error);
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
      }
    }
  }

  async function addTab() {
    try {
      const newTab = await invoke<Tab>('create_tab', { name: 'New Tab' });
      tabs.update((t: Tab[]) => [...t, newTab]);
      currentTabId = newTab.id;
      await loadNotes();
      if (statusBar) {
        statusBar.textContent = "Added tab successfully";
      }
    } catch (error) {
      console.error("create_tab failed:", error);
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
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
        } else if ($tabs.length === 0) {
          currentTabId = null;
        }
        await loadNotes();
        contextTabId = null;
        if (statusBar) {
          statusBar.textContent = "Deleted tab successfully";
        }
      }
    } catch (error) {
      console.error("delete_tab failed:", error);
      if (statusBar) {
        statusBar.textContent = `Error: ${error}`;
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
        statusBar.textContent = "Updated tab name successfully";
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

  function selectTab(tabId: number) {
    currentTabId = tabId;
    loadNotes();
  }

  function handleContextMenu(tabId: number, event: MouseEvent) {
    event.preventDefault();
    currentTabId = tabId;
    contextTabId = tabId;
    loadNotes();
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
        on:click={() => selectTab(tab.id)}
        on:dblclick={() => startRename(tab)}
        on:contextmenu={(e) => handleContextMenu(tab.id, e)}
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
