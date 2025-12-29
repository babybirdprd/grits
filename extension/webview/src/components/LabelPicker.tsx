import React from 'react';

interface LabelPickerProps {
    selectedLabels: string[];
    availableLabels: string[];
    onAddLabel: (label: string) => void;
    onRemoveLabel: (label: string) => void;
}

export const LabelPicker: React.FC<LabelPickerProps> = ({
    selectedLabels,
    availableLabels,
    onAddLabel,
    onRemoveLabel,
}) => {
    const [inputValue, setInputValue] = React.useState('');
    const [suggestions, setSuggestions] = React.useState<string[]>([]);

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value;
        setInputValue(val);
        if (val) {
            setSuggestions(
                availableLabels.filter(
                    l => l.toLowerCase().includes(val.toLowerCase()) && !selectedLabels.includes(l)
                )
            );
        } else {
            setSuggestions([]);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && inputValue.trim()) {
            onAddLabel(inputValue.trim());
            setInputValue('');
            setSuggestions([]);
        }
    };

    return (
        <div className="relative space-y-3">
            <div className="flex flex-wrap gap-2 min-h-[32px]">
                {selectedLabels.map(label => (
                    <span
                        key={label}
                        className="group flex items-center gap-1.5 px-2 py-0.5 bg-vscode-accent/10 text-vscode-accent text-[10px] font-bold rounded-md border border-vscode-accent/20 uppercase tracking-tighter"
                    >
                        {label}
                        <button
                            className="w-3.5 h-3.5 flex items-center justify-center rounded-full hover:bg-vscode-accent/20 transition-colors"
                            onClick={() => onRemoveLabel(label)}
                        >
                            ✕
                        </button>
                    </span>
                ))}
            </div>

            <div className="relative">
                <input
                    type="text"
                    className="w-full bg-vscode-sidebar/50 border border-vscode-border rounded-lg px-3 py-1.5 text-xs focus:outline-none focus:ring-1 focus:ring-vscode-accent transition-all placeholder-vscode-fg/20"
                    placeholder="+ Add label (Press Enter)"
                    value={inputValue}
                    onChange={handleInputChange}
                    onKeyDown={handleKeyDown}
                />

                {suggestions.length > 0 && (
                    <div className="absolute z-100 bottom-full mb-2 left-0 right-0 bg-vscode-sidebar border border-vscode-border rounded-lg shadow-2xl overflow-hidden animate-in fade-in slide-in-from-bottom-2 duration-150">
                        {suggestions.map(s => (
                            <div
                                key={s}
                                className="px-3 py-2 text-xs font-medium text-vscode-fg/60 hover:bg-vscode-accent hover:text-white cursor-pointer transition-colors"
                                onClick={() => {
                                    onAddLabel(s);
                                    setInputValue('');
                                    setSuggestions([]);
                                }}
                            >
                                {s}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};
