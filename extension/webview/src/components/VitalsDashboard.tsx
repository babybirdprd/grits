interface VitalsDashboardProps {
    solidScore: number;
    betti0: number;  // Connected components
    betti1: number;  // Cycles
    hotspots: Array<{ name: string; score: number }>;
    inProgressCount: number;
    isCacheStale?: boolean;
    onRescan?: () => void;
}

function SolidScoreGauge({ score }: { score: number }) {
    const percentage = Math.round(score * 100);
    const getColor = () => {
        if (score >= 0.7) return 'var(--status-closed)';
        if (score >= 0.4) return 'var(--status-open)';
        return 'var(--status-blocked)';
    };

    return (
        <div className="flex flex-col items-center gap-2 group">
            <div className="relative w-32 h-16 overflow-hidden">
                <svg viewBox="0 0 100 50" className="w-full h-full drop-shadow-lg">
                    <path
                        d="M 5 50 A 45 45 0 0 1 95 50"
                        fill="none"
                        stroke="var(--vscode-border)"
                        strokeWidth="10"
                        strokeLinecap="round"
                        className="opacity-20"
                    />
                    <path
                        d="M 5 50 A 45 45 0 0 1 95 50"
                        fill="none"
                        stroke={getColor()}
                        strokeWidth="10"
                        strokeLinecap="round"
                        strokeDasharray={`${score * 141.3} 141.3`}
                        className="transition-all duration-1000 ease-out"
                    />
                </svg>
                <div
                    className="absolute bottom-0 inset-x-0 text-2xl font-black text-center transition-colors duration-500"
                    style={{ color: getColor() }}
                >
                    {percentage}%
                </div>
            </div>
            <div className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 group-hover:text-vscode-fg/60 transition-colors">Solid Score</div>
        </div>
    );
}

function SpaghettiMeter({ cycles }: { cycles: number }) {
    const intensity = Math.min(cycles / 10, 1);
    return (
        <div className="flex flex-col items-center gap-2 group">
            <div className="relative w-24 h-16 flex items-center justify-center">
                <div className="flex gap-1 items-end h-8">
                    {[0.4, 0.7, 1.0, 0.6, 0.3].map((h, i) => (
                        <div
                            key={i}
                            className="w-1.5 bg-vscode-accent rounded-full transition-all duration-500"
                            style={{
                                height: `${h * 100}%`,
                                opacity: 0.2 + (intensity * 0.8),
                                filter: intensity > 0.5 ? `blur(${intensity * 2}px)` : 'none'
                            }}
                        />
                    ))}
                </div>
                <div className="absolute inset-0 flex items-center justify-center text-2xl font-black text-vscode-fg/80 drop-shadow-md">
                    {cycles}
                </div>
            </div>
            <div className="text-[10px] font-bold uppercase tracking-widest text-vscode-fg/30 group-hover:text-vscode-fg/60 transition-colors">Cycles (B₁)</div>
        </div>
    );
}

function HotspotsList({ hotspots }: { hotspots: Array<{ name: string; score: number }> }) {
    return (
        <div className="bg-vscode-bg/50 rounded-xl border border-vscode-border/50 p-4 space-y-3">
            <div className="flex items-center justify-between">
                <h4 className="text-[10px] font-black uppercase tracking-widest text-vscode-fg/40">🔥 Critical Hotspots</h4>
                <div className="w-1.5 h-1.5 rounded-full bg-red-500 animate-pulse"></div>
            </div>
            {hotspots.length === 0 ? (
                <div className="text-xs text-vscode-fg/20 italic font-medium py-2">No symbols analyzed yet...</div>
            ) : (
                <div className="space-y-2">
                    {hotspots.slice(0, 3).map((h, i) => (
                        <div key={i} className="flex items-center gap-3 group/item">
                            <span className="text-[10px] font-black text-vscode-fg/20 w-4">0{i + 1}</span>
                            <span className="flex-1 text-[11px] font-bold text-vscode-fg/60 truncate group-hover/item:text-vscode-accent transition-colors">
                                {h.name.split('::').pop() || h.name}
                            </span>
                            <span className="text-[10px] font-mono font-bold bg-vscode-accent/10 text-vscode-accent px-1.5 py-0.5 rounded border border-vscode-accent/20">
                                {(h.score * 100).toFixed(0)}
                            </span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}

export function VitalsDashboard({
    solidScore,
    betti0,
    betti1,
    hotspots,
    inProgressCount,
    isCacheStale,
    onRescan
}: VitalsDashboardProps) {
    return (
        <div className="p-6 space-y-8 select-none">
            <div className="flex items-center justify-around py-4">
                <SolidScoreGauge score={solidScore} />
                <div className="w-px h-12 bg-vscode-border/30"></div>
                <SpaghettiMeter cycles={betti1} />
            </div>

            <div className="grid grid-cols-2 gap-4">
                <div className="bg-vscode-bg/50 border border-vscode-border/50 rounded-xl p-4 text-center hover:border-vscode-accent/50 transition-all cursor-default group">
                    <div className="text-2xl font-black text-vscode-fg/80 group-hover:text-vscode-accent transition-colors">{betti0}</div>
                    <div className="text-[9px] font-bold uppercase tracking-widest text-vscode-fg/30">Components</div>
                </div>
                <div className="bg-vscode-bg/50 border border-vscode-border/50 rounded-xl p-4 text-center hover:border-vscode-accent/50 transition-all cursor-default group">
                    <div className="text-2xl font-black text-vscode-fg/80 group-hover:text-vscode-accent transition-colors">{inProgressCount}</div>
                    <div className="text-[9px] font-bold uppercase tracking-widest text-vscode-fg/30">In Progress</div>
                </div>
            </div>

            <HotspotsList hotspots={hotspots} />

            {isCacheStale && (
                <div className="flex items-center justify-between p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-xl animate-in slide-in-from-bottom-2 duration-300">
                    <div className="flex items-center gap-2">
                        <span className="text-lg">⚠️</span>
                        <div className="flex flex-col">
                            <span className="text-[10px] font-black uppercase text-yellow-500 leading-none">Cache Stale</span>
                            <span className="text-[9px] font-bold text-yellow-500/60 uppercase tracking-tighter">Topology is out of date</span>
                        </div>
                    </div>
                    {onRescan && (
                        <button
                            className="bg-yellow-500 text-black text-[9px] font-black uppercase px-3 py-1.5 rounded-lg shadow-lg shadow-yellow-500/20 hover:scale-105 active:scale-95 transition-all"
                            onClick={onRescan}
                        >
                            Rescan
                        </button>
                    )}
                </div>
            )}
        </div>
    );
}
