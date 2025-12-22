import React from 'react';
import './OnboardingView.css';

interface OnboardingViewProps {
    onInitialize: () => void;
}

export const OnboardingView: React.FC<OnboardingViewProps> = ({ onInitialize }) => {
    return (
        <div className="onboarding-view">
            <div className="onboarding-content">
                <div className="logo">📋</div>
                <h1>Welcome to Grits</h1>
                <p className="subtitle">Git-native, local-first issue tracking.</p>

                <div className="features">
                    <div className="feature">
                        <h3>Twin Engine</h3>
                        <p>Issues live in your code (JSONL) and in your database.</p>
                    </div>
                    <div className="feature">
                        <h3>Offline First</h3>
                        <p>Works without internet. Syncs via Git.</p>
                    </div>
                    <div className="feature">
                        <h3>Visual & CLI</h3>
                        <p>Manage issues from VS Code or the terminal.</p>
                    </div>
                </div>

                <div className="actions">
                    <p>No <code>.grits</code> folder found in this workspace.</p>
                    <button className="primary-btn" onClick={onInitialize}>
                        Initialize Grits Project
                    </button>
                </div>
            </div>
        </div>
    );
};
