import { useState, useEffect, useCallback } from 'react';
import { Issue, ViewType } from './types';
import { ListView } from './components/ListView';
import { KanbanView } from './components/KanbanView';
import { GraphView } from './components/GraphView';
import { AgendaView } from './components/AgendaView';
import './App.css';

// VS Code API
const vscode = window.vscode;

export function App() {
    const [issues, setIssues] = useState<Issue[]>([]);
    const [view, setView] = useState<ViewType>('list');
    const [selectedIssue, setSelectedIssue] = useState<Issue | null>(null);
    const [jsonlContent, setJsonlContent] = useState<string>('');

    // Handle messages from VS Code extension
    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;
            if (message.type === 'update' && message.content) {
                setJsonlContent(message.content);

                // Parse JSONL to issues
                try {
                    const parsed: Issue[] = [];
                    for (const line of message.content.split('\n')) {
                        const trimmed = line.trim();
                        if (trimmed) {
                            parsed.push(JSON.parse(trimmed));
                        }
                    }
                    setIssues(parsed);
                } catch (err) {
                    console.error('Failed to parse issues:', err);
                }
            }
        };

        window.addEventListener('message', handleMessage);

        // Signal ready
        vscode.postMessage({ type: 'ready' });

        return () => window.removeEventListener('message', handleMessage);
    }, []);

    // Handle field updates
    const handleUpdateField = useCallback(
        (id: string, field: string, value: unknown) => {
            // Update local state immediately for responsiveness
            setIssues((prev) =>
                prev.map((issue) =>
                    issue.id === id
                        ? { ...issue, [field]: value, updated_at: new Date().toISOString() }
                        : issue
                )
            );

            // Update JSONL and save
            const lines = jsonlContent.split('\n');
            const newLines = lines.map((line) => {
                const trimmed = line.trim();
                if (!trimmed) return line;

                try {
                    const issue = JSON.parse(trimmed);
                    if (issue.id === id) {
                        issue[field] = value;
                        issue.updated_at = new Date().toISOString();
                        return JSON.stringify(issue);
                    }
                } catch {
                    // Keep original line if parse fails
                }
                return line;
            });

            const newContent = newLines.join('\n');
            setJsonlContent(newContent);

            // Send save message to VS Code
            vscode.postMessage({ type: 'save', content: newContent });
        },
        [jsonlContent]
    );

    // View switcher
    const ViewTab = ({ id, label, icon }: { id: ViewType; label: string; icon: string }) => (
        <button
            className={`view-tab ${view === id ? 'active' : ''}`}
            onClick={() => setView(id)}
        >
            <span className="view-icon">{icon}</span>
            <span className="view-label">{label}</span>
        </button>
    );

    return (
        <div className="app">
            <header className="app-header">
                <div className="header-title">
                    <span className="logo">📋</span>
                    <h1>Grits</h1>
                    <span className="issue-count">{issues.length} issues</span>
                </div>
                <nav className="view-tabs">
                    <ViewTab id="list" label="List" icon="📄" />
                    <ViewTab id="kanban" label="Kanban" icon="📊" />
                    <ViewTab id="graph" label="Graph" icon="🔗" />
                    <ViewTab id="agenda" label="Focus" icon="🎯" />
                </nav>
            </header>

            <main className="app-main">
                {view === 'list' && (
                    <ListView
                        issues={issues}
                        onUpdateField={handleUpdateField}
                        onSelectIssue={setSelectedIssue}
                    />
                )}
                {view === 'kanban' && (
                    <KanbanView
                        issues={issues}
                        onUpdateField={handleUpdateField}
                        onSelectIssue={setSelectedIssue}
                    />
                )}
                {view === 'graph' && (
                    <GraphView issues={issues} onSelectIssue={setSelectedIssue} />
                )}
                {view === 'agenda' && (
                    <AgendaView
                        issues={issues}
                        onUpdateField={handleUpdateField}
                        onSelectIssue={setSelectedIssue}
                    />
                )}
            </main>

            {/* Issue detail panel */}
            {selectedIssue && (
                <aside className="detail-panel">
                    <div className="detail-header">
                        <h2>{selectedIssue.title}</h2>
                        <button
                            className="close-btn"
                            onClick={() => setSelectedIssue(null)}
                        >
                            ✕
                        </button>
                    </div>
                    <div className="detail-body">
                        <div className="detail-field">
                            <label>ID</label>
                            <span className="id-value">{selectedIssue.id}</span>
                        </div>
                        <div className="detail-field">
                            <label>Status</label>
                            <span>{selectedIssue.status}</span>
                        </div>
                        <div className="detail-field">
                            <label>Priority</label>
                            <span>P{selectedIssue.priority}</span>
                        </div>
                        <div className="detail-field">
                            <label>Type</label>
                            <span>{selectedIssue.issue_type}</span>
                        </div>
                        {selectedIssue.assignee && (
                            <div className="detail-field">
                                <label>Assignee</label>
                                <span>{selectedIssue.assignee}</span>
                            </div>
                        )}
                        <div className="detail-field full-width">
                            <label>Description</label>
                            <p className="description">
                                {selectedIssue.description || 'No description'}
                            </p>
                        </div>
                    </div>
                </aside>
            )}
        </div>
    );
}
