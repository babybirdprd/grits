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
}: AgendaViewProps) {
    // Filter to actionable items
    const openIssues = issues.filter((i) => i.status !== 'closed');
    const blocked = openIssues.filter((i) => i.status === 'blocked');
    const inProgress = openIssues.filter((i) => i.status === 'in-progress');
    const highPriority = openIssues.filter((i) => i.priority <= 2 && i.status === 'open');

    const IssueCard = ({ issue, showActions = true }: { issue: Issue; showActions?: boolean }) => (
        <div
            className={`focus-card priority-${issue.priority} status-${issue.status}`}
            onClick={() => onSelectIssue(issue)}
        >
            <div className="focus-card-header">
                <span className="card-id">gr-{issue.id.slice(0, 6)}</span>
                <span className={`priority-badge p${issue.priority}`}>
                    P{issue.priority}
                </span>
            </div>
            <div className="focus-card-title">{issue.title}</div>
            <div className="focus-card-meta">
                <span className="issue-type">{issue.issue_type}</span>
                {issue.assignee && (
                    <span className="assignee">
                        <span className="assignee-icon">👤</span>
                        {issue.assignee}
                    </span>
                )}
            </div>
            {showActions && (
                <div className="focus-card-actions">
                    {issue.status === 'blocked' && (
                        <button
                            className="action-btn unblock"
                            onClick={(e) => {
                                e.stopPropagation();
                                onUpdateField(issue.id, 'status', 'open');
                            }}
                        >
                            ✓ Unblock
                        </button>
                    )}
                    {issue.status === 'open' && (
                        <button
                            className="action-btn start"
                            onClick={(e) => {
                                e.stopPropagation();
                                onUpdateField(issue.id, 'status', 'in-progress');
                            }}
                        >
                            ▶ Start
                        </button>
                    )}
                    {issue.status === 'in-progress' && (
                        <button
                            className="action-btn complete"
                            onClick={(e) => {
                                e.stopPropagation();
                                onUpdateField(issue.id, 'status', 'closed');
                            }}
                        >
                            ✓ Complete
                        </button>
                    )}
                </div>
            )}
        </div>
    );

    return (
        <div className="focus-view">
            <div className="focus-header">
                <h2>🎯 Focus</h2>
                <p className="focus-subtitle">What needs your attention right now</p>
            </div>

            {/* BLOCKED - Most important for PM */}
            {blocked.length > 0 && (
                <section className="focus-section blocked-section">
                    <div className="section-header">
                        <h3>🚫 Blocked</h3>
                        <span className="section-count">{blocked.length} waiting on you</span>
                    </div>
                    <p className="section-hint">These issues need PM action to continue</p>
                    <div className="focus-cards">
                        {blocked.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {/* IN PROGRESS */}
            {inProgress.length > 0 && (
                <section className="focus-section progress-section">
                    <div className="section-header">
                        <h3>🔄 In Progress</h3>
                        <span className="section-count">{inProgress.length} active</span>
                    </div>
                    <div className="focus-cards">
                        {inProgress.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {/* HIGH PRIORITY READY TO START */}
            {highPriority.length > 0 && (
                <section className="focus-section priority-section">
                    <div className="section-header">
                        <h3>🔥 High Priority</h3>
                        <span className="section-count">{highPriority.length} ready</span>
                    </div>
                    <p className="section-hint">Assign these to an agent next</p>
                    <div className="focus-cards">
                        {highPriority.slice(0, 5).map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            )}

            {/* ALL CLEAR */}
            {blocked.length === 0 && inProgress.length === 0 && highPriority.length === 0 && (
                <div className="focus-empty">
                    <span className="empty-icon">🎉</span>
                    <h3>All Clear!</h3>
                    <p>No blocked issues or urgent work. Great job!</p>
                </div>
            )}
        </div>
    );
}

