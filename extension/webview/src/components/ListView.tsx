import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { Issue } from '../types';
import clsx from 'clsx';
import './ListView.css';

interface ListViewProps {
    issues: Issue[];
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onSelectIssue: (issue: Issue) => void;
}

const ROW_HEIGHT = 40;

const priorityColors: Record<number, string> = {
    1: 'priority-critical',
    2: 'priority-high',
    3: 'priority-medium',
    4: 'priority-low',
    5: 'priority-trivial',
};

const statusColors: Record<string, string> = {
    open: 'status-open',
    'in-progress': 'status-in-progress',
    closed: 'status-closed',
    blocked: 'status-blocked',
};

export function ListView({ issues, onUpdateField, onSelectIssue }: ListViewProps) {
    const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
        const issue = issues[index];

        return (
            <div
                className={clsx('list-row', index % 2 === 0 && 'list-row-even')}
                style={style}
                onClick={() => onSelectIssue(issue)}
            >
                <div className="cell cell-id" title={issue.id}>
                    {issue.id.slice(0, 8)}
                </div>
                <div className={clsx('cell cell-status', statusColors[issue.status])}>
                    <select
                        value={issue.status}
                        onChange={(e) => onUpdateField(issue.id, 'status', e.target.value)}
                        onClick={(e) => e.stopPropagation()}
                    >
                        <option value="open">Open</option>
                        <option value="in-progress">In Progress</option>
                        <option value="blocked">Blocked</option>
                        <option value="closed">Closed</option>
                    </select>
                </div>
                <div className={clsx('cell cell-priority', priorityColors[issue.priority])}>
                    <select
                        value={issue.priority}
                        onChange={(e) => onUpdateField(issue.id, 'priority', parseInt(e.target.value))}
                        onClick={(e) => e.stopPropagation()}
                    >
                        <option value={1}>P1 - Critical</option>
                        <option value={2}>P2 - High</option>
                        <option value={3}>P3 - Medium</option>
                        <option value={4}>P4 - Low</option>
                        <option value={5}>P5 - Trivial</option>
                    </select>
                </div>
                <div className="cell cell-type">{issue.issue_type}</div>
                <div className="cell cell-title" title={issue.title}>
                    {issue.title}
                </div>
                <div className="cell cell-assignee">
                    {issue.assignee || '-'}
                </div>
            </div>
        );
    };

    return (
        <div className="list-view">
            <div className="list-header">
                <div className="cell cell-id">ID</div>
                <div className="cell cell-status">Status</div>
                <div className="cell cell-priority">Priority</div>
                <div className="cell cell-type">Type</div>
                <div className="cell cell-title">Title</div>
                <div className="cell cell-assignee">Assignee</div>
            </div>
            <div className="list-body">
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
