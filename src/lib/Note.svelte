<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import 'overlayscrollbars/overlayscrollbars.css';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { slide } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';
  import type { Note } from '../types/types';

  const noteOpenStates = getContext<Writable<Record<number, boolean>>>('noteOpenStates');
  const notes = getContext<Writable<Note[]>>('notes');

  export let note: {
    id: number;
    title: string;
    content: string;
    tab_id: number | null;
    parent_id: number | null;
    note_type: string;
    created_at: string;
    updated_at: string;
  };

  let open: boolean;

  $: open = $noteOpenStates[note.id] ?? true;

  export let reloadNotes: () => Promise<void>;
  export let setStatus: (msg: string) => void = () => {};

  let isEditing = false;
  let editingTitle = '';
  let editingContent = '';

  const isCategory = note.note_type === 'categorical';

  let originalOpenState = false;
  let hasSavedState = false;

  $: collapseOpen = isEditing || open;

  async function addChild() {
    try {
      const newNote = await invoke<typeof note>('create_note', { title: 'Untitled', content: '', tabId: note.tab_id, noteType: 'basic', parentId: note.id });
      notes.update((n: Note[]) => [...n, newNote])
      noteOpenStates.update(states => ({ ...states, [newNote.id]: (open = true) }));

      await reloadNotes();

      setStatus("Added sub-note successfully");
    } catch (error) {
      console.error("Failed to add sub-note:", error);
      setStatus(`Failed to add sub-note: ${error}`);
    }
  }

  function startEdit() {
    originalOpenState = open;
    hasSavedState = true;
    isEditing = true;
    editingTitle = note.title;
    editingContent = note.content;
  }

  async function saveEdit() {
    try {
      await invoke('update_note', { id: note.id, title: editingTitle, content: editingContent || '' });

      isEditing = false;
      noteOpenStates.update(states => ({ ...states, [note.id]: (open = true) }));

      await reloadNotes();

      setStatus(`Updated note ${note.title} successfully`);
    } catch (error) {
      console.error('update_note failed:', error);
      setStatus(`Failed to update note: ${error}`);
    }
  }

  function cancelEdit() {
    isEditing = false;

    if (hasSavedState) {
      noteOpenStates.update(states => ({ ...states, [note.id]: originalOpenState }));
      hasSavedState = false;
    }
  }

  async function removeNote() {
    try {
      await invoke('delete_note', { id: note.id });

      await reloadNotes();

      setStatus(`Deleted note ${note.title} successfully`);
    } catch (error) {
      console.error('delete_note failed:', error);
      setStatus(`Failed to delete note: ${error}`);
    }
  }

  function insertNewLineAtCursor(textarea: HTMLTextAreaElement) {
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const value = textarea.value;

    textarea.value = value.slice(0, start) + '\n' + value.slice(end);

    textarea.selectionStart = textarea.selectionEnd = start + 1;

    textarea.dispatchEvent(new Event('input'));
  }

  function clickOutside(node: HTMLElement) {
    const handleClick = (event: MouseEvent) => {
      if (isEditing && node && !node.contains(event.target as Node)) {
        saveEdit();
      }
    };
    document.addEventListener('click', handleClick, true);

    return {
      destroy() {
        document.removeEventListener('click', handleClick, true);
      }
    };
  }

  function toggle() {
    const isOpen = !(open ?? true);
    noteOpenStates.update(states => ({ ...states, [note.id]: isOpen }));
  }

</script>

<div
  class="note"
  class:category={isCategory}
  class:editing={isEditing}
  role="article"
  on:dblclick|stopPropagation={startEdit}
  use:clickOutside
>
  <div id="noteTitleBox">
    <small>Last edited: {note.updated_at}</small>
    <small>Note ID: {note.id}</small>
    <small>Tab ID: {note.tab_id}</small>
    <div id="noteControls">
      {#if isCategory}
        <button on:click={toggle} disabled={isEditing}>{open ? 'Hide' : 'Show'}</button>
        <button on:click|stopPropagation={addChild}>Add Note</button>
        <button on:click={startEdit}>Edit</button>
        <button on:click={removeNote}>Delete</button>
      {:else if !isCategory}
        <button on:click={toggle} disabled={isEditing}>{open ? 'Hide' : 'Show'}</button>
        <button on:click={startEdit}>Edit</button>
        <button on:click={removeNote}>Delete</button>
      {/if}
    </div>
    {#if isEditing}
      <input
        bind:value={editingTitle}
        placeholder="Title | Enter to save"
        on:keydown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            saveEdit();
          } else if (e.key === 'Escape') {
            cancelEdit();
          }
        }}
      />
    {:else}
      <h3 class="noteTitle">{note.title || 'Untitled'}</h3>
    {/if}
  </div>
  <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
    <div id="noteContentOuter">
      {#if collapseOpen}
        <div style="overflow: hidden;" transition:slide={{ duration: 400, easing: cubicInOut }}>
          {#if isEditing}
            {#if !isCategory}
              <textarea
                bind:value={editingContent}
                placeholder="Enter to save | Shift+Enter for new line | Esc to cancel"
                rows="16"
                on:keydown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    saveEdit();
                  } else if (e.key === 'Enter' && e.shiftKey) {
                    e.preventDefault();
                    insertNewLineAtCursor(e.currentTarget);
                  } else if (e.key === 'Escape') {
                    cancelEdit();
                  }
                }}
              >
              </textarea>
              <div class="edit-actions">
                <button on:click={saveEdit}>Save</button>
                <button on:click={cancelEdit}>Cancel</button>
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {:else if isCategory}
              <div class="edit-actions">
                <button on:click={saveEdit}>Save</button>
                <button on:click={cancelEdit}>Cancel</button>
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {/if}
          {:else}
            {#if !isCategory}
              <p class="noteContent">{note.content || 'No content'}</p>
            {:else if isCategory}
              <div class="subNotes">
                {#each $notes.filter(n => n.parent_id === note.id).sort((a, b) => b.id - a.id) as child (child.id)}
                  <svelte:self note={child} {reloadNotes} {setStatus} />
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </div>
  </OverlayScrollbarsComponent>
</div>

<style>
  * { box-sizing: border-box; }

  .note {
    display: flex;
    flex-direction: column;
    background: #2f2f2f;
    border-radius: 8px;
    padding: 16px;
    height: fit-content;
    max-height: 550px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .note.category {
    align-items: center;
    max-height: 800px;
    align-items: stretch;
  }

  .note.editing textarea, .note textarea {
    max-height: 550px;
    width: 100%;
    max-width: 100%;
    color: #f6f6f6;
    background: transparent;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    border: none;
    outline: none;
  }

  .noteTitle {
    margin: 0 0 18px 0;
  }

  .note:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 20px rgba(0,0,0,0.4);
  }

  .noteContent {
    flex: 1 1 auto;
    overflow-y: auto;
    margin: 12px 0;
    text-align: left;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  #noteContentOuter {
    padding: 0 15px;
  }

  #noteTitleBox {
    justify-items: center;
  }

  #noteControls {
    display: flex;
    flex-direction: row;
    gap: 5px;
  }

  .subNotes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(370px, 1fr));
    gap: 16px;
    margin-top: 16px;
    padding: 12px;
    background: #2a2a2a;
    border-radius: 8px;
  }

</style>
