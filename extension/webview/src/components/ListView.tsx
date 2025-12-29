import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { Issue } from '../types';
import clsx from 'clsx';
import { useState } from 'react';

interface ListViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onBulkUpdate: (ids: string[], field: string, value: unknown) => void;
    onSelectIssue: (issue: Issue) => void;
}

const ROW_HEIGHT = 44;

const priorityColors: Record<number, string> = {
    1: 'text-priority-1 bg-priority-1/10 border-priority-1/20',
    2: 'text-priority-2 bg-priority-2/10 border-priority-2/20',
    3: 'text-priority-3 bg-priority-3/10 border-priority-3/20',
    4: 'text-priority-4 bg-priority-4/10 border-priority-4/20',
    5: 'text-priority-5 bg-priority-5/10 border-priority-5/20',
};

const statusColors: Record<string, string> = {
    open: 'bg-status-open/10 text-status-open border-status-open/20',
    'in-progress': 'bg-status-progress/10 text-status-progress border-status-progress/20',
    closed: 'bg-status-closed/10 text-status-closed border-status-closed/20',
    blocked: 'bg-status-blocked/10 text-status-blocked border-status-blocked/20',
};

export function ListView({ issues, onBulkUpdate, onSelectIssue }: ListViewProps) {
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

    const toggleSelection = (id: string) => {
        const newSet = new Set(selectedIds);
        if (newSet.has(id)) {
            newSet.delete(id);
        } else {
            newSet.add(id);
        }
        setSelectedIds(newSet);
    };

    const toggleAll = () => {
        if (selectedIds.size === issues.length && issues.length > 0) {
            setSelectedIds(new Set());
        } else {
            setSelectedIds(new Set(issues.map(i => i.id)));
        }
    };

    const handleBulkAction = (field: string, value: unknown) => {
        onBulkUpdate(Array.from(selectedIds), field, value);
        setSelectedIds(new Set());
    };

    const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
        const issue = issues[index];
        const isSelected = selectedIds.has(issue.id);

        return (
            <div
                className={clsx(
                    'flex items-center gap-4 px-6 border-b border-vscode-border/30 transition-colors cursor-pointer group',
                    index % 2 === 0 ? 'bg-vscode-bg' : 'bg-vscode-sidebar/10',
                    isSelected ? 'bg-vscode-accent/5' : 'hover:bg-vscode-accent/10'
                )}
                style={style}
                onClick={() => onSelectIssue(issue)}
            >
                <div className="w-6 flex shrink-0 items-center justify-center" onClick={(e) => e.stopPropagation()}>
                    <input
                        type="checkbox"
                        className="w-4 h-4 rounded border-vscode-border text-vscode-accent focus:ring-vscode-accent bg-vscode-bg cursor-pointer"
                        checked={isSelected}
                        onChange={() => toggleSelection(issue.id)}
                    />
                </div>

                <div className="w-20 shrink-0 text-[10px] font-mono text-vscode-fg/30 tracking-wider">
                    {issue.id.slice(0, 8)}
                </div>

                <div className="w-28 shrink-0">
                    <div className={clsx('px-2 py-0.5 rounded-md text-[10px] font-bold uppercase border w-fit', statusColors[issue.status])}>
                        {issue.status}
                    </div>
                </div>

                <div className="w-24 shrink-0">
                    <div className={clsx('px-2 py-0.5 rounded-md text-[10px] font-bold uppercase border w-fit', priorityColors[issue.priority])}>
                        P{issue.priority}
                    </div>
                </div>

                <div className="w-20 shrink-0 text-[11px] text-vscode-fg/40 font-medium truncate uppercase tracking-tighter">
                    {issue.issue_type}
                </div>

                <div className="flex-1 min-w-0 text-[13px] font-medium text-vscode-fg/90 truncate group-hover:text-vscode-accent transition-colors">
                    {issue.title}
                </div>

                <div className="w-24 shrink-0 text-[11px] text-vscode-fg/50 text-right font-medium italic">
                    {issue.assignee || 'Unassigned'}
                </div>
            </div>
        );
    };

    return (
        <div className="flex flex-col h-full bg-vscode-bg overflow-hidden relative select-none">
            {selectedIds.size > 0 && (
                <div className="absolute top-0 inset-x-0 z-50 h-10 bg-vscode-accent flex items-center justify-between px-6 shadow-lg animate-in slide-in-from-top duration-200">
                    <div className="flex items-center gap-4 text-white">
                        <span className="text-xs font-bold uppercase tracking-wider">{selectedIds.size} SELECTED</span>
                        <div className="h-4 w-px bg-white/30"></div>
                        <div className="flex gap-2">
                            <button
                                className="px-2 py-1 hover:bg-white/20 rounded text-[10px] font-bold uppercase transition-colors"
                                onClick={() => handleBulkAction('status', 'in-progress')}
                            >
                                Start Work
                            </button>
                            <button
                                className="px-2 py-1 hover:bg-white/20 rounded text-[10px] font-bold uppercase transition-colors"
                                onClick={() => handleBulkAction('status', 'closed')}
                            >
                                Resolve
                            </button>
                        </div>
                    </div>
                    <button
                        className="text-white hover:bg-white/20 p-1 rounded transition-colors"
                        onClick={() => setSelectedIds(new Set())}
                    >
                        ✕
                    </button>
                </div>
            )}

            <div className="flex items-center gap-4 px-6 h-10 border-b border-vscode-border bg-vscode-sidebar/30 shrink-0">
                <div className="w-6 flex shrink-0 items-center justify-center">
                    <input
                        type="checkbox"
                        className="w-4 h-4 rounded border-vscode-border text-vscode-accent focus:ring-vscode-accent bg-vscode-bg cursor-pointer"
                        checked={selectedIds.size === issues.length && issues.length > 0}
                        onChange={toggleAll}
                    />
                </div>
                <div className="w-20 shrink-0 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest">ID</div>
                <div className="w-28 shrink-0 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest">Status</div>
                <div className="w-24 shrink-0 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest">Priority</div>
                <div className="w-20 shrink-0 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest">Type</div>
                <div className="flex-1 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest">Title</div>
                <div className="w-24 shrink-0 text-[10px] font-bold text-vscode-fg/40 uppercase tracking-widest text-right">Assignee</div>
            </div>

            <div className="flex-1 min-h-0">
                <AutoSizer>
                    {({ height, width }) => (
                        <List
                            height={height}
                            itemCount={issues.length}
                            itemSize={ROW_HEIGHT}
                            width={width}
                        >
                            {Row}
                        </List>
                    )}
                </AutoSizer>
            </div>
        </div>
    );
}
