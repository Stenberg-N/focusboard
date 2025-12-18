<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import ComponentNote from './componentNote.svelte';
  import { cubicInOut } from 'svelte/easing';
  import { type DndEvent, dragHandleZone, dragHandle } from 'svelte-dnd-action';

  import type { Note } from '../types/types';
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/app.css';

  let {
    note,
    reloadNotes,
    setStatus,
    notes,
    noteOpenStates = $bindable<Record<number, boolean>>()
  }: {
    note: Note;
    reloadNotes: () => void;
    setStatus: (msg: string) => void;
    notes: Note[];
    noteOpenStates: Record<number, boolean>;
  } = $props();

  let childNotes = $state<Note[]>([]);
  let previewChildNotes = $state<Note[] | null>(null);

  $effect(() => {
    childNotes = notes.filter(n => n.parent_id === note.id).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0))
  });

  let open = $derived(noteOpenStates[note.id] ?? true);

  let isEditing = $state<boolean>(false);
  let editingTitle = $state('');
  let editingContent = $state('');

  const isCategory = note.note_type === 'categorical';

  let originalOpenState = false;
  let hasSavedState = false;

  let collapseOpen = $derived(isEditing || open);

  const flipDurationMs = 200;

  async function addChild() {
    try {
      const newNote = await invoke<typeof note>('create_note', { title: 'Untitled', content: '', tabId: note.tab_id, noteType: 'basic', parentId: note.id });
      noteOpenStates[newNote.id] = true

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
      noteOpenStates[note.id] = true;

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
      noteOpenStates[note.id] = originalOpenState;
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
    const isOpen = !(noteOpenStates[note.id] ?? true);
    noteOpenStates[note.id] = isOpen;
  }

  function CloseAllSubnotes() {
    const selectedNotes = childNotes.filter(n => n.parent_id === note.id);

    for (const selected of selectedNotes) {
      noteOpenStates[selected.id] = false;
    }
  }

  function OpenAllSubnotes() {
    const selectedNotes = childNotes.filter(n => n.parent_id === note.id);

    for (const selected of selectedNotes) {
      noteOpenStates[selected.id] = true;
    }
  }

  let pendingChildNoteUpdate: { ids: number[]; tabId: number | null; parentId: number } | null = null;
  let areChildNotesSyncing = false;

  function handleDnd(e: CustomEvent<DndEvent<Note>>) {
    previewChildNotes = [...e.detail.items] as Note[];
  }

  function handleDndFinalize(e: CustomEvent<DndEvent<Note>>) {
    previewChildNotes = null;
    const newItems = [...e.detail.items];
    childNotes = newItems;

    const orderedIds = newItems.map(n => n.id);

    pendingChildNoteUpdate = { ids: orderedIds, tabId: note.tab_id, parentId: note.id };

    if (!areChildNotesSyncing) {
      processPendingChildNoteUpdate();
    }
  }

  async function processPendingChildNoteUpdate() {
    if (!pendingChildNoteUpdate) {
      areChildNotesSyncing = false;
      return;
    }

    areChildNotesSyncing = true;

    const currentBatch = pendingChildNoteUpdate;
    pendingChildNoteUpdate = null;

    let attempt = 0;
    const maxRetries = 3;

    while (attempt <= maxRetries) {
      try {
        await invoke('reorder_notes', { noteIds: currentBatch.ids, tabId: currentBatch.tabId, parentId: currentBatch.parentId });

        setStatus('Sub-notes reordered successfully');
        break;
      } catch (error) {
        console.error("Failed to reorder sub-notes:", error);

        if (attempt >= maxRetries) {
          await reloadNotes();
          setStatus(`Failed to reorder sub-notes! Retrying. Error: ${error}`);
          break;
        }
        await new Promise(r => setTimeout(r, 200 * Math.pow(2, attempt)));
        attempt++;
      }
    }

    areChildNotesSyncing = false;
    processPendingChildNoteUpdate();
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
  ondblclick={e => { e.stopPropagation(); startEdit(); }}
  use:clickOutside
>
  <div id="noteTitleBox">
    <div class="dragHandle" use:dragHandle><p>Handle</p></div>
    <small>Last edited: {note.updated_at}</small>
    <small>Note ID: {note.id}</small>
    <small>Tab ID: {note.tab_id}</small>
    <small>Order ID: {note.order_id}</small>
    <small>Parent ID: {note.parent_id}</small>
    <div id="noteControls">
      {#if isCategory}
        <button onclick={toggle} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>{open ? 'Hide' : 'Show'}</button>
        <button onclick={OpenAllSubnotes} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Open all</button>
        <button onclick={CloseAllSubnotes} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Close all</button>
        <button onclick={addChild} ondblclick={e => { e.stopPropagation(); }}>Add Note</button>
        <button onclick={startEdit} ondblclick={e => { e.stopPropagation(); }}>Edit</button>
        <button onclick={removeNote} ondblclick={e => { e.stopPropagation(); }}>Delete</button>
      {:else if !isCategory}
        <button onclick={toggle} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>{open ? 'Hide' : 'Show'}</button>
        <button onclick={startEdit} ondblclick={e => { e.stopPropagation(); }}>Edit</button>
        <button onclick={removeNote} ondblclick={e => { e.stopPropagation(); }}>Delete</button>
      {/if}
    </div>
    {#if isEditing}
      <input
        bind:value={editingTitle}
        placeholder="Title | Enter to save"
        onkeydown={(e) => {
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
                onkeydown={(e) => {
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
                <button onclick={saveEdit}>Save</button>
                <button onclick={cancelEdit}>Cancel</button>
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {:else if isCategory}
              <div class="edit-actions">
                <button onclick={saveEdit}>Save</button>
                <button onclick={cancelEdit}>Cancel</button>
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {/if}
          {:else}
            {#if !isCategory}
              <p class="noteContent">{note.content || 'No content'}</p>
            {:else if isCategory}
              <div class="subNotes" use:dragHandleZone={{
                items: previewChildNotes ?? childNotes,
                type: `child-note`,
                flipDurationMs: flipDurationMs,
                dropTargetStyle: {},
                transformDraggedElement: transformElement,
                morphDisabled: true,
                centreDraggedOnCursor: true }}
                onconsider={handleDnd}
                onfinalize={handleDndFinalize}
              >
                {#key childNotes.map(n => n.id).join('-')}
                  {#each (previewChildNotes ?? childNotes) as child (child.id)}
                    <div animate:flip={{ duration: flipDurationMs }}>
                      <ComponentNote note={child} {reloadNotes} {setStatus} {notes} {noteOpenStates} />
                    </div>
                  {/each}
                {/key}
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </div>
  </OverlayScrollbarsComponent>
</div>
