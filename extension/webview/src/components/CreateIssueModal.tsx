import React, { useState } from 'react';
import './CreateIssueModal.css';

interface CreateIssueModalProps {
    onClose: () => void;
    onCreate: (title: string, description: string, type: string, priority: number) => void;
    templates: string[];
    onLoadTemplate: (templateName: string) => Promise<string>;
}

export function CreateIssueModal({ onClose, onCreate, templates, onLoadTemplate }: CreateIssueModalProps) {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');
    const [type, setType] = useState('bug');
    const [priority, setPriority] = useState(2);
    const [selectedTemplate, setSelectedTemplate] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        onCreate(title, description, type, priority);
        onClose();
    };

    const handleTemplateChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
        const tmpl = e.target.value;
        setSelectedTemplate(tmpl);
        if (tmpl) {
            const content = await onLoadTemplate(tmpl);
            setDescription(content);
        }
    };

    return (
        <div className="modal-overlay">
            <div className="modal-content">
                <div className="modal-header">
                    <h2>New Issue</h2>
                    <button className="close-btn" onClick={onClose}>✕</button>
                </div>
                <form onSubmit={handleSubmit}>
                    <div className="form-group">
                        <label>Title</label>
                        <input
                            type="text"
                            value={title}
                            onChange={(e) => setTitle(e.target.value)}
                            required
                            autoFocus
                        />
                    </div>

                    <div className="form-row">
                        <div className="form-group">
                            <label>Type</label>
                            <select value={type} onChange={(e) => setType(e.target.value)}>
                                <option value="bug">Bug</option>
                                <option value="feature">Feature</option>
                                <option value="task">Task</option>
                                <option value="epic">Epic</option>
                            </select>
                        </div>
                        <div className="form-group">
                            <label>Priority</label>
                            <select value={priority} onChange={(e) => setPriority(parseInt(e.target.value))}>
                                <option value={1}>P1 - Critical</option>
                                <option value={2}>P2 - High</option>
                                <option value={3}>P3 - Normal</option>
                                <option value={4}>P4 - Low</option>
                                <option value={5}>P5 - Trivial</option>
                            </select>
                        </div>
                    </div>

                    <div className="form-group">
                        <label>Template</label>
                        <select value={selectedTemplate} onChange={handleTemplateChange}>
                            <option value="">None</option>
                            {templates.map(t => <option key={t} value={t}>{t}</option>)}
                        </select>
                    </div>

                    <div className="form-group full-height">
                        <label>Description</label>
                        <textarea
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                        />
                    </div>

                    <div className="modal-footer">
                        <button type="button" onClick={onClose}>Cancel</button>
                        <button type="submit" className="primary-btn">Create Issue</button>
                    </div>
                </form>
            </div>
        </div>
    );
}
