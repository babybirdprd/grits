import { useState, useEffect, useCallback, useRef } from 'react';
import init, { WasmStore, WasmTopologyStore } from './pkg/grits_core';
import { Issue, ViewType } from './types';
import { ListView } from './components/ListView';
import { KanbanView } from './components/KanbanView';
import { GraphView } from './components/GraphView';
import { AgendaView } from './components/AgendaView';
import { DetailPanel } from './components/DetailPanel';
import { OnboardingView } from './components/OnboardingView';
import { TopologyScene } from './components/TopologyScene';
import { VitalsDashboard } from './components/VitalsDashboard';
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
    const [topologyData, setTopologyData] = useState<any>(null);
    const [vitals, setVitals] = useState<{
        solidScore: number;
        betti0: number;
        betti1: number;
        betti2: number;
        hotspots: any[];
    }>({
        solidScore: 0,
        betti0: 0,
        betti1: 0,
        betti2: 0,
        hotspots: []
    });
    const storeRef = useRef<WasmStore | null>(null);
    const topologyStoreRef = useRef<WasmTopologyStore | null>(null);
    const isWasmLoading = useRef(false);
    const [symbolSearchQuery, setSymbolSearchQuery] = useState('');
    const [symbolSearchResults, setSymbolSearchResults] = useState<any[]>([]);

    // Initialize WASM
    useEffect(() => {
        if (isWasmLoading.current) return;
        isWasmLoading.current = true;

        console.log('Initializing Grits WASM...');
        const wasmUri = (window as any).wasmUri;
        init(wasmUri).then(() => {
            console.log('Grits WASM ready');
            storeRef.current = new WasmStore();
            topologyStoreRef.current = new WasmTopologyStore();
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
            } else if (message.type === 'topology') {
                // Handle topology data from dashboard panel - load into WASM store
                if (topologyStoreRef.current && message.data) {
                    try {
                        const stats = topologyStoreRef.current.load_topology(message.data);
                        console.log('Topology loaded into WASM:', JSON.parse(stats));
                        refreshVitals();
                        const data = JSON.parse(message.data);
                        setTopologyData(data.graph || data);
                    } catch (e) {
                        console.error('Failed to load topology into WASM:', e);
                    }
                } else {
                    try {
                        const data = JSON.parse(message.data);
                        setTopologyData(data.graph || data);
                    } catch (e) {
                        console.error('Failed to parse topology:', e);
                    }
                }
            } else if (message.type === 'init') {
                // Dashboard mode initialization
                if (wasmReady && storeRef.current && message.issues) {
                    storeRef.current.load(message.issues);
                    refreshIssues();
                }
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

    // Instant symbol search via WASM (<10ms)
    const searchSymbolsInstant = useCallback((query: string) => {
        if (!topologyStoreRef.current || !topologyStoreRef.current.is_loaded()) {
            setSymbolSearchResults([]);
            return;
        }
        if (!query.trim()) {
            setSymbolSearchResults([]);
            return;
        }
        try {
            const resultsJson = topologyStoreRef.current.search_symbols(query, 20);
            const results = JSON.parse(resultsJson);
            setSymbolSearchResults(results);
        } catch (e) {
            console.error('Symbol search failed:', e);
            setSymbolSearchResults([]);
        }
    }, []);

    // Handle symbol search input changes
    const handleSymbolSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const query = e.target.value;
        setSymbolSearchQuery(query);
        searchSymbolsInstant(query);
    };

    const refreshVitals = useCallback(() => {
        if (!topologyStoreRef.current || !topologyStoreRef.current.is_loaded()) return;
        try {
            const solidScore = topologyStoreRef.current.get_solid_score();
            const bettiJson = topologyStoreRef.current.get_betti();
            const betti = JSON.parse(bettiJson);
            const hotspotsJson = topologyStoreRef.current.get_hotspots(5);
            const hotspots = JSON.parse(hotspotsJson);

            setVitals({
                solidScore,
                betti0: betti.b0,
                betti1: betti.b1,
                betti2: betti.b2,
                hotspots
            });
        } catch (e) {
            console.error('Failed to refresh vitals from WASM:', e);
        }
    }, []);

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
                    <ViewTab id="topology" label="3D" icon="🌐" />
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
                {view === 'topology' && (
                    <div className="topology-container">
                        <div className="topology-overlay">
                            <div className="symbol-search">
                                <input
                                    type="text"
                                    placeholder="Instant jump to symbol... (<10ms)"
                                    value={symbolSearchQuery}
                                    onChange={handleSymbolSearchChange}
                                />
                                {symbolSearchResults.length > 0 && (
                                    <div className="search-results">
                                        {symbolSearchResults.map((res: any) => (
                                            <div
                                                key={res.id}
                                                className="search-row"
                                                onClick={() => {
                                                    // TODO: Orbit controls jump to node
                                                    console.log('Jump to:', res.id);
                                                    setSymbolSearchResults([]);
                                                    setSymbolSearchQuery('');
                                                }}
                                            >
                                                <span className="res-name">{res.name}</span>
                                                <span className="res-file">{res.file.split(/[\\/]/).pop()}</span>
                                                <span className="res-rank">{(res.pagerank * 100).toFixed(0)}</span>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        </div>
                        <TopologyScene
                            data={topologyData}
                            onNodeSelect={(id) => console.log('Selected:', id)}
                            solidScore={vitals.solidScore}
                        />
                        <div className="vitals-sidebar">
                            <VitalsDashboard
                                solidScore={vitals.solidScore}
                                betti0={vitals.betti0}
                                betti1={vitals.betti1}
                                hotspots={vitals.hotspots}
                                inProgressCount={issues.filter(i => i.status === 'in_progress').length}
                            />
                        </div>
                    </div>
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
