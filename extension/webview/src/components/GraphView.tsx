import React, { useCallback, useEffect } from 'react';
import ReactFlow, {
    Node,
    Edge,
    useNodesState,
    useEdgesState,
    Connection,
    addEdge,
    Controls,
    Background,
    MiniMap,
    Position,
} from 'reactflow';
import 'reactflow/dist/style.css';
import dagre from 'dagre';
import { Issue } from '../types';

interface GraphViewProps {
    issues: Issue[];
    onSelectIssue: (issue: Issue) => void;
}

const nodeWidth = 180;
const nodeHeight = 80;

const getLayoutedElements = (nodes: Node[], edges: Edge[], direction = 'TB') => {
    const dagreGraph = new dagre.graphlib.Graph();
    dagreGraph.setDefaultEdgeLabel(() => ({}));

    const isHorizontal = direction === 'LR';
    dagreGraph.setGraph({ rankdir: direction });

    nodes.forEach((node) => {
        dagreGraph.setNode(node.id, { width: nodeWidth, height: nodeHeight });
    });

    edges.forEach((edge) => {
        dagreGraph.setEdge(edge.source, edge.target);
    });

    dagre.layout(dagreGraph);

    nodes.forEach((node) => {
        const nodeWithPosition = dagreGraph.node(node.id);
        node.targetPosition = isHorizontal ? Position.Left : Position.Top;
        node.sourcePosition = isHorizontal ? Position.Right : Position.Bottom;

        // We are shifting the dagre node position (anchor=center center) to the top left
        // so it matches the React Flow node anchor point (top left).
        node.position = {
            x: nodeWithPosition.x - nodeWidth / 2,
            y: nodeWithPosition.y - nodeHeight / 2,
        };

        return node;
    });

    return { nodes, edges };
};

export const GraphView: React.FC<GraphViewProps> = ({ issues, onSelectIssue }) => {
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    useEffect(() => {
        // Default View: Issue Dependencies
        const initialNodes: Node[] = issues.map(issue => ({
            id: issue.id,
            data: { label: issue.title, status: issue.status },
            position: { x: 0, y: 0 },
            style: {
                background: getStatusColor(issue.status),
                color: '#fff',
                padding: '12px',
                borderRadius: '10px',
                width: nodeWidth,
                fontSize: '12px',
                fontWeight: 'bold',
                border: 'none',
                boxShadow: '0 10px 15px -3px rgb(0 0 0 / 0.1)'
            }
        }));

        const initialEdges: Edge[] = [];
        issues.forEach(issue => {
            if (issue.dependencies) {
                issue.dependencies.forEach(dep => {
                    initialEdges.push({
                        id: `${issue.id}-${dep.depends_on_id}`,
                        source: dep.depends_on_id,
                        target: issue.id,
                        type: 'smoothstep',
                        animated: true,
                        style: { stroke: 'var(--vscode-accent)', strokeWidth: 2 }
                    });
                });
            }
        });

        const { nodes: layoutedNodes, edges: layoutedEdges } = getLayoutedElements(
            initialNodes,
            initialEdges
        );

        setNodes(layoutedNodes);
        setEdges(layoutedEdges);
    }, [issues, setNodes, setEdges]);

    const onConnect = useCallback(
        (params: Connection) => setEdges((eds: Edge[]) => addEdge(params, eds)),
        [setEdges]
    );

    const onNodeClick = (_event: React.MouseEvent, node: Node) => {
        const issue = issues.find(i => i.id === node.id);
        if (issue) {
            onSelectIssue(issue);
        }
    };

    return (
        <div className="w-full h-full bg-vscode-bg relative overflow-hidden flex flex-col">
            <div className="flex-1">
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    onNodesChange={onNodesChange}
                    onEdgesChange={onEdgesChange}
                    onConnect={onConnect}
                    onNodeClick={onNodeClick}
                    fitView
                >
                    <Controls />
                    <MiniMap nodeStrokeWidth={3} zoomable pannable />
                    <Background gap={20} size={1} color="var(--vscode-border)" />
                </ReactFlow>
            </div>
        </div>
    );
};

function getStatusColor(status: string) {
    switch (status) {
        case 'open': return 'var(--status-open)';
        case 'in-progress': return 'var(--status-progress)';
        case 'blocked': return 'var(--status-blocked)';
        case 'closed': return 'var(--status-closed)';
        default: return 'var(--vscode-fg)';
    }
}
