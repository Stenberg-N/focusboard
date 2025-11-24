<script lang="ts">
  import ContextMenu, { Item } from 'svelte-contextmenu';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writable } from 'svelte/store';
    import { text } from '@sveltejs/kit';

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

  async function loadTabs() {
    const data = await invoke<Tab[]>('get_tabs');
    tabs.set(data);
  }

  async function loadNotes() {
    console.log(`Loading notes for tab ID: ${currentTabId}`);
    const data = await invoke<Note[]>('get_notes', { tabId: currentTabId });
    notes.set(data);
  }

  async function addNote() {
    if (!newTitle) return;
    await invoke('create_note', { title: newTitle, content: newContent, tabId: currentTabId });
    newTitle = '';
    newContent = '';
    await loadNotes();
  }

  async function removeNote(id: number) {
    await invoke('delete_note', { id });
    await loadNotes();
  }

  async function addTab() {
    const newTab = await invoke<Tab>('create_tab', { name: 'New Tab' });
    tabs.update((t: Tab[]) => [...t, newTab]);
    currentTabId = newTab.id;
    await loadNotes();
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

  function startRename(tab: Tab) {
    editingTabId = tab.id;
    editingTabName = tab.name;
  }

  async function saveRename(tab: Tab) {
    if (editingTabName.trim() === '') return;
    await invoke('update_tab', { id: tab.id, name: editingTabName });
    tab.name = editingTabName;
    tabs.update((t: Tab[]) => [...t]);
    editingTabId = null;
  }

  function cancelRename() {
    editingTabId = null;
  }

  async function onRemoveTab() {
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
  }

  function selectTab(tabId: number) {
    console.log(`Selected Tab ID: ${tabId}`);
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
  <h1>Hello! This is a test.</h1>

  <div id="menuBar">
    <h2>Create Note in Current Tab</h2>
    <input bind:value={newTitle} placeholder="Title" />
    <textarea bind:value={newContent} placeholder="Content"></textarea>
    <button on:click={addNote} disabled={!currentTabId}>Save</button>
  </div>

  <h2>Notes for the Current Tab</h2>
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
            <small>Press Esc to cancel | click outside to save</small>
          </div>
        {:else}
          <h3 class="noteTitle">{note.title || 'Untitled'}</h3>
          <p class="noteContent">{note.content || 'No content'}</p>
          <small>Last edited: {note.updated_at}</small>
          <button on:click={() => removeNote(note.id)}>Delete</button>
        {/if}
      </div>
    {/each}
  {:else}
    <p>No tabs available.</p>
  {/if}

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
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.note {
  border: 1px solid blueviolet;
}

#tabBar {
  display: flex;
  flex-direction: row;
  justify-content: left;
  width: 100vh;
  border: 1px solid red;
}

.tab {
  text-align: left;
  direction: ltr;
  cursor: pointer;
  min-width: 10px;
  border: 1px solid green;
  margin: 0 0 0 5px;
  padding: 3px;
}

h1 {
  text-align: center;
}

.tab.selected {
  background-color: #444;
}

</style>