<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { dragHandle } from 'svelte-dnd-action';
  import { Editor, EditorContent } from 'svelte-tiptap';
  import StarterKit from '@tiptap/starter-kit';
  import { TextStyle } from '@tiptap/extension-text-style';
  import Color from '@tiptap/extension-color';
  import { Extension } from '@tiptap/core';
  import HardBreak from '@tiptap/extension-hard-break';
  import { getContext } from 'svelte';

  import type { Note } from '../types/types';
  import 'overlayscrollbars/overlayscrollbars.css';
  import '../routes/style.css';

  const { setDeleteNoteId } = getContext<{ getDeleteNoteId: () => number | null, setDeleteNoteId: (id: number | null) => void }>('deleteNoteContext');

  let {
    note,
    reloadNotes,
    setStatus,
    zoomedNote,
    zoomedNoteId = null,
    isSearching = false,
  }: {
    note: Note;
    reloadNotes: () => void;
    setStatus: (msg: string) => void;
    zoomedNote: (id: number) => void;
    zoomedNoteId: number | null;
    isSearching: boolean;
  } = $props();

  let isEditing = $state<boolean>(false);
  let editingTitle = $state('');
  let editingContent = $state('');
  let editor = $state<Editor | null>(null);
  let titleEditor = $state<Editor | null>(null);
  let SelectedContentType = $state<'noteTitle' | 'noteContent' | null>(null);

  let isZoomed = $derived(zoomedNoteId === note.id);
  let noteMaxHeight = $state<number>(0);

  const editorButtons = [
    { name: 'underline', img: 'underline.svg' },
    { name: 'bold', img: 'boldtext.svg' },
    { name: 'italic', img: 'italic.svg' },
  ]

  $effect(() => {
    if (isEditing) {
      if (!editor) {
        editor = new Editor({
          extensions: [
            StarterKit,
            TextStyle,
            Color,
            FontSize,
            HardBreak.extend({
              addKeyboardShortcuts() {
                return {
                  Enter: () => this.editor.commands.setHardBreak(),
                };
              },
            }),
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

  function startEdit() {
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

      await reloadNotes();

      const plainTitle = stripHtml(editingTitle) || 'Untitled';
      setStatus(`Updated note ${plainTitle} successfully`);
    } catch (error) {
      console.error('update_note failed:', error);
      setStatus(`Failed to update note: ${error}`);
    }
  }

  function cancelEdit() {
    isEditing = false;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
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
      case 'italic':
        targetEditor.chain().focus().toggleItalic().run();
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
  class:editing={isEditing}
  class:zoomed={isZoomed}
  role="article"
  use:clickOutside
>
  <div id="noteTitleBox">
    <div class="noteControls">
      {#if !isEditing && !isZoomed}
        <button class="zoomBtn" onclick={() => zoomedNote(note.id)} ondblclick={e => { e.stopPropagation(); }}>
          <img id="zoom-icon" src="zoom.svg" alt="ZoomIcon">
        </button>
      {/if}
      {#if isEditing}
        <button class="primary-button" onclick={saveEdit}><img src="save.svg" alt="Save" style="max-height: 20px; max-width: 20px;"></button>
        <button class="primary-button" onclick={cancelEdit}><img src="cancel.svg" alt="Cancel" style="max-height: 20px; max-width: 20px;"></button>
      {/if}
      {#if !isEditing}
        <button class="primary-button" onclick={startEdit} ondblclick={e => { e.stopPropagation(); }}><img src="edit-pencil.svg" alt="Edit icon" style="max-height: 22px; max-width: 22px;"></button>
        <button class="primary-button" onclick={() => setDeleteNoteId(note.id)} ondblclick={e => { e.stopPropagation(); }}><img src="trash-can.svg" alt="Trash can" style="max-height: 20px; max-width: 20px;"></button>
      {/if}
      <div class="spacer"></div>
      {#if isEditing}
        <div class="editorToolbar">
          {#each editorButtons as button}
            <button class="primary-button" onclick={() => applyFormat(button.name)}>
              <img class="editorIcons" src={button.img} alt="symbol">
            </button>
          {/each}
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
    {#if isEditing}
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
        <small>Press Esc to cancel</small>
      </div>
    {:else}
      <p class="noteContent" ondblclick={(e) => { if (isEditing) return; e.stopPropagation(); startEdit(); }}>{@html note.content || 'No content'}</p>
    {/if}
  </div>
</div>

<style>

.note {
  display: flex;
  flex-direction: column;
  background-color: #222;
  border-radius: 8px;
  padding: 18px 4px;
  min-height: 100%;
  height: 100%;
  min-width: 312px;
  width: 100%;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
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
  height: calc(100% - 24px);
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
  flex: 1 1 0;
  align-items: center;
  justify-content: flex-end;
  gap: 5px;
}

.editorToolbar button {
  height: 30px;
  width: 30px;
  padding: 6px;
  background-color: #333;
}

.editorIcons {
  width: 18px;
  height: 18px;
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
  min-width: 40px;
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
