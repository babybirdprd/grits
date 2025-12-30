import { Issue } from '../types';
import clsx from 'clsx';

interface KanbanViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onSelectIssue: (issue: Issue) => void;
}

const COLUMNS = [
    { id: 'open', title: 'Open', color: 'bg-status-open', textColor: 'text-white' },
    { id: 'in-progress', title: 'In Progress', color: 'bg-status-progress', textColor: 'text-white' },
    { id: 'blocked', title: 'Blocked', color: 'bg-status-blocked', textColor: 'text-white' },
    { id: 'closed', title: 'Closed', color: 'bg-status-closed', textColor: 'text-vscode-fg/60' },
];

const priorityConfig: Record<number, { icon: string; color: string; label: string }> = {
    1: { icon: '↑', color: 'text-priority-1', label: 'Highest' },
    2: { icon: '↑', color: 'text-priority-2', label: 'High' },
    3: { icon: '—', color: 'text-priority-3', label: 'Medium' },
    4: { icon: '↓', color: 'text-priority-4', label: 'Low' },
    5: { icon: '↓', color: 'text-priority-5', label: 'Lowest' },
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
        <div className="flex gap-4 p-6 h-full overflow-x-auto select-none">
            {COLUMNS.map((column) => (
                <div
                    key={column.id}
                    className="flex flex-col w-80 shrink-0 bg-vscode-sidebar/30 rounded-xl border border-vscode-border/50 overflow-hidden"
                    onDragOver={handleDragOver}
                    onDrop={(e) => handleDrop(e, column.id)}
                >
                    <div className={clsx("flex items-center justify-between px-4 py-3 shrink-0", column.color, column.textColor)}>
                        <h3 className="text-xs font-bold uppercase tracking-widest">{column.title}</h3>
                        <span className="bg-white/20 px-2 py-0.5 rounded-full text-[10px] font-bold">
                            {issuesByStatus[column.id]?.length || 0}
                        </span>
                    </div>

                    <div className="flex-1 overflow-y-auto p-3 space-y-3 scrollbar-hide">
                        {issuesByStatus[column.id]?.map((issue) => {
                            const priority = priorityConfig[issue.priority] || priorityConfig[3];
                            return (
                                <div
                                    key={issue.id}
                                    className="group bg-vscode-bg border border-vscode-border rounded-lg p-3 hover:border-vscode-accent transition-all cursor-pointer shadow-sm hover:shadow-md active:scale-[0.98]"
                                    draggable
                                    onDragStart={(e) => handleDragStart(e, issue)}
                                    onClick={() => onSelectIssue(issue)}
                                >
                                    <div className="mb-2 text-[13px] font-medium leading-tight text-vscode-fg/90 group-hover:text-vscode-accent transition-colors">
                                        {issue.title}
                                    </div>

                                    <div className="flex flex-wrap gap-1.5 mb-3">
                                        {(issue.labels || []).slice(0, 3).map(label => (
                                            <span key={label} className="px-1.5 py-0.5 bg-vscode-accent/10 text-vscode-accent text-[9px] font-bold rounded-md border border-vscode-accent/20 uppercase">
                                                {label}
                                            </span>
                                        ))}
                                    </div>

                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <span className={clsx("text-xs font-bold flex items-center gap-0.5", priority.color)}>
                                                {priority.icon} <span className="text-[10px] hidden sm:inline">{priority.label}</span>
                                            </span>
                                            <span className="text-[10px] text-vscode-fg/30 font-mono">
                                                {issue.id.slice(0, 6)}
                                            </span>
                                        </div>

                                        {issue.assignee && (
                                            <div className="w-6 h-6 rounded-full bg-vscode-accent/20 border border-vscode-accent/30 flex items-center justify-center text-vscode-accent text-[10px] font-bold" title={issue.assignee}>
                                                {issue.assignee.charAt(0).toUpperCase()}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            ))}
        </div>
    );
}
