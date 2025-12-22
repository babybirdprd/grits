import React, { useCallback, useMemo } from 'react';
import ReactFlow, {
  Node,
  Edge,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  Connection,
  addEdge,
  Handle,
  Position,
  NodeProps,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Issue } from '../types';

interface GraphViewProps {
    issues: Issue[];
    onSelectIssue: (issue: Issue | null) => void;
}

const CustomNode = ({ data }: NodeProps) => {
    const statusColors: Record<string, string> = {
        open: '#e2e2e2',
        in_progress: '#3b82f6',
        blocked: '#ef4444',
        closed: '#22c55e',
    };
    const statusColor = statusColors[data.status as string] || '#e2e2e2';

    return (
        <div style={{
            padding: '10px',
            borderRadius: '5px',
            border: `2px solid ${statusColor}`,
            background: 'var(--vscode-editor-background)',
            color: 'var(--vscode-editor-foreground)',
            width: '150px',
            fontSize: '12px',
        }}>
            <Handle type="target" position={Position.Top} />
            <div style={{ fontWeight: 'bold', marginBottom: '4px' }}>{data.label}</div>
            <div style={{ fontSize: '10px', color: 'var(--vscode-descriptionForeground)' }}>{data.id}</div>
            <Handle type="source" position={Position.Bottom} />
        </div>
    );
};

const nodeTypes = {
  custom: CustomNode,
};

export function GraphView({ issues, onSelectIssue }: GraphViewProps) {
    // Transform issues to nodes and edges
    const initialNodes: Node[] = useMemo(() => {
        return issues.map((issue, index) => ({
            id: issue.id,
            type: 'custom',
            data: { label: issue.title, status: issue.status, id: issue.id },
            position: { x: (index % 5) * 200, y: Math.floor(index / 5) * 100 },
        }));
    }, [issues]);

    const initialEdges: Edge[] = useMemo(() => {
        const edges: Edge[] = [];
        issues.forEach(issue => {
            issue.dependencies.forEach(dep => {
                edges.push({
                    id: `${issue.id}-${dep.depends_on_id}`,
                    source: dep.depends_on_id, // depends_on means the other is a prerequisite (source)
                    target: issue.id,
                    animated: dep.type_ === 'blocking',
                    style: { stroke: dep.type_ === 'blocking' ? '#ef4444' : '#999' },
                });
            });
        });
        return edges;
    }, [issues]);

    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

    // Sync nodes when issues change, preserving positions if possible
    React.useEffect(() => {
        setNodes(nds => {
            const currentPositions = new Map(nds.map(n => [n.id, n.position]));
            return issues.map((issue, index) => ({
                id: issue.id,
                type: 'custom',
                data: { label: issue.title, status: issue.status, id: issue.id },
                position: currentPositions.get(issue.id) || { x: (index % 5) * 200, y: Math.floor(index / 5) * 150 },
            }));
        });

        // Rebuild edges completely
        const newEdges: Edge[] = [];
        issues.forEach(issue => {
            issue.dependencies.forEach(dep => {
                newEdges.push({
                    id: `${issue.id}-${dep.depends_on_id}`,
                    source: dep.depends_on_id,
                    target: issue.id,
                    animated: dep.type_ === 'blocking',
                    style: { stroke: dep.type_ === 'blocking' ? '#ef4444' : '#999' },
                });
            });
        });
        setEdges(newEdges);
    }, [issues, setNodes, setEdges]);

    const onConnect = useCallback((params: Connection) => setEdges((eds) => addEdge(params, eds)), [setEdges]);

    const onNodeClick = (_: React.MouseEvent, node: Node) => {
        const issue = issues.find(i => i.id === node.id);
        onSelectIssue(issue || null);
    };

    return (
        <div style={{ width: '100%', height: '100%' }}>
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onConnect={onConnect}
                nodeTypes={nodeTypes}
                onNodeClick={onNodeClick}
                fitView
            >
                <Background />
                <Controls />
                <MiniMap />
            </ReactFlow>
        </div>
    );
}
