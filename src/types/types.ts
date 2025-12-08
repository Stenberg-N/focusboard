export type Note = {
  id: number;
  title: string;
  content: string;
  tab_id: number | null;
  parent_id: number | null;
  order_id: number | null;
  note_type: string;
  created_at: string;
  updated_at: string;
};

export type Tab = {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
}
