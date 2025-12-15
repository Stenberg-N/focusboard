import React, { createContext, useContext, useState, ReactNode, useMemo } from 'react';

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

interface AppContextType {
  notes: NoteType[];
  setNotes: React.Dispatch<React.SetStateAction<NoteType[]>>;
  tabs: TabType[];
  setTabs: React.Dispatch<React.SetStateAction<TabType[]>>;
  noteOpenStates: Record<number, Record<number, boolean>>;
  setNoteOpenStates: React.Dispatch<React.SetStateAction<Record<number, Record<number, boolean>>>>;
  uiVisibility: { tabBar: boolean };
  setUiVisibility: React.Dispatch<React.SetStateAction<{ tabBar: boolean }>>;
  topLevelNotes: NoteType[];
  childNotesByParent: Map<number | null, NoteType[]>;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const AppProvider = ({ children }: { children: ReactNode }) => {
  const [notes, setNotes] = useState<NoteType[]>([]);
  const [tabs, setTabs] = useState<TabType[]>([]);
  const [noteOpenStates, setNoteOpenStates] = useState<Record<number, Record<number, boolean>>>({});
  const [uiVisibility, setUiVisibility] = useState<{ tabBar: boolean }>({ tabBar: true });

  const topLevelNotes = useMemo(() => {
    return notes
      .filter(n => n.parent_id === null)
      .sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0));
  }, [notes]);

  const childNotesByParent = useMemo(() => {
    const map = new Map<number | null, NoteType[]>();

    for (const note of notes) {
      if (note.parent_id === null) continue;
      if (!map.has(note.parent_id)) map.set(note.parent_id, []);
      map.get(note.parent_id)!.push(note);
    }

    for (const list of map.values()) {
      list.sort((a, b) => (a.order_id ?? 0) - (b.order_id ?? 0));
    }

    return map;
  }, [notes]);

  return (
    <AppContext.Provider value = {{
      notes, setNotes,
      tabs, setTabs,
      noteOpenStates, setNoteOpenStates,
      uiVisibility, setUiVisibility,
      topLevelNotes, childNotesByParent
    }}>
      {children}
    </AppContext.Provider>
  );
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('AppContext not ready');
  }
  return context;
};
