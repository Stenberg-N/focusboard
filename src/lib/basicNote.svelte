<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import 'overlayscrollbars/overlayscrollbars.css';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import collapse from 'svelte-collapse';
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';

  const noteOpenStates = getContext<Writable<Record<number, boolean>>>('noteOpenStates');

  export let note: {
    id: number;
    title: string;
    content: string;
    tab_id: number | null;
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

  let originalOpenState = false;
  let hasSavedState = false;

  $: open = isEditing || open;

  const options = {
    scrollbars: {
      autoHide: 'move' as const,
      autoHideDelay: 800,
      theme: 'os-theme-dark'
    }
  };

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
  class:editing={isEditing}
  role="article"
  on:dblclick={startEdit}
  use:clickOutside
>
  <div id="noteTitleBox">
    <div id="noteControls">
      <button on:click={toggle}>{open ? 'Hide' : 'Show'}</button>
      <button on:click={startEdit}>Edit</button>
      <button on:click={removeNote}>Delete</button>
    </div>
    <h3 class="noteTitle">{note.title || 'Untitled'}</h3>
  </div>
  <OverlayScrollbarsComponent {options}>
    <div id="noteContentOuter" use:collapse={{ open, duration: 0.4, easing: 'ease' }}>
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
        <textarea
          bind:value={editingContent}
          placeholder="Enter to save | Shift+Enter for new line | Esc to cancel"
          rows="6"
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
      {:else}
        <p class="noteContent">{note.content || 'No content'}</p>
        <small>Last edited: {note.updated_at}</small>
        <small>Note ID: {note.id}</small>
        <small>Tab ID: {note.tab_id}</small>
      {/if}
    </div>
  </OverlayScrollbarsComponent>
</div>

<style>
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

  .note.editing textarea {
    max-height: 550px;
    width: 100%;
    max-width: 100%;
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
    margin-right: 15px;
  }

  #noteTitleBox {
    justify-items: center;
  }

  #noteControls {
    display: flex;
    flex-direction: row;
    gap: 5px;
  }
</style>
