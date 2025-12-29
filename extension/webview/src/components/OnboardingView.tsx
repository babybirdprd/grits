import React from 'react';

interface OnboardingViewProps {
    onInitialize: () => void;
}

export const OnboardingView: React.FC<OnboardingViewProps> = ({ onInitialize }) => {
    return (
        <div className="flex flex-col items-center justify-center min-h-screen bg-vscode-bg p-8 select-none overflow-hidden">
            <div className="max-w-4xl w-full flex flex-col items-center text-center space-y-12 animate-in fade-in zoom-in duration-500">

                <div className="space-y-4">
                    <div className="text-6xl mb-6 flex justify-center drop-shadow-2xl">⚡</div>
                    <h1 className="text-5xl font-black tracking-tight text-vscode-fg">Welcome to <span className="text-vscode-accent">Grits</span></h1>
                    <p className="text-xl text-vscode-fg/40 font-medium max-w-2xl mx-auto leading-relaxed">
                        The world's most powerful Git-native, local-first issue tracker for autonomous agents and elite developers.
                    </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full">
                    {[
                        { title: 'Twin Engine', desc: 'Syncs SQLite for speed and JSONL for Git-native persistence.', icon: '◇' },
                        { title: 'Offline First', desc: 'Zero latency. Works without internet. Syncs via Git push/pull.', icon: '◎' },
                        { title: 'Agent-Ready', desc: 'Built for LLMs with deep code topology analysis out of the box.', icon: '★' }
                    ].map((feature, i) => (
                        <div key={i} className="bg-vscode-sidebar/30 border border-vscode-border/50 rounded-2xl p-6 text-left hover:border-vscode-accent transition-all group hover:scale-[1.02] cursor-default">
                            <div className="text-2xl text-vscode-accent mb-3 font-bold">{feature.icon}</div>
                            <h3 className="text-lg font-bold text-vscode-fg/80 mb-2 group-hover:text-vscode-accent transition-colors">{feature.title}</h3>
                            <p className="text-sm text-vscode-fg/30 leading-relaxed font-medium">{feature.desc}</p>
                        </div>
                    ))}
                </div>

                <div className="bg-vscode-accent/5 border border-vscode-accent/20 rounded-3xl p-10 w-full max-w-2xl shadow-2xl shadow-vscode-accent/5 backdrop-blur-sm">
                    <div className="space-y-6">
                        <div className="text-sm font-bold uppercase tracking-widest text-vscode-accent">Getting Started</div>
                        <p className="text-lg text-vscode-fg/60 font-medium leading-relaxed">
                            No <code className="bg-vscode-bg px-2 py-0.5 rounded border border-vscode-border/50 text-vscode-accent">.grits</code> folder was found in this workspace. Initializing will set up your local environment.
                        </p>
                        <button
                            className="inline-flex items-center gap-2 px-8 py-4 bg-vscode-accent text-white rounded-2xl text-lg font-black uppercase tracking-widest shadow-xl shadow-vscode-accent/20 hover:scale-[1.05] hover:shadow-vscode-accent/40 active:scale-95 transition-all"
                            onClick={onInitialize}
                        >
                            Initialize Project <span>→</span>
                        </button>
                    </div>
                </div>

                <div className="text-[10px] font-bold uppercase tracking-[0.2em] text-vscode-fg/20">
                    Release v2.4.0 • Local-First Synergy
                </div>
            </div>
        </div>
    );
};
