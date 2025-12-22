import React from 'react';
import './LabelPicker.css';

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

    const getLabelColor = (label: string) => {
        // Simple consistent color generation based on string hash
        let hash = 0;
        for (let i = 0; i < label.length; i++) {
            hash = label.charCodeAt(i) + ((hash << 5) - hash);
        }
        const c = (hash & 0x00FFFFFF).toString(16).toUpperCase();
        return '#' + '00000'.substring(0, 6 - c.length) + c;
    };

    return (
        <div className="label-picker">
            <div className="label-list">
                {selectedLabels.map(label => (
                    <span
                        key={label}
                        className="label-chip"
                        style={{ backgroundColor: getLabelColor(label) + '40', borderColor: getLabelColor(label) }}
                    >
                        {label}
                        <button onClick={() => onRemoveLabel(label)}>×</button>
                    </span>
                ))}
            </div>
            <div className="label-input-container">
                <input
                    type="text"
                    placeholder="+ Add label"
                    value={inputValue}
                    onChange={handleInputChange}
                    onKeyDown={handleKeyDown}
                />
                {suggestions.length > 0 && (
                    <div className="label-suggestions">
                        {suggestions.map(s => (
                            <div
                                key={s}
                                className="suggestion-item"
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
