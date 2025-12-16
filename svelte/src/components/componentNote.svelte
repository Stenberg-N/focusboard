<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { slide } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';
  import { getContext } from 'svelte';
  import { writable, type Writable } from 'svelte/store';
  import { dndzone, type DndEvent, type Item as DndItem } from 'svelte-dnd-action';

  import type { Note } from '../types/types';
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/app.css';

  const noteOpenStates = getContext<Writable<Record<number, boolean>>>('noteOpenStates');
  const notes = getContext<Writable<Note[]>>('notes');
  const childNotes = writable<Note[]>([]);

  export let note: Note;

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

  $: childNotes.set(
    $notes
      .filter(n => n.parent_id === note.id)
      .sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0))
  );

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

  function handleDnd(e: CustomEvent<DndEvent<DndItem>>) {
    childNotes.set(e.detail.items as Note[]);
  }

  async function handleDndFinalize(e: CustomEvent<DndEvent<DndItem>>) {
    childNotes.set(e.detail.items as Note[]);

    const orderedIds = e.detail.items.map(n => n.id);

    try {
      await invoke('reorder_notes', { noteIds: orderedIds });
      setStatus('Sub-notes reordered successfully');
    } catch (error) {
      console.error("Failed to reorder sub-notes:", error);
      setStatus(`Failed to reorder sub-notes: ${error}`);
      await reloadNotes();
    }
  }

  function transformElement(element: HTMLElement | undefined) {
    if (element) {
      element.style.outline = '#723fffd0 solid 2px';
      element.style.zIndex = '1000';
    }
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
    <small>Order ID: {note.order_id}</small>
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
        <div style="overflow: hidden;" transition:slide={{ delay: 100, duration: 400, easing: cubicInOut }}>
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
              <div class="subNotes" use:dndzone={{
                items: $childNotes,
                type: `sub-notes-${note.id}`,
                flipDurationMs: 250,
                dropTargetStyle: {},
                transformDraggedElement: transformElement,
                morphDisabled: true,
                centreDraggedOnCursor: true }}
                on:consider={handleDnd}
                on:finalize={handleDndFinalize}
              >
                {#each $childNotes as child (child.id)}
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
