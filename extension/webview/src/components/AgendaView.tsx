import { Issue } from '../types';
import './AgendaView.css';

interface AgendaViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onSelectIssue: (issue: Issue) => void;
    currentUser?: string;
}

export function AgendaView({
    issues,
    onUpdateField,
    onSelectIssue,
    currentUser,
}: AgendaViewProps) {
    // Filter to actionable items: open/in-progress, assigned to user or unassigned, high priority
    const focusedIssues = issues
        .filter((i) => i.status !== 'closed')
        .filter((i) => !currentUser || !i.assignee || i.assignee === currentUser)
        .sort((a, b) => a.priority - b.priority);

    const highPriority = focusedIssues.filter((i) => i.priority <= 2);
    const inProgress = focusedIssues.filter((i) => i.status === 'in-progress');
    const blocked = focusedIssues.filter((i) => i.status === 'blocked');

    const IssueCard = ({ issue }: { issue: Issue }) => (
        <div
            className={`agenda-card priority-${issue.priority}`}
            onClick={() => onSelectIssue(issue)}
        >
            <div className="agenda-card-header">
                <span className="card-id">{issue.id.slice(0, 8)}</span>
                <span className={`status-badge status-${issue.status}`}>
                    {issue.status}
                </span>
            </div>
            <div className="agenda-card-title">{issue.title}</div>
            <div className="agenda-card-meta">
                <span>{issue.issue_type}</span>
                {issue.assignee && <span>• {issue.assignee}</span>}
            </div>
            <div className="agenda-card-actions">
                {issue.status === 'open' && (
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            onUpdateField(issue.id, 'status', 'in-progress');
                        }}
                    >
                        Start
                    </button>
                )}
                {issue.status === 'in-progress' && (
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            onUpdateField(issue.id, 'status', 'closed');
                        }}
                    >
                        Complete
                    </button>
                )}
            </div>
        </div>
    );

    return (
        <div className="agenda-view">
            <h2 className="agenda-title">🎯 Focus Mode</h2>

            {blocked.length > 0 && (
                <section className="agenda-section blocked-section">
                    <h3>🚫 Blocked ({blocked.length})</h3>
                    <div className="agenda-cards">
                        {blocked.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {inProgress.length > 0 && (
                <section className="agenda-section in-progress-section">
                    <h3>🔄 In Progress ({inProgress.length})</h3>
                    <div className="agenda-cards">
                        {inProgress.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {highPriority.length > 0 && (
                <section className="agenda-section priority-section">
                    <h3>🔥 High Priority ({highPriority.length})</h3>
                    <div className="agenda-cards">
                        {highPriority.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {focusedIssues.length === 0 && (
                <div className="agenda-empty">
                    <p>🎉 All caught up! No urgent issues.</p>
                </div>
            )}
        </div>
    );
}
