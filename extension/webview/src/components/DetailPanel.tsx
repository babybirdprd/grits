import React, { useState, useEffect, useRef } from 'react';
import { Issue, Comment } from '../types';
import { LabelPicker } from './LabelPicker';
import './DetailPanel.css';

interface DetailPanelProps {
    issue: Issue;
    onClose: () => void;
    onUpdateField: (id: string, field: string, value: unknown) => void;
    onAddComment: (id: string, text: string) => void;
    allLabels: string[];
    onAddLabel: (id: string, label: string) => void;
    onRemoveLabel: (id: string, label: string) => void;
}

export const DetailPanel: React.FC<DetailPanelProps> = ({
    issue,
    onClose,
    onUpdateField,
    onAddComment,
    allLabels,
    onAddLabel,
    onRemoveLabel,
}) => {
    const [commentText, setCommentText] = useState('');
    const [isEditingTitle, setIsEditingTitle] = useState(false);
    const [titleValue, setTitleValue] = useState(issue.title);
    const titleInputRef = useRef<HTMLInputElement>(null);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (isEditingTitle) {
                    setIsEditingTitle(false);
                    setTitleValue(issue.title);
                } else {
                    onClose();
                }
            }
        };
        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [onClose, isEditingTitle, issue.title]);

    // Sync title when issue changes
    useEffect(() => {
        setTitleValue(issue.title);
    }, [issue.title]);

    // Focus title input when editing
    useEffect(() => {
        if (isEditingTitle && titleInputRef.current) {
            titleInputRef.current.focus();
            titleInputRef.current.select();
        }
    }, [isEditingTitle]);

    const handleTitleSubmit = () => {
        if (titleValue.trim() && titleValue !== issue.title) {
            onUpdateField(issue.id, 'title', titleValue.trim());
        }
        setIsEditingTitle(false);
    };

    const handleCommentSubmit = () => {
        if (commentText.trim()) {
            onAddComment(issue.id, commentText);
            setCommentText('');
        }
    };

    return (
        <aside className="detail-panel">
            <div className="detail-header">
                {isEditingTitle ? (
                    <input
                        ref={titleInputRef}
                        className="title-input"
                        value={titleValue}
                        onChange={(e) => setTitleValue(e.target.value)}
                        onBlur={handleTitleSubmit}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') handleTitleSubmit();
                        }}
                    />
                ) : (
                    <h2
                        className="editable-title"
                        onClick={() => setIsEditingTitle(true)}
                        title="Click to edit"
                    >
                        {issue.title}
                    </h2>
                )}
                <button className="close-btn" onClick={onClose} title="Close (Esc)">✕</button>
            </div>
            <div className="detail-body">
                <div className="detail-field">
                    <label>ID</label>
                    <span className="id-value">{issue.id}</span>
                </div>

                <div className="detail-field">
                    <label>Status</label>
                    <select
                        value={issue.status}
                        onChange={(e) => onUpdateField(issue.id, 'status', e.target.value)}
                    >
                        <option value="open">Open</option>
                        <option value="in_progress">In Progress</option>
                        <option value="blocked">Blocked</option>
                        <option value="closed">Closed</option>
                    </select>
                </div>

                <div className="detail-field">
                    <label>Priority</label>
                    <input
                        type="number"
                        min="1" max="5"
                        value={issue.priority}
                        onChange={(e) => onUpdateField(issue.id, 'priority', parseInt(e.target.value))}
                    />
                </div>

                <div className="detail-field">
                    <label>Type</label>
                    <select
                        value={issue.issue_type}
                        onChange={(e) => onUpdateField(issue.id, 'issue_type', e.target.value)}
                    >
                        <option value="bug">Bug</option>
                        <option value="feature">Feature</option>
                        <option value="task">Task</option>
                        <option value="epic">Epic</option>
                    </select>
                </div>

                <div className="detail-field full-width">
                    <label>Assignee</label>
                    <input
                        type="text"
                        value={issue.assignee || ''}
                        placeholder="Unassigned"
                        onChange={(e) => onUpdateField(issue.id, 'assignee', e.target.value || null)}
                    />
                </div>

                <div className="detail-field full-width">
                    <label>Labels</label>
                    <LabelPicker
                        selectedLabels={issue.labels || []}
                        availableLabels={allLabels}
                        onAddLabel={(l) => onAddLabel(issue.id, l)}
                        onRemoveLabel={(l) => onRemoveLabel(issue.id, l)}
                    />
                </div>

                <div className="detail-field full-width">
                    <label>Description</label>
                    <textarea
                        className="description-editor"
                        value={issue.description || ''}
                        onChange={(e) => onUpdateField(issue.id, 'description', e.target.value)}
                        rows={6}
                    />
                </div>

                {issue.affected_symbols && issue.affected_symbols.length > 0 && (
                    <div className="detail-field full-width">
                        <label>Affected Symbols</label>
                        <ul className="affected-symbols-list">
                            {issue.affected_symbols.map((symbol) => (
                                <li key={symbol}>
                                    <a href="#" onClick={(e) => {
                                        e.preventDefault();
                                        window.vscode.postMessage({ command: 'openFile', file: symbol.split('::')[0] });
                                    }}>
                                        {symbol}
                                    </a>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                <div className="detail-field full-width comments-section">
                    <h3>Comments</h3>
                    <div className="comments-list">
                        {(issue.comments || []).map((comment: Comment) => (
                            <div key={comment.id} className="comment-item">
                                <div className="comment-header">
                                    <strong>{comment.author}</strong>
                                    <span>{new Date(comment.created_at).toLocaleString()}</span>
                                </div>
                                <div className="comment-body">{comment.text}</div>
                            </div>
                        ))}
                    </div>
                    <div className="comment-input">
                        <textarea
                            value={commentText}
                            onChange={(e) => setCommentText(e.target.value)}
                            placeholder="Add a comment..."
                            rows={3}
                        />
                        <button onClick={handleCommentSubmit} disabled={!commentText.trim()}>
                            Post Comment
                        </button>
                    </div>
                </div>
            </div>
        </aside>
    );
};
