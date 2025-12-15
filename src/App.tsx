import React, { useEffect, useRef, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import { appLogDir } from '@tauri-apps/api/path';
import { openPath } from '@tauri-apps/plugin-opener';
import { Store } from '@tauri-apps/plugin-store';
import { OverlayScrollbarsComponent } from 'overlayscrollbars-react';
import { motion, AnimatePresence } from 'framer-motion';
import { DndContext, closestCenter, KeyboardSensor, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { arrayMove, horizontalListSortingStrategy, SortableContext, sortableKeyboardCoordinates, rectSortingStrategy } from '@dnd-kit/sortable';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Menu, Item, useContextMenu } from 'react-contexify';

import { useApp } from './contexts/AppContext.tsx';
import { SortableNote } from './components/ComponentNote.tsx';

import "./App.css";
import 'react-contexify/dist/ReactContexify.css';

const MENU_ID = 'tab-context-menu';

interface TabType {
  id: number;
  name: string;
  order_id: number | null;
  created_at: string;
  updated_at: string;
}

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

interface SortableTabWrapperProps {
  tab: TabType;
  isSelected: boolean;
  onSelect: (tab: TabType) => void;
  onDoubleClick: (tab: TabType) => void;
  onContextMenu: (e: React.MouseEvent<HTMLDivElement>, tab: TabType) => void;
  editingTabId: number | null;
  editingTabName: string;
  setEditingTabId: (id: number | null) => void;
  setEditingTabName: (name: string) => void;
  saveRename: (tab: TabType) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
}

function SortableTabWrapper({
  tab,
  isSelected,
  onSelect,
  onDoubleClick,
  onContextMenu,
  editingTabId,
  editingTabName,
  setEditingTabId,
  setEditingTabName,
  saveRename,
  inputRef
}: SortableTabWrapperProps) {
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({ id: tab.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition
  };

  const isEditing = editingTabId === tab.id;

  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      onContextMenu={(e) => onContextMenu(e, tab)}
    >
      {isEditing ? (
        <input
          ref={inputRef}
          value={editingTabName}
          onChange={e => setEditingTabName(e.target.value)}
          onBlur={() => saveRename(tab)}
          onKeyDown={e => {
            if (e.key === 'Enter') e.preventDefault(), saveRename(tab);
            if (e.key === 'Escape') setEditingTabId(null);
          }}
        />
      ) : (
        <button
          className={`tab ${isSelected ? 'selected' : ''}`}
          onClick={() => onSelect(tab)}
          onDoubleClick={() => onDoubleClick(tab)}
        >
          {tab.name || 'Untitled'}
        </button>
      )}
    </div>
  );
}

export default function App() {
  const {
    notes: _, setNotes,
    tabs, setTabs,
    noteOpenStates, setNoteOpenStates,
    uiVisibility, setUiVisibility,
  } = useApp();

  const { topLevelNotes } = useApp();

  const [currentTabId, setCurrentTabId] = useState<number | null>(null);
  const [currentTabName, setCurrentTabName] = useState<string | null>(null);
  const [editingTabId, setEditingTabId] = useState<number | null>(null);
  const [editingTabName, setEditingTabName] = useState('');
  const [noteType, setNoteType] = useState('basic');
  const [statusMessage, setStatusMessage] = useState('');
  const [showOverlay, setShowOverlay] = useState(false);
  const [store, setStore] = useState<any>(null);

  const statusBarRef = useRef<HTMLSpanElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const { show } = useContextMenu({ id: MENU_ID });

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5} }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates
    })
  );

  const reloadNotes = useCallback(async () => {
    if (currentTabId !== null) {
      await loadNotes(currentTabId);
    }
  }, [currentTabId]);

  const setStatus = useCallback(
    (msg: string) => setStatusMessage(msg), []
  );

  useEffect(() => {
    async function init() {
      const store = await Store.load('ui-state.json');
      setStore(store);

      let currentTabs: TabType[] = await invoke('get_tabs');
      console.log('Fetched tabs from DB:', currentTabs.map(t => ({ id: t.id, name: t.name, order: t.order_id })));
      currentTabs = [...currentTabs].sort((a, b) => (a.order_id ?? Infinity) - (b.order_id ?? Infinity));

      if (currentTabs.length === 0) {
        const newTab: TabType = await invoke('create_tab', { name: 'Untitled' });
        console.log('Created initial tab:', newTab.id);
        currentTabs = [newTab];
        setCurrentTabId(newTab.id);
        setCurrentTabName(newTab.name);
      }

      setTabs(currentTabs);

      const savedTabId = await store.get<number>('currentTabId') ?? null;
      const savedTabName = await store.get<string>('currentTabName') ?? null;
      const savedOpenStates = await store.get<Record<number, Record<number, boolean>>>('noteOpenStates') ?? {};

      let selectedTabId: number | null = savedTabId;
      let selectedTabName: string | null = savedTabName;

      if (selectedTabId && currentTabs.some(t => t.id === selectedTabId && t.name === selectedTabName)) {
        setCurrentTabId(selectedTabId);
        setCurrentTabName(selectedTabName);

        try {
          const data: NoteType[] = await invoke('get_notes', { tabId: selectedTabId });
          setNotes(data);

          console.log('Loaded notes for tab', selectedTabId, ':', data.length);

        } catch (error) {
          console.error("Failed to fetch notes:", error);
        }

      }

      if (savedOpenStates) {
        try {
          setNoteOpenStates(savedOpenStates);
        } catch (error) {
          console.error("Failed to fetch note states:", error);
        }
      }

      setStatusMessage('App setup complete');
    }

    init();
  }, []);

  async function saveNoteOpenStates(states: Record<number, Record<number, boolean>>) {
    await store.set('noteOpenStates', states);
    await store.save();
  }

  useEffect(() => {
    if (!store) return;
    const t = setTimeout(() => {
      saveNoteOpenStates(noteOpenStates);
    }, 300);
    return () => clearTimeout(t);
  }, [noteOpenStates]);

  useEffect(() => {
    if (store && currentTabId !== null) {
      store.set('currentTabId', currentTabId);
      store.set('currentTabName', currentTabName);
      store.save();
      console.log("Saved currentTabId to store:", currentTabId);
    }
  }, [currentTabId, currentTabName, store]);

  useEffect(() => {
    const unlisten = listen('app-closing', async () => {
      setShowOverlay(true);

      if (store) {
        store.set('currentTabId', currentTabId);
        store.set('currentTabName', currentTabName);
        store.set('noteOpenStates', noteOpenStates);
        await store.save();
      }

    });

    return () => { unlisten.then(f => f()); };
  }, [currentTabId, currentTabName, noteOpenStates, store]);

  useEffect(() => {
    if (currentTabId !== null) {
      loadNotes(currentTabId);
    } else {
      setNotes([]);
    }
  }, [currentTabId]);

  const openLogs = async () => {
    const logDir = await appLogDir();
    await openPath(logDir);
  };

  const backupDatabase = async () => {
    try {
      await invoke('backup_database');
      setStatusMessage('Backup successful');

    } catch (error) {
      console.error("Database backup failed:", error);
      setStatusMessage(`Failed to backup database: ${error}`);
    }

  };

  const loadNotes = async (tabId: number | null) => {
    if (!tabId) {
      setNotes([]);
      return;
    }

    try {
      const data: NoteType[] = await invoke('get_notes', { tabId });
      setNotes(data);

      setStatusMessage(`Loaded notes for tab ${currentTabName}`);

    } catch (error) {
      console.error("get_notes failed:", error);
      setStatusMessage(`Failed to load notes: ${error}`);
    }

  };

  const loadTabs = async () => {
    try {
      const data: TabType[] = await invoke('get_tabs');
      const sorted = [...data].sort((a, b) => (a.order_id ?? Infinity) - (b.order_id ?? Infinity));
      setTabs(sorted);

      console.log('Loaded tabs:', sorted.map(t => ({ id: t.id, order: t.order_id })));

    } catch (error) {
      console.error("get_tabs failed:", error);
      setStatusMessage(`Failed to load tabs: ${error}`);
    }

  };

  const addNote = async () => {
    try {
      await invoke('create_note', { title: 'Untitled', content: '', tabId: currentTabId, parentId: null, noteType: noteType });

      await loadNotes(currentTabId);

      setStatusMessage('Created note successfully');

    } catch (error) {
      console.error("create_note failed:", error);
      setStatusMessage(`Failed to create note: ${error}`);
    }

  };

  const addTab = async () => {
    try {
      const newTab: TabType = await invoke('create_tab', { name: 'New Tab' });
      await loadTabs();

      setCurrentTabId(newTab.id);
      setCurrentTabName(newTab.name);

      setStatusMessage(`Added tab ${currentTabName} successfully`);

    } catch (error) {
      console.error("'create_tab failed:", error);
      setStatusMessage(`Failed to create tab: ${error}`);
    }

  };

  const handleTabContext = (e: React.MouseEvent, tab: TabType) => {
    e.preventDefault();
    show({ event: e, props: { tab } });
  };

  const handleDeleteTab = async (args: { props?: { tab: TabType } }) => {
    if (!args.props?.tab) return;

    const tab = args.props.tab;

    try {
      await invoke('delete_tab', { id: tab.id });
      const newTabs: TabType[] = await invoke('get_tabs');
      setTabs(newTabs);

      if (currentTabId === tab.id && newTabs.length > 0) {
        setCurrentTabId(newTabs[0].id);
        setCurrentTabName(newTabs[0].name);
        console.log(`Switched to tab ${newTabs[0].name} after deletion`)
      } else if (newTabs.length === 0) {
        setCurrentTabId(null);
        setCurrentTabName(null);
      }
      await loadNotes(currentTabId);

      setStatusMessage(`Deleted tab ${tab.name} successfully`);

    } catch (error) {
      console.error("delete_tab failed:", error);
      setStatusMessage(`Failed to delete tab: ${error}`);
    }

  };

  const startRename = (tab: TabType) => {
    setEditingTabId(tab.id);
    setEditingTabName(tab.name);
    setTimeout(() => inputRef.current?.focus(), 0);
  };

  const saveRename = async (tab: TabType) => {
    if (editingTabName.trim() === '') return;
    try {
      await invoke('update_tab', { id: tab.id, name: editingTabName });
      setTabs(prev => prev.map(t => t.id === tab.id ? { ...t, name: editingTabName } : t));

      setEditingTabId(null);
      setCurrentTabName(editingTabName);

      setStatusMessage(`Updated tab name to ${editingTabName} successfully`);

    } catch (error) {
      console.error("update_tab failed:", error);
      setStatusMessage(`Failed to update tab: ${error}`);
    }

  };

  const handleDragEndNotes = async (event: any) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = topLevelNotes.findIndex(n => n.id === active.id);
    const newIndex = topLevelNotes.findIndex(n => n.id === over.id);
    const newOrder = arrayMove(topLevelNotes, oldIndex, newIndex);

    const ids = newOrder.map(n => n.id);

    setNotes((prevNotes: NoteType[]) => {
      const orderMap = new Map(newOrder.map((note, index) => [note.id, index + 1]));

      return prevNotes.map((note) => {
        if (note.parent_id === null && orderMap.has(note.id)) {
          const newOrderId = orderMap.get(note.id);
          return { ...note, order_id: newOrderId ?? null };
        }
        return note;
      });
    });

    try {
      await invoke('reorder_notes', { tabId: currentTabId, noteIds: ids });
      setStatusMessage('Notes reordered successfully');

    } catch (error) {
      console.error("reorder_notes failed:", error);
      await loadNotes(currentTabId);

      setStatusMessage(`Failed to reorder notes: ${error}`);
    }

  };

  const handleDragEndTabs = async (event: any) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = tabs.findIndex(t => t.id === active.id);
    const newIndex = tabs.findIndex(t => t.id === over.id);
    const newOrder = arrayMove(tabs, oldIndex, newIndex);

    const ids = newOrder.map(t => t.id);

    setTabs(newOrder.map((tab, index) => ({ ...tab, order_id: index })));

    try {
      await invoke('reorder_tabs', { tabIds: ids });
      setStatusMessage('Tabs reordered successfully');

    } catch (error) {
      console.error("reorder_tabs failed:", error);
      await loadTabs();

      setStatusMessage(`Failed to reorder tabs: ${error}`);
    }

  };

  const tabBarIsOpen = uiVisibility.tabBar ?? true;

  return (
    <main className="container">
      {showOverlay && (
        <div className="loader-overlay">
          <div className="loader-content">
            <p className="loader-message">Closing App...</p>
            <p className="loader-message">Optimizing database, please wait</p>
            <div className="loader"></div>
          </div>
        </div>
      )}

      <div id="menuBar">
        <small>Create Note in Current Tab</small>
        <select value={noteType} onChange={(e) => setNoteType(e.target.value)}>
          <option value="basic">Basic</option>
          <option value="categorical">Categorical</option>
        </select>
        <button onClick={addNote} disabled={!currentTabId}>Create Note</button>
        <button onClick={openLogs}>Open logs</button>
        <button onClick={backupDatabase}>Backup Database</button>
      </div>

      <div id="middle" className={!tabBarIsOpen ? 'enlarged' : ''}>
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move', autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: 'hidden' } }}>
          <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEndNotes}>
            <SortableContext items={topLevelNotes.map(n => n.id)} strategy={rectSortingStrategy}>
              <div id="noteContainer">
                {currentTabId ? topLevelNotes.map(note => (
                  <SortableNote key={note.id} note={note} reloadNotes={reloadNotes} setStatus={setStatus} />
                )) : <p>No tabs available.</p>}
              </div>
            </SortableContext>
          </DndContext>
        </OverlayScrollbarsComponent>
      </div>

      <AnimatePresence>
        {tabBarIsOpen && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: '30px' }}
            exit={{ height: 0 }}
            transition={{ duration: 0.2, ease: 'easeInOut' }}
            id="tabBar"
          >
            <button id="buttonAddTab" onClick={addTab}>Add Tab</button>
            <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEndTabs}>
              <SortableContext items={tabs.map(t => t.id)} strategy={horizontalListSortingStrategy}>
                <div id="tabList">
                  {tabs.map(tab => (
                    <SortableTabWrapper
                      key={tab.id}
                      tab={tab}
                      isSelected={currentTabId === tab.id}
                      onSelect={() => {
                        console.log("selected tab:", tab.id);
                        setCurrentTabId(tab.id);
                        setCurrentTabName(tab.name);
                      }}
                      onDoubleClick={() => startRename(tab)}
                      onContextMenu={handleTabContext}
                      editingTabId={editingTabId}
                      editingTabName={editingTabName}
                      setEditingTabId={setEditingTabId}
                      setEditingTabName={setEditingTabName}
                      saveRename={saveRename}
                      inputRef={inputRef}
                    />
                  ))}
                </div>
              </SortableContext>
            </DndContext>
          </motion.div>
        )}
      </AnimatePresence>

      <div id="statusBar">
        <span ref={statusBarRef}>{statusMessage}</span>
        <button id="tabBarToggle" onClick={() => setUiVisibility(prev => ({ ...prev, tabBar: !tabBarIsOpen }))}>
          {tabBarIsOpen ? 'v' : '^'}
        </button>
      </div>

      <Menu id={MENU_ID}>
        <Item onClick={handleDeleteTab}>Remove Tab</Item>
      </Menu>
    </main>
  );

}
