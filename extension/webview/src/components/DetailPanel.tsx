import React, { useState, useMemo } from 'react';
import { Issue } from '../types';
import './DetailPanel.css';
import { format } from 'date-fns';

interface DetailPanelProps {
    issue: Issue;
    onClose: () => void;
    onUpdate: (issue: Issue) => void;
    onAddComment: (issueId: string, author: string, text: string) => void;
    onAddLabel: (issueId: string, label: string) => void;
    onRemoveLabel: (issueId: string, label: string) => void;
    allIssues: Issue[];
}

export function DetailPanel({ issue, onClose, onUpdate, onAddComment, onAddLabel, onRemoveLabel, allIssues }: DetailPanelProps) {
    const [newComment, setNewComment] = useState('');
    const [newLabel, setNewLabel] = useState('');
    const [isLabelPickerOpen, setIsLabelPickerOpen] = useState(false);

    const availableLabels = useMemo(() => {
        const labels = new Set<string>();
        allIssues.forEach(i => i.labels.forEach(l => labels.add(l)));
        return Array.from(labels).sort();
    }, [allIssues]);

    const handleAddComment = () => {
        if (!newComment.trim()) return;
        onAddComment(issue.id, 'me', newComment); // TODO: Get author from config
        setNewComment('');
    };

    const handleAddLabel = (label: string) => {
        if (issue.labels.includes(label)) return;
        onAddLabel(issue.id, label);
        setNewLabel('');
        setIsLabelPickerOpen(false);
    };

    const handleRemoveLabel = (label: string) => {
        onRemoveLabel(issue.id, label);
    };

    const handleStatusChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        onUpdate({ ...issue, status: e.target.value });
    };

    const handlePriorityChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        onUpdate({ ...issue, priority: parseInt(e.target.value) });
    };

    return (
        <aside className="detail-panel">
            <div className="detail-header">
                <h2>{issue.title}</h2>
                <button className="close-btn" onClick={onClose}>✕</button>
            </div>

            <div className="detail-body">
                <div className="meta-grid">
                    <div className="detail-field">
                        <label>Status</label>
                        <select value={issue.status} onChange={handleStatusChange}>
                            <option value="open">Open</option>
                            <option value="in_progress">In Progress</option>
                            <option value="blocked">Blocked</option>
                            <option value="closed">Closed</option>
                        </select>
                    </div>
                    <div className="detail-field">
                        <label>Priority</label>
                        <select value={issue.priority} onChange={handlePriorityChange}>
                            <option value={1}>P1 - Critical</option>
                            <option value={2}>P2 - High</option>
                            <option value={3}>P3 - Normal</option>
                            <option value={4}>P4 - Low</option>
                            <option value={5}>P5 - Trivial</option>
                        </select>
                    </div>
                </div>

                <div className="detail-field">
                    <label>Labels</label>
                    <div className="labels-list">
                        {issue.labels.map(label => (
                            <span key={label} className="label-tag">
                                {label}
                                <button onClick={() => handleRemoveLabel(label)}>×</button>
                            </span>
                        ))}
                        <button className="add-label-btn" onClick={() => setIsLabelPickerOpen(!isLabelPickerOpen)}>
                            +
                        </button>
                    </div>
                    {isLabelPickerOpen && (
                        <div className="label-picker">
                            <input
                                type="text"
                                placeholder="New label..."
                                value={newLabel}
                                onChange={(e) => setNewLabel(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' && newLabel) {
                                        handleAddLabel(newLabel);
                                    }
                                }}
                            />
                            <div className="existing-labels">
                                {availableLabels.map(l => (
                                    <div key={l} onClick={() => handleAddLabel(l)}>{l}</div>
                                ))}
                            </div>
                        </div>
                    )}
                </div>

                <div className="detail-field full-width">
                    <label>Description</label>
                    <textarea
                        className="description-editor"
                        value={issue.description}
                        onChange={(e) => onUpdate({...issue, description: e.target.value})}
                    />
                </div>

                <div className="detail-section">
                    <h3>Comments</h3>
                    <div className="comments-list">
                        {issue.comments.map((comment, idx) => (
                            <div key={idx} className="comment">
                                <div className="comment-header">
                                    <span className="author">{comment.author}</span>
                                    <span className="date">{format(new Date(comment.created_at), 'MMM d, HH:mm')}</span>
                                </div>
                                <div className="comment-text">{comment.text}</div>
                            </div>
                        ))}
                    </div>
                    <div className="new-comment">
                        <textarea
                            placeholder="Add a comment..."
                            value={newComment}
                            onChange={(e) => setNewComment(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                                    handleAddComment();
                                }
                            }}
                        />
                        <button onClick={handleAddComment} disabled={!newComment.trim()}>Comment</button>
                    </div>
                </div>

                <div className="detail-section">
                     <h3>Dependencies</h3>
                     <ul className="dependency-list">
                        {issue.dependencies.map((dep, idx) => (
                            <li key={idx}>
                                {dep.type_ === 'blocking' ? '🚫 Blocks ' : '🔗 Related to '}
                                {dep.depends_on_id}
                            </li>
                        ))}
                     </ul>
                </div>
            </div>
        </aside>
    );
}
