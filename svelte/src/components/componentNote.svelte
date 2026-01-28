<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import ComponentNote from './componentNote.svelte';
  import { cubicInOut } from 'svelte/easing';
  import { type DndEvent, dragHandleZone, dragHandle, SHADOW_ITEM_MARKER_PROPERTY_NAME } from 'svelte-dnd-action';
  import { Editor, EditorContent } from 'svelte-tiptap';
  import StarterKit from '@tiptap/starter-kit';
  import { TextStyle } from '@tiptap/extension-text-style';
  import Color from '@tiptap/extension-color';
  import { Extension } from '@tiptap/core';

  import type { Note } from '../types/types';
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/app.css';

  let {
    note,
    reloadNotes,
    setStatus,
    notes,
    noteOpenStates = $bindable<Record<number, boolean>>(),
    zoomedNote,
    zoomedNoteId = null,
    isSearching = false,
    getAllNotes,
  }: {
    note: Note;
    reloadNotes: () => void;
    setStatus: (msg: string) => void;
    notes: Note[];
    noteOpenStates: Record<number, boolean>;
    zoomedNote: (id: number) => void;
    zoomedNoteId: number | null;
    isSearching: boolean;
    getAllNotes: () => Promise<void>;
  } = $props();

  let childNotes = $derived.by(() => {return notes.filter(n => n.parent_id === note.id).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0)) });
  let previewChildNotes = $state<Note[] | null>(null);

  let isZoomed = $derived(zoomedNoteId === note.id);

  $effect(() => {
    childNotes = notes.filter(n => n.parent_id === note.id).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0))
  });

  let open = $derived(noteOpenStates[note.id] ?? true);

  let isEditing = $state<boolean>(false);
  let editingTitle = $state('');
  let editingContent = $state('');
  let editor = $state<Editor | null>(null);
  let titleEditor = $state<Editor | null>(null);
  let SelectedContentType = $state<'noteTitle' | 'noteContent' | null>(null);

  const isCategory = note.note_type === 'categorical';

  let originalOpenState = false;
  let hasSavedState = false;

  let collapseOpen = $derived(isEditing || open);

  const flipDurationMs = 200;

  $effect(() => {
    if (isEditing) {
      if (!editor) {
        editor = new Editor({
          extensions: [
            StarterKit,
            TextStyle,
            Color,
            FontSize,
          ],
          content: editingContent,
          onUpdate: ({ editor }) => { editingContent = editor.getHTML(); },
          onFocus: () => setSelectedContentType('noteContent'),
        });
        editor.view.dom.addEventListener('keydown', handleKeyDown);
      }

      if (!titleEditor) {
        titleEditor = new Editor({
          extensions: [
            StarterKit,
            TextStyle,
            Color,
            FontSize,
          ],
          content: editingTitle,
          onUpdate: ({ editor }) => { editingTitle = editor.getHTML(); },
          onFocus: () => setSelectedContentType('noteTitle'),
        });
        titleEditor.view.dom.addEventListener('keydown', handleKeyDown);
      }
    } else if (!isEditing) {
      return () => {
        if (editor) {
          editor.view.dom.removeEventListener('keydown', handleKeyDown);
          editor.destroy();
          editor = null;
        }
        if (titleEditor) {
          titleEditor.view.dom.removeEventListener('keydown', handleKeyDown);
          titleEditor.destroy;
          titleEditor = null;
        }
      };
    }
  });

  async function addChild() {
    try {
      const newNote = await invoke<typeof note>('create_note', { title: 'Untitled', content: '', tabId: note.tab_id, noteType: 'basic', parentId: note.id });
      noteOpenStates[newNote.id] = true

      if (isSearching) {
        await getAllNotes();
      } else {
        await reloadNotes();
      }

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

  function stripHtml(html: string): string {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    return doc.body.textContent?.trim() || '';
  }

  function setSelectedContentType(type: 'noteTitle' | 'noteContent') {
    SelectedContentType = type;
  }

  function getActiveEditor() {
    if (SelectedContentType === 'noteTitle') return titleEditor;
    if (SelectedContentType === 'noteContent') return editor;
    return editor ?? titleEditor;
  }

  async function saveEdit() {
    try {
      editingContent = editor?.getHTML() || '';
      editingTitle = titleEditor?.getHTML() || '';

      await invoke('update_note', { id: note.id, title: editingTitle, content: editingContent || '' });

      isEditing = false;
      noteOpenStates[note.id] = true;

      if (isSearching) {
        await getAllNotes();
      } else {
        await reloadNotes();
      }

      const plainTitle = stripHtml(editingTitle) || 'Untitled';
      setStatus(`Updated note ${plainTitle} successfully`);
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

      if (isSearching) {
        await getAllNotes();
      } else {
        await reloadNotes();
      }

      const plainTitle = stripHtml(editingTitle) || 'Untitled';
      setStatus(`Deleted note ${plainTitle} successfully`);
    } catch (error) {
      console.error('delete_note failed:', error);
      setStatus(`Failed to delete note: ${error}`);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      saveEdit();
    } else if (e.key === 'Escape') {
      cancelEdit();
    }
  }

  function clickOutside(node: HTMLElement) {
    const handleClick = (event: MouseEvent) => {
      if (!isZoomed) {
        if (isEditing && node && !node.contains(event.target as Node)) {
          saveEdit();
        }
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

        if (isSearching) {
          await getAllNotes();
        } else {
          await reloadNotes();
        }

        setStatus('Sub-notes reordered successfully');
        break;
      } catch (error) {
        console.error("Failed to reorder sub-notes:", error);

        if (attempt >= maxRetries) {
          if (isSearching) {
            await getAllNotes();
          } else {
            await reloadNotes();
          }
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
        innerNote.style.transformOrigin = 'center';

        const content: HTMLElement | null = innerNote.querySelector('.noteContent');
        if (content) content.style.display = 'none';
      }
    }
  }

  function applyFormat(command: string, value?: string) {
    const targetEditor = getActiveEditor();
    if (!targetEditor) return;

    switch (command) {
      case 'underline':
        targetEditor.chain().focus().toggleUnderline().run();
        break;
      case 'bold':
        targetEditor.chain().focus().toggleBold().run();
        break;
      case 'fontSize':
        if (value) {
          targetEditor.chain().focus().setFontSize(value).run();
        }
        break;
      case 'foreColor':
        if (value) {
          targetEditor.chain().focus().setColor(value).run();
        }
        break;
    }
  }

  const FontSize = Extension.create({
    name: 'fontSize',
    addOptions() {
      return {
        types: ['textStyle'],
      };
    },
    addGlobalAttributes() {
      return [
        {
          types: this.options.types,
          attributes: {
            fontSize: {
              default: null,
              parseHTML: element => element.style.fontSize.replace(/['"]+/g, ''),
              renderHTML: attributes => {
                if (!attributes.fontSize) return {};
                return { style: `font-size: ${attributes.fontSize}` };
              },
            },
          },
        },
      ];
    },
    addCommands() {
      return {
        setFontSize: fontSize => ({ chain }) => {
          return chain().setMark('textStyle', { fontSize }).run();
        },
        unsetFontSize: () => ({ chain }) => {
          return chain().setMark('textStyle', { fontSize: null }).removeEmptyTextStyle().run();
        },
      };
    },
  });

</script>

<div
  class="note"
  class:category={isCategory}
  class:editing={isEditing}
  class:zoomed={isZoomed}
  role="article"
  use:clickOutside
>
  <div id="noteTitleBox">
    <div id="dragZoomContainer">
      {#if !isCategory}
        <button class="zoomBtn" onclick={() => zoomedNote(note.id)} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing || isZoomed}>
          {#if isZoomed || isEditing}
            <img id="zoom-icon-disabled" src="zoom.svg" alt="ZoomIconDisabled">
          {:else}
            <img id="zoom-icon" src="zoom.svg" alt="ZoomIcon">
          {/if}
        </button>
        {#if isEditing}
          <div class="editorToolbar">
            <button onclick={() => applyFormat('underline')}>
              <img id="textUnderline-icon" src="underline.svg" alt="textUnderline">
            </button>
            <button onclick={() => applyFormat('bold')}>
              <img id="textBold-icon" src="boldtext.svg" alt="textBold">
            </button>
            <select
              onchange={(e) => {
                const target = e.target as HTMLSelectElement | null;
                if (target?.value) {
                  applyFormat('fontSize', target.value);
                }
              }}
            >
              <option value="">Font size</option>
              <option value="12px">Small</option>
              <option value="16px">Normal</option>
              <option value="20px">Large</option>
              <option value="36px">Huge</option>
            </select>
            <input
              type="color"
              id="colorSelectBar"
              onchange={(e) => {
                const target = e.target as HTMLInputElement | null;
                if (target?.value) {
                  applyFormat('foreColor', target.value);
                }
              }}
              title="Text color"
            />
          </div>
        {/if}
      {/if}
      <div class="spacer"></div>
      {#if !isZoomed}
        {#if !isEditing}
          {#if !isSearching}
            <div class="dragHandle" role="button" tabindex="0" ondblclick={e => { e.stopPropagation(); }} use:dragHandle>
              <img id="dragHandle-icon" src="drag-handle.svg" alt="dragHandleIcon">
            </div>
          {/if}
        {/if}
      {/if}
    </div>
    <div class="noteControls">
      {#if isCategory}
        <button onclick={addChild} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Add Note</button>
        <button onclick={OpenAllSubnotes} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Open all</button>
        <button onclick={CloseAllSubnotes} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Close all</button>
        <button onclick={startEdit} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Edit</button>
        <button onclick={removeNote} ondblclick={e => { e.stopPropagation(); }}>Delete</button>
      {:else if !isCategory}
        {#if note.parent_id !== null}
          <button onclick={toggle} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing || isZoomed}>{open ? 'Hide' : 'Show'}</button>
        {/if}
        <button onclick={startEdit} ondblclick={e => { e.stopPropagation(); }} disabled={isEditing}>Edit</button>
        <button onclick={removeNote} ondblclick={e => { e.stopPropagation(); }}>Delete</button>
      {/if}
    </div>
    {#if isEditing}
      <div class="noteControls">
        <button id="noteEditSaveBtn" onclick={saveEdit}>Save</button>
        <button id="noteEditCancelBtn" onclick={cancelEdit}>Cancel</button>
      </div>
      <div class="infoText">
        <small>Press Esc to cancel | Press Enter or click outside to save</small>
      </div>
      {#if titleEditor}
        <EditorContent
          editor={titleEditor}
          class="noteTitleEditable"
        />
      {/if}
    {:else}
      <h3 class="noteTitle" ondblclick={(e) => { if (isEditing) return; e.stopPropagation(); startEdit(); }}>{@html note.title || 'Untitled'}</h3>
    {/if}
  </div>

  <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
    <div id="noteContentOuter">
      {#if collapseOpen}
        <div style="overflow: hidden;" transition:slide={{ delay: 100, duration: 400, easing: cubicInOut }}>
          {#if isEditing}
            {#if !isCategory}
              {#if editor}
                <EditorContent
                  editor={editor}
                  class="noteContentEditable"
                />
              {/if}
              <div class="noteControls" style="margin-top: 10px;">
                <button id="noteEditSaveBtn" onclick={saveEdit}>Save</button>
                <button id="noteEditCancelBtn" onclick={cancelEdit}>Cancel</button>
              </div>
              <div class="infoText">
                <small>Press Esc to cancel | Press Enter or click outside to save</small>
              </div>
            {/if}
          {:else}
            {#if !isCategory}
              <p class="noteContent" ondblclick={(e) => { if (isEditing) return; e.stopPropagation(); startEdit(); }}>{@html note.content || 'No content'}</p>
            {:else if isCategory}
              <div class="subNotes" use:dragHandleZone={{
                items: previewChildNotes ?? childNotes,
                type: `child-note`,
                flipDurationMs: flipDurationMs,
                dropTargetStyle: {border: '1px solid #ccc'},
                transformDraggedElement: transformElement,
                morphDisabled: true,
                centreDraggedOnCursor: true }}
                onconsider={handleDnd}
                onfinalize={handleDndFinalize}
              >
                {#key childNotes.map(n => n.id).join('-')}
                  {#each (previewChildNotes ?? childNotes) as child (child.id)}
                    <div animate:flip={{ duration: flipDurationMs }} data-is-dnd-shadow-item-hint={(child as any)[SHADOW_ITEM_MARKER_PROPERTY_NAME] ?? false}>
                      <ComponentNote note={child} {reloadNotes} {setStatus} {notes} {noteOpenStates} {zoomedNote} zoomedNoteId={zoomedNoteId} {isSearching} {getAllNotes} />
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
