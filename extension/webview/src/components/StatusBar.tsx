interface StatusBarProps {
    solidScore: number;
    inProgressCount: number;
    blockedCount: number;
}

export function StatusBar({ solidScore, inProgressCount, blockedCount }: StatusBarProps) {
    const scoreColor = solidScore > 0.7 ? 'text-green-400' : solidScore > 0.4 ? 'text-yellow-400' : 'text-red-400';
    const dotColor = solidScore > 0.7 ? 'bg-green-400' : solidScore > 0.4 ? 'bg-yellow-400' : 'bg-red-400';

    return (
        <footer className="h-8 bg-vscode-sidebar border-t border-vscode-border flex items-center justify-between px-4 text-xs text-vscode-fg/60 shrink-0">
            <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${dotColor} animate-pulse`}></span>
                <span>Solid Score: <span className={`font-semibold ${scoreColor}`}>{(solidScore * 100).toFixed(0)}</span></span>
            </div>

            <div className="flex items-center gap-4">
                <span className="flex items-center gap-1.5">
                    <span className="text-vscode-fg/80 font-medium">{inProgressCount}</span> in-progress
                </span>
                <span className="w-px h-3 bg-vscode-border"></span>
                <span className="flex items-center gap-1.5">
                    <span className="text-vscode-fg/80 font-medium">{blockedCount}</span> blocked
                </span>
            </div>

            <div className="flex items-center gap-2 italic">
                <span>↻ Synced just now</span>
            </div>
        </footer>
    );
}
