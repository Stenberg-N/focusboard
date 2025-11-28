<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writable } from 'svelte/store';
  import { listen } from '@tauri-apps/api/event';
  import { appLogDir } from '@tauri-apps/api/path';
  import { openPath } from '@tauri-apps/plugin-opener';
  import LoaderOverlay from '../lib/loaderOverlay.svelte';

  const notes = writable<Note[]>([]);
  const tabs = writable<Tab[]>([]);

  let contextMenu: ContextMenu;

  let currentTabId: number | null = null;
  let editingTabId: number | null = null;
  let contextTabId: number | null = null;
  let editingTabName: string = '';

  let editingNoteId: number | null = null;
  let editingNoteTitle: string = '';
  let editingNoteContent: string = '';

  let newTitle: string = '';
  let newContent: string = '';

  type Note = {
    id: number;
    title: string;
    content: string;
    tab_id: number | null;
    created_at: string;
    updated_at: string;
  };

  type Tab = {
    id: number;
    name: string;
    created_at: string;
    updated_at: string;
  }

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
  });

  let showOverlay = false;

  listen('app-closing', () => {
    showOverlay = true;
  });

  async function loadTabs() {
    try {
      const data = await invoke<Tab[]>('get_tabs');
      tabs.set(data);
    } catch (error) {
      console.error("get_notes failed:", error);
    }
  }

  async function loadNotes() {
    try {
      const data = await invoke<Note[]>('get_notes', { tabId: currentTabId });
      notes.set(data);
    } catch (error) {
      console.error("get_notes failed:", error);
    }
  }

  async function addNote() {
    try {
      if (!newTitle) return;
      await invoke('create_note', { title: newTitle, content: newContent, tabId: currentTabId });
      newTitle = '';
      newContent = '';
      await loadNotes();
    } catch (error) {
      console.error("create_note failed:", error);
    }
  }

  async function removeNote(id: number) {
    try {
      await invoke('delete_note', { id });
      await loadNotes();
    } catch (error) {
      console.error("delete_note failed:", error);
    }
  }

  async function addTab() {
    try {
      const newTab = await invoke<Tab>('create_tab', { name: 'New Tab' });
      tabs.update((t: Tab[]) => [...t, newTab]);
      currentTabId = newTab.id;
      await loadNotes();
    } catch (error) {
      console.error("create_tab failed:", error);
    }
  }

  function clickOutside(node: HTMLElement, callback: () => void) {
    const handleClick = (event: MouseEvent) => {
      if (node && !node.contains(event.target as Node) && !event.defaultPrevented) {
        callback();
      }
    };
    document.addEventListener('click', handleClick, true);

    return {
      destroy() {
        document.removeEventListener('click', handleClick, true);
      }
    };
  }

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
    } catch (error) {
      console.error("update_tab failed:", error)
    }
  }

  function cancelRename() {
    editingTabId = null;
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
      }
    } catch (error) {
      console.error("delete_tab failed:", error);
    }
  }

  function selectTab(tabId: number) {
    currentTabId = tabId;
    loadNotes();
  }

  function startNoteEdit(note: Note) {
    editingNoteId = note.id;
    editingNoteTitle = note.title;
    editingNoteContent = note.content;
  }

  async function saveNoteEdit(note: Note) {
    if (editingNoteContent.trim() === '') return;
    await invoke('update_note', { id: note.id, title: editingNoteTitle, content: editingNoteContent || '' });
    note.title = editingNoteTitle;
    note.content = editingNoteContent;
    notes.update((n: Note[]) => [...n]);
    editingNoteId = null;
  }

  function cancelNoteEdit() {
    editingNoteId = null;
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
  </div>

  <div id="middle">
    <h2>Notes for the Current Tab</h2>
    <div id="noteContainer">
      {#if currentTabId}
        {#each $notes as note (note.id)}
          <div
            style="display: flex; flex-direction: column;"
            class="note"
            class:editing={editingNoteId === note.id}
            role="article"
            on:dblclick={() => startNoteEdit(note)}
            use:clickOutside={() => {
              if (editingNoteId === note.id) {
                saveNoteEdit(note);
              }
            }}
          >
            {#if editingNoteId === note.id}
              <input
                bind:value={editingNoteTitle}
                placeholder="Title | Enter to save"
                on:keydown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    saveNoteEdit(note);
                  } else if (e.key === 'Escape') {
                    cancelNoteEdit();
                  }
                }}
              />
              <textarea
                bind:value={editingNoteContent}
                placeholder="Enter to save | Shift+Enter for new line | Esc to cancel"
                rows="6"
                on:keydown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    saveNoteEdit(note);
                  } else if (e.key === 'Enter' && e.shiftKey) {
                    e.preventDefault();
                    insertNewLineAtCursor(e.currentTarget);
                  } else if (e.key === 'Escape') {
                    cancelNoteEdit();
                  }
                }}
              >
              </textarea>
              <div class="edit-actions">
                <button on:click={() => saveNoteEdit(note)}>Save</button>
                <button on:click={cancelNoteEdit}>Cancel</button>
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {:else}
              <h3 class="noteTitle">{note.title || 'Untitled'}</h3>
              <p class="noteContent">{note.content || 'No content'}</p>
              <small>Last edited: {note.updated_at}</small>
              <small>Tab ID: {currentTabId}</small>
              <small>Note ID: {note.id}</small>
              <button on:click={() => startNoteEdit(note)}>Edit</button>
              <button on:click={() => removeNote(note.id)}>Delete</button>
            {/if}
          </div>
        {/each}
      {:else}
        <p>No tabs available.</p>
      {/if}
    </div>
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

  <ContextMenu bind:this={contextMenu}>
    <Item on:click={onRemoveTab}>Remove Tab</Item>
  </ContextMenu>
</main>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #f6f6f6;
  background-color: #2f2f2f;

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
  box-sizing: border-box;
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
  box-sizing: border-box;
}

#middle {
  position: fixed;
  top: 70px;
  bottom: 30px;
  left: 0;
  right: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  scrollbar-width: thin;
  scrollbar-color: #555 #2f2f2f;
  scroll-behavior: smooth;
  border: 1px solid teal;
}

#middle::-webkit-scrollbar {
  width: 8px;
}
#middle::-webkit-scrollbar-track {
  background: #2f2f2f;
}
#middle::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}

#noteContainer {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
  width: 100%;
  max-width: calc(100vw - 250px);
  padding: 10px;
  border: 1px solid cyan;
}

.note {
  background: #383838;
  border-radius: 8px;
  padding: 16px;
  min-height: 120px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  transition: transform 0.2s, box-shadow 0.2s;
}

.noteContent {
  white-space: pre-wrap;
  word-wrap: break-word;
}

.note:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 20px rgba(0,0,0,0.4);
}

#tabBar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  flex-direction: row;
  justify-content: left;
  width: 100%;
  height: 30px;
  gap: 5px;
  border: 1px solid red;
  background-color: #2f2f2f;
  box-sizing: border-box;
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

</style>
