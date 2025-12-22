import { useState, useEffect, useCallback, useRef } from 'react';
import init, { WasmStore } from './wasm/grits_core';
import { Issue, ViewType } from './types';
import { ListView } from './components/ListView';
import { KanbanView } from './components/KanbanView';
import { GraphView } from './components/GraphView';
import { AgendaView } from './components/AgendaView';
import { DetailPanel } from './components/DetailPanel';
import { CreateIssueModal } from './components/CreateIssueModal';
import './App.css';

// VS Code API
const vscode = window.vscode;

export function App() {
    const [issues, setIssues] = useState<Issue[]>([]);
    const [view, setView] = useState<ViewType>('list');
    const [selectedIssueId, setSelectedIssueId] = useState<string | null>(null);
    const [wasmStore, setWasmStore] = useState<WasmStore | null>(null);
    const isWasmLoading = useRef(false);
    const [loading, setLoading] = useState(true);
    const [conflictMode, setConflictMode] = useState(false);
    const [onboardingNeeded, setOnboardingNeeded] = useState(false);
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
    const [templates, setTemplates] = useState<string[]>(['bug', 'feature']); // Default templates

    // Initialize WASM
    useEffect(() => {
        if (isWasmLoading.current) return;
        isWasmLoading.current = true;

        console.log('Initializing Grits WASM...');
        const wasmUri = (window as any).wasmUri;
        init(wasmUri).then(() => {
            console.log('Grits WASM ready');
            const store = new WasmStore();
            setWasmStore(store);
            setLoading(false);
        }).catch((err: any) => {
            console.error('Failed to initialize Grits WASM:', err);
            setLoading(false);
        });
    }, []);

    // Handle messages from VS Code extension
    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;
            if (message.type === 'update' && message.content) {
                if (wasmStore) {
                    try {
                        wasmStore.load_from_jsonl(message.content);
                        refreshIssues();
                    } catch (err) {
                        console.error('WASM load failed:', err);
                    }
                }
            } else if (message.type === 'workspace_load' && message.contents) {
                if (wasmStore) {
                    try {
                        wasmStore.load_workspace(message.contents);
                        refreshIssues();
                    } catch (err) {
                        console.error('WASM workspace load failed:', err);
                    }
                }
            } else if (message.type === 'conflict') {
                setConflictMode(true);
            } else if (message.type === 'onboarding_needed') {
                setOnboardingNeeded(true);
            } else if (message.type === 'templates') {
                setTemplates(message.templates || []);
            } else if (message.type === 'template_content') {
                window.dispatchEvent(new CustomEvent('template-loaded', { detail: message }));
            }
        };

        window.addEventListener('message', handleMessage);

        // Signal ready
        vscode.postMessage({ type: 'ready' });

        return () => window.removeEventListener('message', handleMessage);
    }, [wasmStore]);

    const refreshIssues = useCallback(() => {
        if (!wasmStore) return;
        try {
            const json = wasmStore.list_issues("");
            setIssues(JSON.parse(json));
        } catch (e) {
            console.error("Failed to list issues:", e);
        }
    }, [wasmStore]);

    const handleUpdateField = useCallback((id: string, field: string, value: any) => {
        if (!wasmStore) return;

        try {
            const issueJson = wasmStore.get_issue(id);
            if (issueJson === "null") return;

            const issue = JSON.parse(issueJson);
            issue[field] = value;

            wasmStore.update_issue(JSON.stringify(issue));
            refreshIssues();

            // Save to disk
            const content = wasmStore.save_to_jsonl();
            vscode.postMessage({ type: 'save', content });
        } catch (e) {
            console.error("Failed to update field:", e);
        }
    }, [wasmStore, refreshIssues]);

    const handleUpdateIssue = useCallback((updatedIssue: Issue) => {
         if (!wasmStore) return;
         try {
             wasmStore.update_issue(JSON.stringify(updatedIssue));
             refreshIssues();
             const content = wasmStore.save_to_jsonl();
             vscode.postMessage({ type: 'save', content });
         } catch(e) {
             console.error("Failed to update issue:", e);
         }
    }, [wasmStore, refreshIssues]);

    const handleAddComment = useCallback((issueId: string, author: string, text: string) => {
        if (!wasmStore) return;
        try {
            wasmStore.add_comment(issueId, author, text);
            refreshIssues();
            const content = wasmStore.save_to_jsonl();
            vscode.postMessage({ type: 'save', content });
        } catch (e) {
            console.error("Failed to add comment:", e);
        }
    }, [wasmStore, refreshIssues]);

    const handleAddLabel = useCallback((issueId: string, label: string) => {
        if (!wasmStore) return;
        try {
            wasmStore.add_label(issueId, label);
            refreshIssues();
            const content = wasmStore.save_to_jsonl();
            vscode.postMessage({ type: 'save', content });
        } catch (e) {
            console.error("Failed to add label:", e);
        }
    }, [wasmStore, refreshIssues]);

    const handleRemoveLabel = useCallback((issueId: string, label: string) => {
        if (!wasmStore) return;
        try {
            wasmStore.remove_label(issueId, label);
            refreshIssues();
            const content = wasmStore.save_to_jsonl();
            vscode.postMessage({ type: 'save', content });
        } catch (e) {
            console.error("Failed to remove label:", e);
        }
    }, [wasmStore, refreshIssues]);

    const handleCreateIssue = useCallback((title: string, description: string, type: string, priority: number) => {
        if (!wasmStore) return;
        try {
            const issueObj = {
                title,
                description,
                issue_type: type,
                priority,
                status: 'open',
                labels: [],
                dependencies: [],
                comments: [],
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                id: '',
                content_hash: '',
                design: '',
                acceptance_criteria: '',
                notes: '',
                assignee: null,
                estimated_minutes: null,
                closed_at: null,
                external_ref: null,
                sender: '',
                ephemeral: false,
                replies_to: '',
                relates_to: [],
                duplicate_of: '',
                superseded_by: '',
                deleted_at: null,
                deleted_by: '',
                delete_reason: '',
                original_type: ''
            };

            wasmStore.create_issue(JSON.stringify(issueObj));
            refreshIssues();
            const content = wasmStore.save_to_jsonl();
            vscode.postMessage({ type: 'save', content });
        } catch (e) {
            console.error("Failed to create issue:", e);
        }
    }, [wasmStore, refreshIssues]);

    const handleLoadTemplate = (templateName: string): Promise<string> => {
        return new Promise((resolve) => {
            const handler = (e: Event) => {
                const detail = (e as CustomEvent).detail;
                if (detail.name === templateName) {
                    window.removeEventListener('template-loaded', handler);
                    resolve(detail.content);
                }
            };
            window.addEventListener('template-loaded', handler);
            vscode.postMessage({ type: 'command', command: 'grits.loadTemplate', name: templateName });

            setTimeout(() => {
                window.removeEventListener('template-loaded', handler);
                resolve('');
            }, 2000);
        });
    };

    const handleSelectIssue = useCallback((issue: Issue | null) => {
        setSelectedIssueId(issue ? issue.id : null);
    }, []);

    const selectedIssue = issues.find(i => i.id === selectedIssueId) || null;

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

    const handleOnboard = () => {
         vscode.postMessage({ type: 'command', command: 'grits.onboard' });
         setOnboardingNeeded(false);
    };

    if (loading) {
        return <div className="loading">Loading Grits Engine...</div>;
    }

    if (conflictMode) {
        return (
             <div className="conflict-banner">
                <h2>⚠️ Git Merge Conflict Detected</h2>
                <p>The issue file contains conflict markers. Please resolve them using the standard Merge Editor.</p>
                <button onClick={() => vscode.postMessage({ type: 'command', command: 'git.openMergeEditor' })}>
                    Open Merge Editor
                </button>
             </div>
        );
    }

    if (onboardingNeeded) {
        return (
            <div className="onboarding-view">
                <h2>Welcome to Grits</h2>
                <p>No .grits directory found. Let's get set up.</p>
                <button onClick={handleOnboard}>Initialize Grits</button>
            </div>
        );
    }

    return (
        <div className="app">
            <header className="app-header">
                <div className="header-title">
                    <span className="logo">📋</span>
                    <h1>Grits</h1>
                    <span className="issue-count">{issues.length} issues</span>
                    <button className="new-issue-btn" onClick={() => setIsCreateModalOpen(true)}>+ New Issue</button>
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
                        onSelectIssue={handleSelectIssue}
                    />
                )}
                {view === 'kanban' && (
                    <KanbanView
                        issues={issues}
                        onUpdateField={handleUpdateField}
                        onSelectIssue={handleSelectIssue}
                    />
                )}
                {view === 'graph' && (
                    <GraphView issues={issues} onSelectIssue={handleSelectIssue} />
                )}
                {view === 'agenda' && (
                    <AgendaView
                        issues={issues}
                        onUpdateField={handleUpdateField}
                        onSelectIssue={handleSelectIssue}
                    />
                )}
            </main>

            {selectedIssue && (
                <DetailPanel
                    issue={selectedIssue}
                    onClose={() => setSelectedIssueId(null)}
                    onUpdate={handleUpdateIssue}
                    onAddComment={handleAddComment}
                    onAddLabel={handleAddLabel}
                    onRemoveLabel={handleRemoveLabel}
                    allIssues={issues}
                />
            )}

            {isCreateModalOpen && (
                <CreateIssueModal
                    onClose={() => setIsCreateModalOpen(false)}
                    onCreate={handleCreateIssue}
                    templates={templates}
                    onLoadTemplate={handleLoadTemplate}
                />
            )}
        </div>
    );
}
