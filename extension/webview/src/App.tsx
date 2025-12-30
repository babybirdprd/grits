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
import { StatusBar } from './components/StatusBar';
import './index.css';

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
    const [issueSearchQuery, setIssueSearchQuery] = useState('');

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

    // View switcher - Icon-based sidebar (Linear/Jira style)
    const NavItem = ({ id, label, icon, badge }: { id: ViewType | 'settings'; label: string; icon: string | JSX.Element; badge?: number }) => (
        <button
            className={`flex flex-col items-center justify-center w-full py-3 gap-1 transition-all group ${view === id
                ? 'bg-vscode-accent/10 text-vscode-accent border-r-2 border-vscode-accent'
                : 'text-vscode-fg/60 hover:text-vscode-fg hover:bg-vscode-bg/50'
                }`}
            onClick={() => setView(id as ViewType)}
        >
            <span className={`text-xl group-hover:scale-110 transition-transform ${view === id ? 'opacity-100' : 'opacity-70 group-hover:opacity-100'}`}>
                {icon}
            </span>
            <span className="text-[10px] font-medium uppercase tracking-wider opacity-60 group-hover:opacity-100">
                {label}
            </span>
            {badge !== undefined && badge > 0 && (
                <span className="absolute top-2 right-2 min-w-[16px] h-4 flex items-center justify-center bg-vscode-accent text-white text-[9px] font-bold rounded-full px-1 shadow-sm">
                    {badge}
                </span>
            )}
        </button>
    );

    // Filter issues based on search query (with defensive null checks)
    const filteredIssues = issues.filter(issue =>
        (issue.title || '').toLowerCase().includes(issueSearchQuery.toLowerCase()) ||
        (issue.id || '').toLowerCase().includes(issueSearchQuery.toLowerCase()) ||
        (issue.labels || []).some(l => l.toLowerCase().includes(issueSearchQuery.toLowerCase()))
    );

    const inProgressCount = issues.filter(i => i.status === 'in-progress').length;
    const blockedCount = issues.filter(i => i.status === 'blocked').length;

    if (needsOnboarding) {
        return <OnboardingView onInitialize={() => vscode.postMessage({ type: 'onboard' })} />;
    }

    return (
        <div className="flex h-screen w-screen overflow-hidden bg-vscode-bg text-vscode-fg font-sans select-none">
            {/* Left Sidebar Navigation */}
            <nav className="w-20 bg-vscode-sidebar border-r border-vscode-border flex flex-col shrink-0">
                <div className="flex flex-col flex-1 mt-4">
                    <NavItem id="kanban" label="Issues" icon="✓" badge={issues.length} />
                    <NavItem id="topology" label="Topology" icon="◇" />
                    <NavItem id="graph" label="Graph" icon="◎" />
                    <NavItem id="list" label="List" icon="☰" />
                </div>

                <div className="mt-auto mb-4 border-t border-vscode-border pt-4">
                    <NavItem id="agenda" label="Focus" icon="🎯" badge={blockedCount} />
                    <NavItem id="settings" label="Settings" icon="⚙" />
                </div>
            </nav>

            {/* Main Content Area */}
            <div className="flex-1 flex flex-col min-w-0">
                {/* Top Header */}
                <header className="h-16 flex items-center justify-between px-6 bg-vscode-sidebar border-b border-vscode-border shrink-0">
                    <div className="flex items-center gap-4">
                        <div className="w-8 h-8 rounded-lg bg-vscode-accent shadow-lg shadow-vscode-accent/20 flex items-center justify-center text-white font-bold text-xl uppercase italic">
                            G
                        </div>
                        <div className="flex flex-col">
                            <h1 className="text-sm font-bold tracking-tight text-vscode-fg/90">Grits Studio</h1>
                            <span className="text-[10px] text-vscode-fg/40 uppercase tracking-[0.2em] font-medium">Internal Release v2.4</span>
                        </div>
                        <div className="ml-2 px-2 py-0.5 rounded-full bg-vscode-border text-[9px] font-bold text-vscode-fg/50 border border-vscode-border">
                            {issues.length} ISSUES
                        </div>
                    </div>

                    {/* Central Search */}
                    <div className="flex-1 max-w-md mx-8 relative group">
                        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-vscode-fg/30 group-focus-within:text-vscode-accent">
                            <span>🔍</span>
                        </div>
                        <input
                            type="text"
                            className="block w-full pl-10 pr-3 py-1.5 bg-vscode-bg/50 border border-vscode-border rounded-lg text-sm placeholder-vscode-fg/30 focus:outline-none focus:ring-1 focus:ring-vscode-accent focus:border-vscode-accent transition-all hover:bg-vscode-bg/80"
                            placeholder="Search issues, labels, or IDs..."
                            value={issueSearchQuery}
                            onChange={(e) => setIssueSearchQuery(e.target.value)}
                        />
                    </div>

                    {/* View Toggle */}
                    <div className="flex items-center gap-3">
                        <div className="p-1 bg-vscode-bg/50 rounded-lg border border-vscode-border flex gap-1">
                            <button
                                className={`px-3 py-1 text-[11px] font-medium rounded-md transition-all ${view === 'list' ? 'bg-vscode-accent text-white shadow-sm' : 'text-vscode-fg/60 hover:text-vscode-fg hover:bg-vscode-bg'}`}
                                onClick={() => setView('list')}
                            >
                                List
                            </button>
                            <button
                                className={`px-3 py-1 text-[11px] font-medium rounded-md transition-all ${view === 'kanban' ? 'bg-vscode-accent text-white shadow-sm' : 'text-vscode-fg/60 hover:text-vscode-fg hover:bg-vscode-bg'}`}
                                onClick={() => setView('kanban')}
                            >
                                Kanban
                            </button>
                        </div>
                        <span className="w-px h-6 bg-vscode-border"></span>
                        <div className="w-8 h-8 rounded-full bg-blue-500/20 border border-blue-500/30 flex items-center justify-center text-blue-400 font-bold text-xs ring-2 ring-transparent hover:ring-blue-500/20 cursor-pointer transition-all">
                            S
                        </div>
                    </div>
                </header>

                {/* Workspace Area */}
                <main className="flex-1 relative overflow-hidden flex flex-col">
                    <div className="flex-1 relative overflow-hidden">
                        {view === 'list' && (
                            <ListView
                                issues={filteredIssues}
                                onUpdateField={handleUpdateField}
                                onBulkUpdate={handleBulkUpdate}
                                onSelectIssue={setSelectedIssue}
                            />
                        )}
                        {view === 'kanban' && (
                            <KanbanView
                                issues={filteredIssues}
                                onUpdateField={handleUpdateField}
                                onSelectIssue={setSelectedIssue}
                            />
                        )}
                        {view === 'graph' && (
                            <GraphView issues={filteredIssues} onSelectIssue={setSelectedIssue} />
                        )}
                        {view === 'agenda' && (
                            <AgendaView
                                issues={filteredIssues}
                                onUpdateField={handleUpdateField}
                                onSelectIssue={setSelectedIssue}
                            />
                        )}
                        {view === 'topology' && (
                            <div className="flex h-full w-full">
                                <div className="flex-1 relative">
                                    <div className="absolute top-6 left-6 z-10 pointer-events-auto w-80">
                                        <div className="relative group">
                                            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-vscode-fg/30 group-focus-within:text-blue-400">
                                                <span>⚡</span>
                                            </div>
                                            <input
                                                type="text"
                                                className="w-full pl-10 pr-3 py-2.5 bg-vscode-bg/70 backdrop-blur-xl border border-vscode-border/50 rounded-xl text-sm placeholder-vscode-fg/30 focus:outline-none focus:ring-1 focus:ring-blue-500/50 shadow-2xl"
                                                placeholder="Instant jump to symbol... (<10ms)"
                                                value={symbolSearchQuery}
                                                onChange={handleSymbolSearchChange}
                                            />
                                            {symbolSearchResults.length > 0 && (
                                                <div className="absolute top-full left-0 right-0 mt-2 bg-vscode-bg/90 backdrop-blur-2xl border border-vscode-border rounded-xl shadow-2xl max-h-96 overflow-y-auto z-50 overflow-hidden">
                                                    {symbolSearchResults.map((res: any) => (
                                                        <div
                                                            key={res.id}
                                                            className="px-4 py-3 flex items-center gap-3 hover:bg-vscode-accent/10 cursor-pointer border-b border-vscode-border/30 last:border-0 transition-colors"
                                                            onClick={() => {
                                                                console.log('Jump to:', res.id);
                                                                setSymbolSearchResults([]);
                                                                setSymbolSearchQuery('');
                                                            }}
                                                        >
                                                            <div className="flex-1 min-w-0">
                                                                <div className="text-blue-400 font-medium truncate">{res.name}</div>
                                                                <div className="text-[10px] text-vscode-fg/40 truncate">{res.file}</div>
                                                            </div>
                                                            <div className="bg-yellow-500/10 text-yellow-500 px-1.5 py-0.5 rounded text-[9px] font-bold">
                                                                {(res.pagerank * 100).toFixed(0)}
                                                            </div>
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
                                </div>
                                <aside className="w-80 bg-vscode-sidebar/50 backdrop-blur-md border-l border-vscode-border overflow-y-auto">
                                    <VitalsDashboard
                                        solidScore={vitals.solidScore}
                                        betti0={vitals.betti0}
                                        betti1={vitals.betti1}
                                        hotspots={vitals.hotspots}
                                        inProgressCount={issues.filter(i => i.status === 'in-progress').length}
                                    />
                                </aside>
                            </div>
                        )}
                        {view === 'settings' && (
                            <div className="p-10 flex flex-col items-center justify-center h-full text-center">
                                <div className="text-4xl mb-4">⚙️</div>
                                <h1 className="text-xl font-bold mb-2">Settings</h1>
                                <p className="text-vscode-fg/50 max-w-sm">Configuration options for Grits Studio will appear here.</p>
                            </div>
                        )}
                    </div>

                    {/* Footer Status Bar */}
                    <StatusBar
                        solidScore={vitals.solidScore}
                        inProgressCount={inProgressCount}
                        blockedCount={blockedCount}
                    />
                </main>
            </div>

            {/* Detail Panel overlay */}
            {selectedIssue && (
                <div className="fixed inset-y-0 right-0 z-50 animate-in slide-in-from-right duration-300 shadow-2xl">
                    <DetailPanel
                        issue={selectedIssue}
                        onClose={() => setSelectedIssue(null)}
                        onUpdateField={handleUpdateField}
                        onAddComment={handleAddComment}
                        allLabels={allLabels}
                        onAddLabel={handleAddLabel}
                        onRemoveLabel={handleRemoveLabel}
                    />
                </div>
            )}
        </div>
    );
}
