import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { OverlayScrollbarsComponent } from 'overlayscrollbars-react';
import { AnimatePresence, motion } from 'framer-motion';
import { DndContext, closestCenter, KeyboardSensor, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { arrayMove, SortableContext, sortableKeyboardCoordinates, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

import { useApp } from '../contexts/AppContext';

interface NoteType {
  id: number;
  title: string;
  content: string;
  tab_id: number | null;
  parent_id: number | null;
  order_id: number | null;
  note_type: string;
  created_at: string;
  updated_at: string;
}

function useOnClickOutside(ref: React.RefObject<HTMLDivElement>, handler: (event: MouseEvent) => void) {
  useEffect(() => {
    const listener = (event: MouseEvent) => {
      if (!ref.current || ref.current.contains(event.target as Node)) {
        return;
      }
      handler(event);
    };
    document.addEventListener('mousedown', listener);
    return () => {
      document.removeEventListener('mousedown', listener);
    };
  }, [ref, handler]);
}

interface SortableNoteProps {
  note: NoteType;
  reloadNotes: () => Promise<void>;
  setStatus: (message: string) => void;
}

export const SortableNote = React.memo(function SortableNote({ note, reloadNotes, setStatus }: SortableNoteProps) {

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: note.id, transition: { duration: 200, easing: 'ease' } });

  const style = {
    transform: transform ? CSS.Transform.toString({ x: transform.x, y: transform.y, scaleX: 1, scaleY: 1}) : undefined,
    transition,
    outline: isDragging ? '#723fffd0 solid 2px' : 'none',
    zIndex: isDragging ? 1000 : 'auto',
  };

  return (
    <div ref={setNodeRef} style={style} {...attributes}>
      <Note note={note} reloadNotes={reloadNotes} setStatus={setStatus} dragHandleProps={listeners} />
    </div>
  );
});

interface NoteProps {
  note: NoteType;
  reloadNotes: () => Promise<void>;
  setStatus?: (message: string) => void;
  dragHandleProps?: any;
}

const Note = ({ note, dragHandleProps, reloadNotes, setStatus = () => {} }: NoteProps) => {
  const { notes, setNotes, childNotesByParent, noteOpenStates, setNoteOpenStates } = useApp();

  const [isEditing, setIsEditing] = useState(false);
  const [editingTitle, setEditingTitle] = useState(note.title);
  const [editingContent, setEditingContent] = useState(note.content);
  const [originalOpenState, setOriginalOpenState] = useState(false);
  const [hasSavedState, setHasSavedState] = useState(false);

  const open = noteOpenStates[note.tab_id ?? -1]?.[note.id] ?? true;
  const isCategory = note.note_type === 'categorical';
  const isOpen = isEditing || open;

  const childNotes = childNotesByParent.get(note.id) ?? [];

  useEffect(() => {
    if (!note.tab_id) return;

    setNoteOpenStates(prev => {
      const tabState = prev[note.tab_id!] ?? {}
      const validIds = new Set(notes.map(n => n.id));

      const cleaned = Object.fromEntries(
        Object.entries(tabState).filter(([id]) => validIds.has(Number(id)))
      );

      return {
        ...prev,
        [note.tab_id!]: cleaned
      };
    });
  }, [notes]);

  const editRef = useRef<HTMLDivElement>(null);
  useOnClickOutside(editRef as React.RefObject<HTMLDivElement>, () => {
    if (isEditing) saveEdit();
  });

  const handleDoubleClick = () => {
    setOriginalOpenState(open);
    setHasSavedState(true);
    setIsEditing(true);
    setEditingTitle(note.title);
    setEditingContent(note.content);
  };

  const toggle = () => {
    setNoteOpenStates((prev: Record<number, Record<number, boolean>>) => ({ ...prev, [note.tab_id!]: { ...(prev[note.tab_id!] ?? {}), [note.id]: !open }}));
  };

  const addChild = async () => {
    try {
      const newNote: NoteType = await invoke('create_note', { title: 'Untitled', content: '', tabId: note.tab_id, noteType: 'basic', parentId: note.id });
      setNotes((prev: NoteType[]) => [ ...prev, newNote]);
      setNoteOpenStates((prev: Record<number, Record<number, boolean>>) => ({ ...prev, [note.tab_id!]: { ...(prev[note.tab_id!] ?? {}), [note.id]: true }}));

      setStatus('Added sub-note successfully');
    } catch (error) {
      console.error(error);
      setStatus(`Failed to add sub-note: ${error}`);
    }
  };

  const saveEdit = async () => {
    try {
      await invoke('update_note', { id: note.id, title: editingTitle, content: editingContent || '' });
      setIsEditing(false);
      setNoteOpenStates((prev: Record<number, Record<number, boolean>>) => ({ ...prev, [note.tab_id!]: { ...(prev[note.tab_id!] ?? {}), [note.id]: true }}));

      setNotes(prev => prev.map(n => n.id === note.id ? { ...n, title: editingTitle, content: editingContent, updated_at: new Date().toISOString() } : n));
      setStatus(`Updated note ${note.title} successfully`);
    } catch (error) {
      console.error(error);
      setStatus(`Failed to update ${note.title} note: ${error}`);
    }
  };

  const cancelEdit = () => {
    setIsEditing(false);
    if (hasSavedState) {
      setNoteOpenStates((prev: Record<number, Record<number, boolean>>) => ({ ...prev, [note.tab_id!]: { ...(prev[note.tab_id!] ?? {}), [note.id]: originalOpenState }}));
      setHasSavedState(false);
    }
  };

  const removeNote = async () => {
    try {
      await invoke('delete_note', { id: note.id });
      await reloadNotes();
      setStatus(`Deleted note ${note.title} successfully`);
    } catch(error) {
      console.error(error);
      setStatus(`Failed to delete ${note.title} successfully`);
    }
  };

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragEnd = async (event: any) => {
    const { active, over } = event;
    if (active.id !== over?.id) {
      const oldIndex = childNotes.findIndex((n) => n.id === active.id);
      const newIndex = childNotes.findIndex((n) => n.id === over.id);
      const newOrder = arrayMove(childNotes, oldIndex, newIndex);
      setNotes((prev: NoteType[]) => {
        const updated = prev.map((n) => {
          if (n.parent_id === note.id) {
            const found = newOrder.find((c) => c.id === n.id);
            if (found) {
              return { ...n, order_id: newOrder.indexOf(found) + 1 };
            }
          }
          return n;
        });
        return updated;
      });

      const orderedIds = newOrder.map((n) => n.id);
      try {
        await invoke('reorder_notes', { noteIds: orderedIds });
        setStatus('Sub-notes reordered successfully');
      } catch (error) {
        console.error(error)
        setStatus(`Failed to reorder sub-notes: ${error}`);
        await reloadNotes();
      }
    }
  };

  return (
    <div ref={editRef} className={`note ${isCategory ? 'category' : ''} ${isEditing ? 'editing' : ''}`} onDoubleClick={(e) => { e.stopPropagation(); handleDoubleClick(); }} role="article">
      <div id="noteTitleBox">
        <div className="noteDragHandle" {...dragHandleProps}>
          Drag
        </div>
        <small>Last edited: {note.updated_at}</small>
        <small>Note ID: {note.id}</small>
        <small>Tab ID: {note.tab_id}</small>
        <small>Order ID: {note.order_id}</small>
        <div id="noteControls">
          {isCategory ? (
            <>
              <button onClick={toggle} disabled={isEditing}>
                {open ? 'Hide' : 'Show'}
              </button>
              <button onClick={addChild}>Add Child</button>
              <button onClick={() => setIsEditing(true)} disabled={isEditing}>Edit</button>
              <button onClick={removeNote}>Delete</button>
            </>
          ) : (
            <>
              <button onClick={toggle} disabled={isEditing}>
                {open ? 'Hide' : 'Show'}
              </button>
              <button onClick={() => setIsEditing(true)} disabled={isEditing}>Edit</button>
              <button onClick={removeNote}>Delete</button>
            </>
          )}
        </div>
        {isEditing ? (
          <input
            value={editingTitle}
            onChange={(e) => setEditingTitle(e.target.value)}
            placeholder="Title | Enter to save"
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                saveEdit();
              } else if (e.key === 'Escape') {
                cancelEdit();
              }
            }}
          />
        ) : (
          <h3 className="noteTitle">{note.title || 'Untitled'}</h3>
        )}
      </div>

      <OverlayScrollbarsComponent options={{ scrollbars: { autoHide: 'move', autoHideDelay: 800, theme: 'os-theme-dark' }, overflow: { x: 'hidden' } }}>
        <div id="noteContentOuter">
          <AnimatePresence>
            {isOpen && (
              <motion.div
                initial={{ height: 0 }}
                animate={{ height: 'auto' }}
                exit={{ height: 0 }}
                transition={{ duration: 0.4, ease: 'easeInOut' }}
                style={{ overflow: 'hidden' }}
              >
                {isEditing ? (
                  <>
                    {!isCategory && (
                      <textarea
                        value={editingContent}
                        onChange={(e) => setEditingContent(e.target.value)}
                        placeholder="Enter to save | Shift+Enter for new line | Esc to cancel"
                        rows={16}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' && !e.shiftKey) {
                            e.preventDefault();
                            saveEdit();
                          } else if (e.key === 'Escape') {
                            cancelEdit();
                          }
                        }}
                      />
                    )}
                    <div className="edit-actions">
                      <button onClick={saveEdit}>Save</button>
                      <button onClick={cancelEdit}>Cancel</button>
                      <small>Press Esc to cancel | Press Enter or click outside to save</small>
                    </div>
                  </>
                ) : (
                  <>
                    {!isCategory && <p className="noteContent">{note.content || 'No content'}</p>}
                    {isCategory && (
                      <DndContext
                        sensors={sensors}
                        collisionDetection={closestCenter}
                        onDragEnd={handleDragEnd}
                      >
                        <SortableContext items={childNotes.map((n) => n.id)} strategy={verticalListSortingStrategy}>
                          <div className="subNotes">
                            {childNotes.map((child) => (
                              <SortableNote key={child.id} note={child} reloadNotes={reloadNotes} setStatus={setStatus} />
                            ))}
                          </div>
                        </SortableContext>
                      </DndContext>
                    )}
                  </>
                )}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </OverlayScrollbarsComponent>
    </div>
  );
};

export default React.memo(Note);
