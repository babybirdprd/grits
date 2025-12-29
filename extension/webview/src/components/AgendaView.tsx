import { Issue } from '../types';

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

    const IssueCard = ({ issue, showActions = true }: { issue: Issue; showActions?: boolean }) => {
        const priorityColors = {
            1: 'border-red-500/50 bg-red-500/10',
            2: 'border-orange-500/50 bg-orange-500/10',
            3: 'border-yellow-500/50 bg-yellow-500/10',
            4: 'border-green-500/50 bg-green-500/10',
            5: 'border-blue-500/50 bg-blue-500/10',
        }[issue.priority as 1 | 2 | 3 | 4 | 5] || 'border-vscode-border bg-vscode-sidebar/50';

        return (
            <div
                className={`group p-4 rounded-xl border transition-all hover:scale-[1.02] hover:shadow-xl cursor-pointer ${priorityColors}`}
                onClick={() => onSelectIssue(issue)}
            >
                <div className="flex justify-between items-start mb-2">
                    <span className="text-xs font-mono opacity-50 uppercase tracking-tight">gr-{issue.id.slice(0, 6)}</span>
                    <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider ${issue.priority <= 2 ? 'bg-red-500 text-white' : 'bg-vscode-bg/50'
                        }`}>
                        P{issue.priority}
                    </span>
                </div>
                <div className="text-sm font-semibold mb-3 line-clamp-2 leading-tight">{issue.title}</div>
                <div className="flex items-center gap-3 text-[11px] opacity-70">
                    <span className="px-1.5 py-0.5 bg-vscode-bg/30 rounded border border-vscode-border/30">{issue.issue_type}</span>
                    {issue.assignee && (
                        <span className="flex items-center gap-1">
                            <span className="opacity-50">👤</span>
                            {issue.assignee}
                        </span>
                    )}
                </div>
                {showActions && (
                    <div className="mt-4 flex gap-2">
                        {issue.status === 'blocked' && (
                            <button
                                className="flex-1 py-1.5 bg-green-600 hover:bg-green-500 text-white rounded-lg text-xs font-medium transition-colors"
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
                                className="flex-1 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors"
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
                                className="flex-1 py-1.5 bg-vscode-accent hover:opacity-90 text-white rounded-lg text-xs font-medium transition-colors"
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
    };

    return (
        <div className="p-8 max-w-7xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="mb-10 text-center">
                <h2 className="text-4xl font-extrabold tracking-tight mb-2 flex items-center justify-center gap-3">
                    <span className="animate-pulse">🎯</span> Focus
                </h2>
                <p className="text-vscode-fg/60 text-lg">Current critical path and active tasks</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                {/* BLOCKED - Most important for PM */}
                <section className={`flex flex-col gap-4 p-6 rounded-2xl border bg-vscode-sidebar/30 backdrop-blur-md transition-opacity ${blocked.length === 0 ? 'opacity-30 grayscale' : 'border-red-500/30'}`}>
                    <div className="flex justify-between items-center">
                        <h3 className="text-lg font-bold flex items-center gap-2">
                            <span className="text-red-500">🚫</span> Blocked
                        </h3>
                        <span className="px-2 py-1 bg-red-500 text-white text-[10px] font-bold rounded-full">{blocked.length}</span>
                    </div>
                    <p className="text-xs opacity-50 -mt-2">Waiting for resolution</p>
                    <div className="flex flex-col gap-4">
                        {blocked.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>

                {/* IN PROGRESS */}
                <section className={`flex flex-col gap-4 p-6 rounded-2xl border bg-vscode-sidebar/30 backdrop-blur-md transition-opacity ${inProgress.length === 0 ? 'opacity-30 grayscale' : 'border-blue-500/30'}`}>
                    <div className="flex justify-between items-center">
                        <h3 className="text-lg font-bold flex items-center gap-2">
                            <span className="text-blue-500">🔄</span> In Progress
                        </h3>
                        <span className="px-2 py-1 bg-blue-500 text-white text-[10px] font-bold rounded-full">{inProgress.length}</span>
                    </div>
                    <p className="text-xs opacity-50 -mt-2">Active development</p>
                    <div className="flex flex-col gap-4">
                        {inProgress.map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>

                {/* HIGH PRIORITY READY TO START */}
                <section className={`flex flex-col gap-4 p-6 rounded-2xl border bg-vscode-sidebar/30 backdrop-blur-md transition-opacity ${highPriority.length === 0 ? 'opacity-30 grayscale' : 'border-vscode-accent/30'}`}>
                    <div className="flex justify-between items-center">
                        <h3 className="text-lg font-bold flex items-center gap-2">
                            <span className="text-orange-500">🔥</span> Priority
                        </h3>
                        <span className="px-2 py-1 bg-orange-500 text-white text-[10px] font-bold rounded-full">{highPriority.length}</span>
                    </div>
                    <p className="text-xs opacity-50 -mt-2">Recommended next tasks</p>
                    <div className="flex flex-col gap-4">
                        {highPriority.slice(0, 5).map((i) => (
                            <IssueCard key={i.id} issue={i} />
                        ))}
                    </div>
                </section>
            </div>

            {/* ALL CLEAR */}
            {blocked.length === 0 && inProgress.length === 0 && highPriority.length === 0 && (
                <div className="mt-20 py-20 text-center rounded-3xl border-2 border-dashed border-vscode-border/50 animate-in zoom-in-95 duration-500">
                    <div className="text-6xl mb-6 drop-shadow-lg">🎉</div>
                    <h3 className="text-3xl font-bold mb-2">All Clear!</h3>
                    <p className="text-vscode-fg/50 max-w-sm mx-auto">No blocked issues or urgent work. Your team is moving fast!</p>
                </div>
            )}
        </div>
    );
}

