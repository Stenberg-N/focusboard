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
  order_id: number;
  created_at: string;
  updated_at: string;
};

export type Timer = {
  id: number;
  initial_duration: number;
  duration: number;
  message: string | null;
}

export type CalendarDay = {
  date: Date;
  isodate: string;
  name: string;
  enabled: boolean;
  monthabbrev: string;
}

export type CalendarEvent = {
  id: number;
  event_date: string;
  event_name: string;
  event_start: number;
  event_end: number;
}
