import { useState, useEffect, useCallback, useRef } from 'react';
import init, { WasmStore } from './pkg/grits_core';
import { Issue, ViewType } from './types';
import { ListView } from './components/ListView';
import { KanbanView } from './components/KanbanView';
import { GraphView } from './components/GraphView';
import { AgendaView } from './components/AgendaView';
import { DetailPanel } from './components/DetailPanel';
import { OnboardingView } from './components/OnboardingView';
import './App.css';

// VS Code API
const vscode = window.vscode;

export function App() {
    const [issues, setIssues] = useState<Issue[]>([]);
    const [allLabels, setAllLabels] = useState<string[]>([]);
    const [view, setView] = useState<ViewType>('list');
    const [selectedIssue, setSelectedIssue] = useState<Issue | null>(null);
    const [wasmReady, setWasmReady] = useState(false);
    const [needsOnboarding, setNeedsOnboarding] = useState(false);
    const storeRef = useRef<WasmStore | null>(null);
    const isWasmLoading = useRef(false);

    // Initialize WASM
    useEffect(() => {
        if (isWasmLoading.current) return;
        isWasmLoading.current = true;

        console.log('Initializing Grits WASM...');
        const wasmUri = (window as any).wasmUri;
        init(wasmUri).then(() => {
            console.log('Grits WASM ready');
            storeRef.current = new WasmStore();
            setWasmReady(true);
        }).catch((err: any) => {
            console.error('Failed to initialize Grits WASM:', err);
        });
    }, []);

    // Handle messages from VS Code extension
    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            const message = event.data;
            if (message.type === 'update') {
                if (message.content) {
                    if (wasmReady && storeRef.current) {
                        try {
                            storeRef.current.load(message.content);
                            refreshIssues();
                            setNeedsOnboarding(false);
                        } catch (err) {
                            console.error('WASM load failed:', err);
                        }
                    } else {
                         console.warn("WASM not ready for update, waiting...");
                    }
                } else {
                    // Empty content might mean empty file or no file
                    // If file doesn't exist, extension usually sends empty string or specific message
                    // Let's assume if we get empty content and it's intended, we clear issues.
                    // But if it's "not initialized", we show onboarding.
                    // We need a specific signal for "no .grits found".
                }
            } else if (message.type === 'no-config') {
                setNeedsOnboarding(true);
            }
        };

        console.log('React App message listener attached. WASM ready:', wasmReady);
        window.addEventListener('message', handleMessage);

        // Signal ready
        if (wasmReady) {
            vscode.postMessage({ type: 'ready' });
        }

        return () => window.removeEventListener('message', handleMessage);
    }, [wasmReady]);

    const refreshIssues = useCallback(() => {
        if (!storeRef.current) return;
        try {
            // Can pass filters here if we add UI controls for them
            const json = storeRef.current.list_issues(undefined);
            const list = JSON.parse(json);
            setIssues(list);

            // Refresh labels
            const labelsJson = storeRef.current.get_all_labels();
            setAllLabels(JSON.parse(labelsJson));

            // If an issue is selected, refresh its data too
            if (selectedIssue) {
                 const updated = list.find((i: Issue) => i.id === selectedIssue.id);
                 if (updated) setSelectedIssue(updated);
            }
        } catch (e) {
            console.error("Failed to refresh issues:", e);
        }
    }, [selectedIssue]);

    // Handle field updates
    const handleUpdateField = useCallback(
        (id: string, field: string, value: unknown) => {
            if (storeRef.current) {
                try {
                    const valueJson = JSON.stringify(value);
                    storeRef.current.update_issue(id, field, valueJson);

                    // Export and save
                    const newContent = storeRef.current.export();
                    vscode.postMessage({ type: 'save', content: newContent });

                    refreshIssues();
                } catch (err: any) {
                    console.error('WASM update failed:', err);
                }
            }
        },
        [refreshIssues]
    );

    const handleBulkUpdate = useCallback((ids: string[], field: string, value: unknown) => {
        if (storeRef.current) {
            try {
                const idsJson = JSON.stringify(ids);
                const valueJson = JSON.stringify(value);
                storeRef.current.bulk_update(idsJson, field, valueJson);

                const newContent = storeRef.current.export();
                vscode.postMessage({ type: 'save', content: newContent });

                refreshIssues();
            } catch (err: any) {
                console.error('WASM bulk update failed:', err);
            }
        }
    }, [refreshIssues]);

    const handleAddLabel = useCallback((id: string, label: string) => {
        if (storeRef.current) {
            storeRef.current.add_label(id, label);
            const newContent = storeRef.current.export();
            vscode.postMessage({ type: 'save', content: newContent });
            refreshIssues();
        }
    }, [refreshIssues]);

    const handleRemoveLabel = useCallback((id: string, label: string) => {
        if (storeRef.current) {
            storeRef.current.remove_label(id, label);
            const newContent = storeRef.current.export();
            vscode.postMessage({ type: 'save', content: newContent });
            refreshIssues();
        }
    }, [refreshIssues]);

    const handleAddComment = useCallback((id: string, text: string) => {
        if (storeRef.current) {
            // TODO: Get real user name from VS Code config passed via message
            const author = "Me";
            storeRef.current.add_comment(id, text, author);
            const newContent = storeRef.current.export();
            vscode.postMessage({ type: 'save', content: newContent });
            refreshIssues();
        }
    }, [refreshIssues]);

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

    if (needsOnboarding) {
        return <OnboardingView onInitialize={() => vscode.postMessage({ type: 'onboard' })} />;
    }

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
                        onBulkUpdate={handleBulkUpdate}
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

            {selectedIssue && (
                <DetailPanel
                    issue={selectedIssue}
                    onClose={() => setSelectedIssue(null)}
                    onUpdateField={handleUpdateField}
                    onAddComment={handleAddComment}
                    allLabels={allLabels}
                    onAddLabel={handleAddLabel}
                    onRemoveLabel={handleRemoveLabel}
                />
            )}
        </div>
    );
}
