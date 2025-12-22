import React, { useState } from 'react';
import { Issue } from '../types';
import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import './ListView.css';

interface ListViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: any) => void;
    onSelectIssue: (issue: Issue | null) => void;
}

export function ListView({ issues, onUpdateField, onSelectIssue }: ListViewProps) {
    const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

    const toggleSelection = (id: string, multi: boolean) => {
        const newSelected = new Set(multi ? selectedIds : []);
        if (newSelected.has(id)) {
            newSelected.delete(id);
        } else {
            newSelected.add(id);
        }
        setSelectedIds(newSelected);

        // If single select, also notify parent
        if (!multi && newSelected.size === 1) {
             const issue = issues.find(i => i.id === id);
             onSelectIssue(issue || null);
        } else if (!multi && newSelected.size === 0) {
            onSelectIssue(null);
        }
    };

    const handleBulkAction = (action: string, value: any) => {
        selectedIds.forEach(id => {
            onUpdateField(id, action, value);
        });
        setSelectedIds(new Set());
    };

    const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
        const issue = issues[index];
        const isSelected = selectedIds.has(issue.id);

        return (
            <div
                className={`list-row ${isSelected ? 'selected' : ''}`}
                style={style}
                onClick={(e) => toggleSelection(issue.id, e.ctrlKey || e.metaKey)}
            >
                <div className="col-check">
                    <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleSelection(issue.id, true)}
                        onClick={(e) => e.stopPropagation()}
                    />
                </div>
                <div className="col-id" title={issue.id}>{issue.id.substring(0, 6)}</div>
                <div className="col-status">
                    <select
                        value={issue.status}
                        onChange={(e) => onUpdateField(issue.id, 'status', e.target.value)}
                        onClick={(e) => e.stopPropagation()}
                    >
                        <option value="open">Open</option>
                        <option value="in_progress">In Progress</option>
                        <option value="blocked">Blocked</option>
                        <option value="closed">Closed</option>
                    </select>
                </div>
                <div className="col-priority">
                    P{issue.priority}
                </div>
                <div className="col-title" title={issue.title}>
                    {issue.title}
                    {issue.labels.map(l => <span key={l} className="mini-label">{l}</span>)}
                </div>
                <div className="col-assignee">{issue.assignee || '-'}</div>
            </div>
        );
    };

    return (
        <div className="list-view">
            {selectedIds.size > 0 && (
                <div className="bulk-actions">
                    <span>{selectedIds.size} selected</span>
                    <button onClick={() => handleBulkAction('status', 'closed')}>Close Selected</button>
                    <button onClick={() => handleBulkAction('status', 'open')}>Reopen Selected</button>
                    <button onClick={() => setSelectedIds(new Set())}>Clear Selection</button>
                </div>
            )}

            <div className="list-header">
                <div className="col-check"></div>
                <div className="col-id">ID</div>
                <div className="col-status">Status</div>
                <div className="col-priority">Pri</div>
                <div className="col-title">Title</div>
                <div className="col-assignee">Assignee</div>
            </div>

            <div className="list-body">
                <AutoSizer>
                    {({ height, width }) => (
                        <List
                            height={height}
                            itemCount={issues.length}
                            itemSize={40}
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
