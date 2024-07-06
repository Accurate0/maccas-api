export interface GetEventsResponse {
  active_events: Event[];
  historical_events: Event[];
}

export interface Event {
  name: string;
  id: number;
  event_id: string;
  data: EventType;
  is_completed: boolean;
  should_be_completed_at: string;
  status: "Completed" | "Failed" | "Pending" | "Running" | "Duplicate";
  created_at: string;
  updated_at: string;
  attempts: number;
  error: boolean;
  error_message?: string;
  completed_at?: string;
}
