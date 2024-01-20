export interface GetEventsResponse {
  active_events: Event[];
  historical_events: Event[];
}

export type EventType = Cleanup;

export interface Cleanup {
  offer_id: string;
  transaction_id: string;
  store_id: string;
}

export interface Event {
  name: string;
  id: number;
  event_id: string;
  data: EventType;
  is_completed: boolean;
  should_be_completed_at: string;
  created_at: string;
  updated_at: string;
  attempts: number;
  error: boolean;
  error_message?: string;
  completed_at?: string;
}
