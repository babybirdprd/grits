import './VitalsDashboard.css';

interface VitalsDashboardProps {
    solidScore: number;
    betti0: number;  // Connected components
    betti1: number;  // Cycles
    hotspots: Array<{ name: string; score: number }>;
    inProgressCount: number;
    isCacheStale?: boolean;
    onRescan?: () => void;
}

// Circular gauge component for Solid Score
function SolidScoreGauge({ score }: { score: number }) {
    const percentage = Math.round(score * 100);

    const getColor = () => {
        if (score >= 0.7) return '#44ff88';
        if (score >= 0.4) return '#ffaa44';
        return '#ff4444';
    };

    return (
        <div className="solid-score-gauge">
            <svg viewBox="0 0 100 50" className="gauge-svg">
                {/* Background arc */}
                <path
                    d="M 5 50 A 45 45 0 0 1 95 50"
                    fill="none"
                    stroke="rgba(255,255,255,0.1)"
                    strokeWidth="8"
                    strokeLinecap="round"
                />
                {/* Score arc */}
                <path
                    d="M 5 50 A 45 45 0 0 1 95 50"
                    fill="none"
                    stroke={getColor()}
                    strokeWidth="8"
                    strokeLinecap="round"
                    strokeDasharray={`${score * 141.3} 141.3`}
                    className="gauge-fill"
                />
            </svg>
            <div className="gauge-value" style={{ color: getColor() }}>
                {percentage}%
            </div>
            <div className="gauge-label">Solid Score</div>
        </div>
    );
}

// Spaghetti meter (cycles indicator)
function SpaghettiMeter({ cycles }: { cycles: number }) {
    const intensity = Math.min(cycles / 10, 1);
    const noodleCount = Math.min(cycles, 5);

    return (
        <div className="spaghetti-meter">
            <div className="spaghetti-icon">
                {Array.from({ length: noodleCount }).map((_, i) => (
                    <span
                        key={i}
                        className="noodle"
                        style={{
                            opacity: 0.5 + intensity * 0.5,
                            transform: `rotate(${-20 + i * 10}deg)`
                        }}
                    >
                        🍝
                    </span>
                ))}
            </div>
            <div className="meter-value">{cycles}</div>
            <div className="meter-label">Cycles (B₁)</div>
        </div>
    );
}

// Hotspots list 
function HotspotsList({ hotspots }: { hotspots: Array<{ name: string; score: number }> }) {
    return (
        <div className="hotspots-list">
            <h4>🔥 Hotspots</h4>
            {hotspots.length === 0 ? (
                <div className="empty-state">No symbols analyzed</div>
            ) : (
                <ul>
                    {hotspots.slice(0, 3).map((h, i) => (
                        <li key={i} className="hotspot-item">
                            <span className="rank">#{i + 1}</span>
                            <span className="name" title={h.name}>
                                {h.name.split('::').pop() || h.name}
                            </span>
                            <span className="score">{(h.score * 100).toFixed(0)}</span>
                        </li>
                    ))}
                </ul>
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
        <div className="vitals-dashboard">
            <div className="vitals-row">
                <SolidScoreGauge score={solidScore} />
                <SpaghettiMeter cycles={betti1} />
            </div>

            <div className="vitals-stats">
                <div className="stat-item">
                    <span className="stat-value">{betti0}</span>
                    <span className="stat-label">Components</span>
                </div>
                <div className="stat-item">
                    <span className="stat-value">{inProgressCount}</span>
                    <span className="stat-label">In Progress</span>
                </div>
            </div>

            <HotspotsList hotspots={hotspots} />

            {isCacheStale && (
                <div className="stale-warning">
                    <span>⚠️ Topology cache is stale</span>
                    {onRescan && (
                        <button onClick={onRescan}>Rescan</button>
                    )}
                </div>
            )}
        </div>
    );
}
