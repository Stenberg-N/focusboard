<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
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
  import { getContext } from 'svelte';

  import type { Note } from '../types/types';
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/style.css';

  const { setDeleteNoteId } = getContext<{ getDeleteNoteId: () => number | null, setDeleteNoteId: (id: number | null) => void }>('deleteNoteContext');

  let {
    note,
    reloadNotes,
    setStatus,
    currentTabNotes,
    noteOpenStates = $bindable<Record<number, boolean>>(),
    zoomedNote,
    zoomedNoteId = null,
    isSearching = false,
    getAllNotes,
    noteHeightMultiplier,
    windowHeight,
  }: {
    note: Note;
    reloadNotes: () => void;
    setStatus: (msg: string) => void;
    currentTabNotes: Note[];
    noteOpenStates: Record<number, boolean>;
    zoomedNote: (id: number) => void;
    zoomedNoteId: number | null;
    isSearching: boolean;
    getAllNotes: () => Promise<void>;
    noteHeightMultiplier: "larger" | "smaller" | null;
    windowHeight: number;
  } = $props();

  let childNotes = $derived.by(() => { return currentTabNotes.filter(n => n.parent_id === note.id).sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0)) });
  let previewChildNotes = $state<Note[] | null>(null);

  let isEditing = $state<boolean>(false);
  let editingTitle = $state('');
  let editingContent = $state('');
  let editor = $state<Editor | null>(null);
  let titleEditor = $state<Editor | null>(null);
  let SelectedContentType = $state<'noteTitle' | 'noteContent' | null>(null);

  const isCategory = $derived(note.note_type === 'categorical');

  let originalOpenState = false;
  let hasSavedState = false;

  let isZoomed = $derived(zoomedNoteId === note.id);
  let open = $derived(noteOpenStates[note.id] ?? true);
  let collapseOpen = $derived(isEditing || open);
  let noteMaxHeight = $state<number>(0);
  let noteContentHeight = $state<number>(0);

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

  $effect(() => {
    if (noteHeightMultiplier == "smaller") noteMaxHeight = (windowHeight - 150) / 2, noteContentHeight = (windowHeight - 444) / 2;
    else if (noteHeightMultiplier == "larger") noteMaxHeight = (windowHeight - 140), noteContentHeight = windowHeight - 287;
    if (isZoomed) noteMaxHeight = windowHeight - 140, noteContentHeight = windowHeight - 420;
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

    newItems.forEach((item, index) => {
      item.order_id = index + 1;
    });
    childNotes = [...newItems];

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

          setStatus(`Failed to reorder sub-notes: ${error}`);
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
  class="note" style="max-height: {noteMaxHeight}px;"
  class:category={isCategory}
  class:editing={isEditing}
  class:zoomed={isZoomed}
  role="article"
  use:clickOutside
>
  <div id="noteTitleBox">
    <div class="noteControls">
      {#if !isCategory && !isEditing && !isZoomed}
        <button class="zoomBtn" onclick={() => zoomedNote(note.id)} ondblclick={e => { e.stopPropagation(); }}>
          <img id="zoom-icon" src="zoom.svg" alt="ZoomIcon">
        </button>
      {/if}
      {#if isEditing}
        <button class="primary-button" onclick={saveEdit}><img src="save.svg" alt="Save" style="max-height: 20px; max-width: 20px;"></button>
        <button class="primary-button" onclick={cancelEdit}><img src="cancel.svg" alt="Cancel" style="max-height: 20px; max-width: 20px;"></button>
      {/if}
      {#if isCategory && !isEditing}
        <button class="primary-button" onclick={addChild} ondblclick={e => { e.stopPropagation(); }}><img src="close.svg" alt="Add note" style="transform: rotate(45deg); max-height: 16px; max-width: 16px;"></button>
        <button class="primary-button" onclick={startEdit} ondblclick={e => { e.stopPropagation(); }}><img src="edit-pencil.svg" alt="Edit icon" style="max-height: 22px; max-width: 22px;"></button>
        <button class="primary-button" onclick={OpenAllSubnotes} ondblclick={e => { e.stopPropagation(); }}><img src="show-eye.svg" alt="Show notes" style="max-height: 24px; max-width: 24px;"></button>
        <button class="primary-button" onclick={CloseAllSubnotes} ondblclick={e => { e.stopPropagation(); }}><img src="hide-eye.svg" alt="Hide notes" style="max-height: 24px; max-width: 24px;"></button>
        <button class="primary-button" onclick={() => setDeleteNoteId(note.id)} ondblclick={e => { e.stopPropagation(); }}><img src="trash-can.svg" alt="Trash can" style="max-height: 20px; max-width: 20px;"></button>
      {:else if !isCategory && !isEditing}
        {#if note.parent_id !== null}
          <button class="primary-button" onclick={toggle} ondblclick={e => { e.stopPropagation(); }}>
            {#if open}
              <img src="hide-eye.svg" alt="Eye icon" style="max-height: 24px; max-width: 24px;">
            {:else}
              <img src="show-eye.svg" alt="Eye icon" style="max-height: 24px; max-width: 24px;">
            {/if}
          </button>
        {/if}
        <button class="primary-button" onclick={startEdit} ondblclick={e => { e.stopPropagation(); }}><img src="edit-pencil.svg" alt="Edit icon" style="max-height: 22px; max-width: 22px;"></button>
        <button class="primary-button" onclick={() => setDeleteNoteId(note.id)} ondblclick={e => { e.stopPropagation(); }}><img src="trash-can.svg" alt="Trash can" style="max-height: 20px; max-width: 20px;"></button>
      {/if}
      <div class="spacer"></div>
      {#if isEditing}
        <div class="editorToolbar">
          <button class="primary-button" onclick={() => applyFormat('underline')}>
            <img id="textUnderline-icon" src="underline.svg" alt="textUnderline">
          </button>
          <button class="primary-button" onclick={() => applyFormat('bold')}>
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
    {#if isEditing && titleEditor}
      <EditorContent
        editor={titleEditor}
        class="noteTitleEditable"
      />
    {:else}
      <h3 class="noteTitle" ondblclick={(e) => { if (isEditing) return; e.stopPropagation(); startEdit(); }}><p>{@html note.title || 'Untitled'}</p></h3>
    {/if}
  </div>

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
              <button class="primary-button" onclick={saveEdit}><img src="save.svg" alt="Save" style="max-height: 20px; max-width: 20px;"></button>
              <button class="primary-button" onclick={cancelEdit}><img src="cancel.svg" alt="Cancel" style="max-height: 20px; max-width: 20px;"></button>
            </div>
            <div class="infoText">
              <small>Press Esc to cancel | Press Enter or click outside to save</small>
            </div>
          {/if}
        {:else}
          {#if !isCategory}
            <p class="noteContent" style="min-height: {noteContentHeight}px" ondblclick={(e) => { if (isEditing) return; e.stopPropagation(); startEdit(); }}>{@html note.content || 'No content'}</p>
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
                    <ComponentNote note={child} {reloadNotes} {setStatus} {currentTabNotes} {noteOpenStates} {zoomedNote} zoomedNoteId={zoomedNoteId} {isSearching} {getAllNotes} {noteHeightMultiplier} {windowHeight} />
                  </div>
                {/each}
              {/key}
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>

.note {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  background-color: #222;
  border-radius: 8px;
  padding: 18px 4px;
  height: 100%;
  min-width: calc((100vw - 155px) / 4);
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
}

.note.category {
  align-items: center;
  max-height: calc((100vh - 151px) / 2);
  height: 100%;
  min-width: calc((100vw - 125px) / 4);
  width: 100%;
}

.noteTitle {
  margin: 0;
  width: fit-content;
  align-self: center;
  font-size: 20px;
}

.note:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

.noteContent {
  flex: 1 1 auto;
  overflow-y: auto;
  margin: 12px 0;
  text-align: left;
  white-space: pre-wrap;
  word-break: break-word;
  word-wrap: break-word;
  border: 1px solid #444;
  border-radius: 8px;
  padding: 6px;
}

#noteContentOuter {
  height: 100%;
  padding: 0 12px;
  margin-left: 6px;
  scrollbar-gutter: stable;
  overflow-y: auto;
}

#noteTitleBox {
  display: flex;
  flex-direction: column;
  width: 100%;
  justify-items: center;
  border-bottom: 1px solid #444;
}

#noteTitleBox input {
  max-width: 250px;
  width: 100%;
  height: 24px;
  margin-bottom: 10px;
  align-self: center;
  text-align: center;
  font-size: 18px;
  font-weight: 700;
  color: #f6f6f6;
  background-color: transparent;
  border: 1px solid #888;
  border-radius: 6px;
}

#noteTitleBox input:focus {
  border-color: #723fffd0;
  outline: none;
}

.noteControls {
  display: flex;
  flex-direction: row;
  flex: 1 1 0;
  justify-content: center;
  height: 32px;
  gap: 5px;
  margin: 0 10px 10px;
}

.noteControls button {
  height: 32px;
  width: 32px;
  background-color: #333;
  justify-content: center;
  align-items: center;
}

.noteControls button img {
  filter: brightness(0) invert(0.7);
}

.noteControls button:hover {
  background-color: #444;
}

.subNotes {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(381px, 100vw));
  min-height: 50px;
  gap: 16px;
  margin-top: 16px;
  padding: 12px;
  background-color: #151515;
  border-radius: 8px;
}

.dragHandle {
  display: flex;
  align-self: flex-end;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  margin-right: 4px;
  background-color: transparent;
  user-select: none;
}

.dragHandle #dragHandle-icon {
  margin: 0;
  width: 20px;
  height: 20px;
  filter: invert(0.7);
}

.noteControls .zoomBtn {
  display: flex;
  align-content: center;
  width: 30px;
  height: 30px;
  border: 0;
  background-color: transparent;
  padding: 0;
  user-select: none;
}

.noteControls .zoomBtn:hover {
  background-color: transparent;
}

.zoomBtn #zoom-icon {
  width: 30px;
  height: 30px;
  filter: invert(0.7);
}

.zoomBtn #zoom-icon:hover {
  filter: invert(0.9);
  cursor: pointer;
}

.editorToolbar {
  display: flex;
  flex-direction: row;
  width: 100%;
  align-items: center;
  justify-content: flex-end;
  gap: 5px;
}

.editorToolbar button {
  height: 30px;
  width: 30px;
  background-color: #333;
}

.editorToolbar #textUnderline-icon, .editorToolbar #textBold-icon {
  width: 22px;
  height: 22px;
  filter: brightness(1) invert(0.7);
}

.editorToolbar button:hover {
  background-color: #444;
}

.editorToolbar select {
  height: 30px;
  background-color: #333;
  margin-left: 0;
  color: #f6f6f6;
  border: none;
  outline: none;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: box-shadow 0.2s;
}

.editorToolbar select:hover {
  cursor: pointer;
  background-color: #444;
  box-shadow: 0 4px 12px rgba(0,0,0,1);
}

.editorToolbar select option {
  background-color: #333;
}

.editorToolbar #colorSelectBar {
  min-width: 20px;
  max-width: 100px;
  width: 100%;
  margin: 0;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
}

.editorToolbar #colorSelectBar:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
  cursor: pointer;
}

.infoText {
  display: flex;
  flex-direction: column;
  opacity: 0.5;
}

</style>
