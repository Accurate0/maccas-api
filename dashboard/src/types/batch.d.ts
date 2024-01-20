export interface GetJobsResponse {
  current_jobs: CurrentJob[];
  history: JobHistory[];
  task_queue: TaskQueue[];
}

export interface CurrentJob {
  name: string;
  state: "Stopped" | "Running";
}

export interface JobHistory {
  job_name: number;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  error: boolean;
  context: any;
  error_message: string | null;
}

export interface TaskQueue {
  seconds_until_next: number;
  name: string;
}
