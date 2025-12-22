import { Issue } from '../types';
import './GraphView.css';

interface GraphViewProps {
    issues: Issue[];
    onSelectIssue: (issue: Issue) => void;
}

/**
 * Simple dependency graph visualization.
 * For a full implementation, use reactflow library.
 * This is a placeholder that shows dependencies as a simple list.
 */
export function GraphView({ issues, onSelectIssue }: GraphViewProps) {
    // Build dependency map
    const issueMap = new Map(issues.map((i) => [i.id, i]));

    // Find issues with dependencies
    const withDeps = issues.filter((i) => i.dependencies.length > 0);

    // Find root issues (no one depends on them) for a tree view
    const dependedOn = new Set(
        issues.flatMap((i) => i.dependencies.map((d) => d.depends_on_id))
    );
    const roots = issues.filter((i) => !dependedOn.has(i.id) && i.status !== 'closed');

    return (
        <div className="graph-view">
            <div className="graph-header">
                <h2>📊 Dependency Graph</h2>
                <p className="graph-subtitle">
                    {issues.length} issues, {withDeps.length} with dependencies
                </p>
            </div>

            <div className="graph-content">
                <section className="graph-section">
                    <h3>🌳 Root Issues (no blockers)</h3>
                    <div className="graph-nodes">
                        {roots.slice(0, 20).map((issue) => (
                            <div
                                key={issue.id}
                                className={`graph-node status-${issue.status}`}
                                onClick={() => onSelectIssue(issue)}
                            >
                                <span className="node-id">{issue.id.slice(0, 8)}</span>
                                <span className="node-title">{issue.title}</span>
                            </div>
                        ))}
                        {roots.length === 0 && (
                            <p className="empty-message">No root issues found</p>
                        )}
                    </div>
                </section>

                <section className="graph-section">
                    <h3>🔗 Dependencies</h3>
                    <div className="dependency-list">
                        {withDeps.map((issue) => (
                            <div key={issue.id} className="dependency-row">
                                <div
                                    className="dep-node source"
                                    onClick={() => onSelectIssue(issue)}
                                >
                                    {issue.id.slice(0, 8)}: {issue.title.slice(0, 40)}
                                </div>
                                <span className="dep-arrow">→</span>
                                <div className="dep-targets">
                                    {issue.dependencies.map((dep) => {
                                        const target = issueMap.get(dep.depends_on_id);
                                        return (
                                            <div
                                                key={dep.depends_on_id}
                                                className={`dep-node target ${dep.type_}`}
                                                onClick={() => target && onSelectIssue(target)}
                                            >
                                                <span className="dep-type">{dep.type_}</span>
                                                <span>{dep.depends_on_id.slice(0, 8)}</span>
                                                {target && (
                                                    <span className="dep-title">
                                                        : {target.title.slice(0, 30)}
                                                    </span>
                                                )}
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>
                        ))}
                        {withDeps.length === 0 && (
                            <p className="empty-message">No dependencies defined</p>
                        )}
                    </div>
                </section>
            </div>

            <div className="graph-footer">
                <p>
                    💡 For interactive graph visualization, install{' '}
                    <code>reactflow</code> and upgrade this component.
                </p>
            </div>
        </div>
    );
}
