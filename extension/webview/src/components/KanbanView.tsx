import { Issue } from '../types';
import clsx from 'clsx';
import './KanbanView.css';

interface KanbanViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onSelectIssue: (issue: Issue) => void;
}

const COLUMNS = [
    { id: 'open', title: 'Open', color: '#f0ad4e' },
    { id: 'in-progress', title: 'In Progress', color: '#0d6efd' },
    { id: 'blocked', title: 'Blocked', color: '#dc3545' },
    { id: 'closed', title: 'Closed', color: '#198754' },
];

const priorityLabels: Record<number, string> = {
    1: 'P1',
    2: 'P2',
    3: 'P3',
    4: 'P4',
    5: 'P5',
};

export function KanbanView({ issues, onUpdateField, onSelectIssue }: KanbanViewProps) {
    const issuesByStatus = COLUMNS.reduce((acc, col) => {
        acc[col.id] = issues.filter((i) => i.status === col.id);
        return acc;
    }, {} as Record<string, Issue[]>);

    const handleDragStart = (e: React.DragEvent, issue: Issue) => {
        e.dataTransfer.setData('text/plain', issue.id);
        e.dataTransfer.effectAllowed = 'move';
    };

    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = 'move';
    };

    const handleDrop = (e: React.DragEvent, targetStatus: string) => {
        e.preventDefault();
        const issueId = e.dataTransfer.getData('text/plain');
        if (issueId) {
            onUpdateField(issueId, 'status', targetStatus);
        }
    };

    return (
        <div className="kanban-view">
            {COLUMNS.map((column) => (
                <div
                    key={column.id}
                    className="kanban-column"
                    onDragOver={handleDragOver}
                    onDrop={(e) => handleDrop(e, column.id)}
                >
                    <div
                        className="kanban-column-header"
                        style={{ borderTopColor: column.color }}
                    >
                        <span className="column-title">{column.title}</span>
                        <span className="column-count">
                            {issuesByStatus[column.id]?.length || 0}
                        </span>
                    </div>
                    <div className="kanban-column-body">
                        {issuesByStatus[column.id]?.map((issue) => (
                            <div
                                key={issue.id}
                                className={clsx('kanban-card', `priority-${issue.priority}`)}
                                draggable
                                onDragStart={(e) => handleDragStart(e, issue)}
                                onClick={() => onSelectIssue(issue)}
                            >
                                <div className="card-header">
                                    <span className="card-id">{issue.id.slice(0, 8)}</span>
                                    <span className="card-priority">
                                        {priorityLabels[issue.priority]}
                                    </span>
                                </div>
                                <div className="card-title">{issue.title}</div>
                                <div className="card-footer">
                                    <span className="card-type">{issue.issue_type}</span>
                                    {issue.assignee && (
                                        <span className="card-assignee">{issue.assignee}</span>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            ))}
        </div>
    );
}
