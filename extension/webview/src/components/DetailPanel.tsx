import React, { useState, useEffect, useRef } from 'react';
import { Issue, Comment } from '../types';
import { LabelPicker } from './LabelPicker';

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

    const Field = ({ label, children }: { label: string; children: React.ReactNode }) => (
        <div className="flex flex-col gap-1.5 px-6 py-4 border-b border-vscode-border/30 last:border-0 hover:bg-vscode-bg/50 transition-colors">
            <label className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30">{label}</label>
            <div className="text-sm font-medium">{children}</div>
        </div>
    );

    const inputClasses = "w-full bg-vscode-sidebar/50 border border-vscode-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-vscode-accent transition-all placeholder-vscode-fg/20";

    return (
        <aside className="w-[450px] h-full bg-vscode-sidebar/95 backdrop-blur-2xl border-l border-vscode-border flex flex-col shadow-[-20px_0_50px_rgba(0,0,0,0.3)]">
            {/* Panel Header */}
            <div className="h-16 flex items-center justify-between px-6 border-b border-vscode-border shrink-0">
                <div className="flex-1 min-w-0 mr-4">
                    {isEditingTitle ? (
                        <input
                            ref={titleInputRef}
                            className={`${inputClasses} font-bold text-lg`}
                            value={titleValue}
                            onChange={(e) => setTitleValue(e.target.value)}
                            onBlur={handleTitleSubmit}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') handleTitleSubmit();
                            }}
                        />
                    ) : (
                        <h2
                            className="text-lg font-bold truncate text-vscode-fg/90 cursor-pointer hover:text-vscode-accent transition-colors"
                            onClick={() => setIsEditingTitle(true)}
                            title="Click to edit title"
                        >
                            {issue.title}
                        </h2>
                    )}
                </div>
                <button
                    className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-vscode-bg text-vscode-fg/40 hover:text-vscode-fg transition-all"
                    onClick={onClose}
                    title="Close (Esc)"
                >
                    ✕
                </button>
            </div>

            {/* Panel Body */}
            <div className="flex-1 overflow-y-auto scrollbar-hide select-none">
                <div className="grid grid-cols-2">
                    <Field label="Issue ID">
                        <span className="font-mono text-xs text-vscode-fg/50 tracking-wider font-bold italic">{issue.id}</span>
                    </Field>

                    <Field label="Status">
                        <select
                            className={`${inputClasses} border-none p-0 bg-transparent text-vscode-accent font-bold cursor-pointer uppercase text-xs`}
                            value={issue.status}
                            onChange={(e) => onUpdateField(issue.id, 'status', e.target.value)}
                        >
                            <option value="open">Open</option>
                            <option value="in_progress">In Progress</option>
                            <option value="blocked">Blocked</option>
                            <option value="closed">Closed</option>
                        </select>
                    </Field>

                    <Field label="Priority">
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min="1" max="5"
                                className="w-24 h-1.5 bg-vscode-border rounded-lg appearance-none cursor-pointer accent-vscode-accent"
                                value={issue.priority}
                                onChange={(e) => onUpdateField(issue.id, 'priority', parseInt(e.target.value))}
                            />
                            <span className="text-xs font-bold text-vscode-accent">P{issue.priority}</span>
                        </div>
                    </Field>

                    <Field label="Type">
                        <select
                            className={`${inputClasses} border-none p-0 bg-transparent text-vscode-fg/60 font-bold cursor-pointer uppercase text-xs`}
                            value={issue.issue_type}
                            onChange={(e) => onUpdateField(issue.id, 'issue_type', e.target.value)}
                        >
                            <option value="bug">Bug</option>
                            <option value="feature">Feature</option>
                            <option value="task">Task</option>
                            <option value="epic">Epic</option>
                        </select>
                    </Field>
                </div>

                <div className="px-6 py-4 border-b border-vscode-border/30 hover:bg-vscode-bg/50 transition-colors">
                    <label className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 mb-2 block">Assignee</label>
                    <input
                        type="text"
                        className={inputClasses}
                        value={issue.assignee || ''}
                        placeholder="Assign this task..."
                        onChange={(e) => onUpdateField(issue.id, 'assignee', e.target.value || null)}
                    />
                </div>

                <div className="px-6 py-4 border-b border-vscode-border/30 hover:bg-vscode-bg/50 transition-colors">
                    <label className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 mb-2 block">Labels</label>
                    <LabelPicker
                        selectedLabels={issue.labels || []}
                        availableLabels={allLabels}
                        onAddLabel={(l) => onAddLabel(issue.id, l)}
                        onRemoveLabel={(l) => onRemoveLabel(issue.id, l)}
                    />
                </div>

                <div className="px-6 py-4 border-b border-vscode-border/30 hover:bg-vscode-bg/50 transition-colors">
                    <label className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 mb-2 block">Description</label>
                    <textarea
                        className={`${inputClasses} min-h-[120px] resize-none leading-relaxed text-vscode-fg/80`}
                        value={issue.description || ''}
                        placeholder="Detailed description of the issue..."
                        onChange={(e) => onUpdateField(issue.id, 'description', e.target.value)}
                    />
                </div>

                {issue.affected_symbols && issue.affected_symbols.length > 0 && (
                    <div className="px-6 py-4 border-b border-vscode-border/30 hover:bg-vscode-bg/50 transition-colors">
                        <label className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 mb-2 block">Analysis Context</label>
                        <ul className="space-y-1.5">
                            {issue.affected_symbols.map((symbol) => (
                                <li key={symbol} className="group/sym">
                                    <a
                                        href="#"
                                        className="text-[11px] font-mono text-blue-400 hover:text-blue-300 transition-colors flex items-center gap-1.5"
                                        onClick={(e) => {
                                            e.preventDefault();
                                            window.vscode.postMessage({ command: 'openFile', file: symbol.split('::')[0] });
                                        }}
                                    >
                                        <span className="opacity-40">➔</span> {symbol}
                                    </a>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {/* Comments Section */}
                <div className="px-6 py-6 pb-20">
                    <div className="flex items-center justify-between mb-4">
                        <h3 className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30">Activity / Comments</h3>
                        <span className="text-[10px] font-bold text-vscode-fg/20">{(issue.comments || []).length}</span>
                    </div>

                    <div className="space-y-4 mb-6">
                        {(issue.comments || []).map((comment: Comment) => (
                            <div key={comment.id} className="group/comment flex gap-3">
                                <div className="w-8 h-8 rounded-full bg-vscode-accent/10 border border-vscode-accent/20 flex items-center justify-center text-[10px] font-bold text-vscode-accent shrink-0">
                                    {comment.author.charAt(0).toUpperCase()}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2 mb-1">
                                        <span className="text-[11px] font-bold text-vscode-fg/80">{comment.author}</span>
                                        <span className="text-[10px] text-vscode-fg/20">{new Date(comment.created_at).toLocaleDateString()}</span>
                                    </div>
                                    <div className="text-xs leading-relaxed text-vscode-fg/60 whitespace-pre-wrap">
                                        {comment.text}
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>

                    <div className="relative">
                        <textarea
                            className={`${inputClasses} min-h-[80px] pr-20`}
                            value={commentText}
                            onChange={(e) => setCommentText(e.target.value)}
                            placeholder="Add a comment..."
                        />
                        <button
                            className="absolute bottom-2 right-2 px-3 py-1 bg-vscode-accent text-white rounded text-[10px] font-bold uppercase disabled:opacity-30 transition-all hover:scale-105 active:scale-95 shadow-lg shadow-vscode-accent/20"
                            onClick={handleCommentSubmit}
                            disabled={!commentText.trim()}
                        >
                            Post
                        </button>
                    </div>
                </div>
            </div>
        </aside>
    );
};
