// Issue model matching Rust grits-core/src/models.rs
export interface Issue {
    id: string;
    content_hash: string;
    title: string;
    description: string;
    design: string;
    acceptance_criteria: string;
    notes: string;
    status: string;
    priority: number;
    issue_type: string;
    assignee: string | null;
    estimated_minutes: number | null;
    created_at: string;
    updated_at: string;
    closed_at: string | null;
    external_ref: string | null;
    sender: string;
    ephemeral: boolean;
    replies_to: string;
    relates_to: string[];
    duplicate_of: string;
    superseded_by: string;

    deleted_at: string | null;
    deleted_by: string;
    delete_reason: string;
    original_type: string;

    labels: string[];
    dependencies: Dependency[];
    comments: Comment[];
}

export interface Dependency {
    issue_id: string;
    depends_on_id: string;
    type_: string;
    created_at: string;
    created_by: string;
}

export interface Comment {
    id: number;
    issue_id: string;
    author: string;
    text: string;
    created_at: string;
}

// View types
export type ViewType = 'list' | 'kanban' | 'graph' | 'agenda';

// VS Code postMessage API
declare global {
    interface Window {
        vscode: {
            postMessage(message: unknown): void;
            getState(): unknown;
            setState(state: unknown): void;
        };
    }
}
