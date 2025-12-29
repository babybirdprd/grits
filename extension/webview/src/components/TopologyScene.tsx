import { useRef, useMemo, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Line, Html } from '@react-three/drei';
import * as THREE from 'three';
import './TopologyScene.css';

// Types for topology data
interface TopologyNode {
    id: string;
    name: string;
    file_path: string;
    package?: string;
    kind: string;
    pageRank?: number;
    inCycle?: boolean;
}

interface TopologyData {
    nodes: Record<string, TopologyNode>;
    edges: Array<[string, string, { relation: string; strength: number }]>;
}

interface TopologySceneProps {
    data: TopologyData | null;
    onNodeSelect?: (nodeId: string) => void;
    solidScore?: number;
}

// 3D Node component
function Node3D({
    position,
    node,
    isSelected,
    onClick
}: {
    position: [number, number, number];
    node: TopologyNode;
    isSelected: boolean;
    onClick: () => void;
}) {
    const meshRef = useRef<THREE.Mesh>(null);
    const [hovered, setHovered] = useState(false);

    // Calculate size based on PageRank (default to small if not set)
    const size = (node.pageRank || 0.1) * 2 + 0.3;

    // Color based on kind and cycle status
    const getColor = () => {
        if (node.inCycle) return '#ff4444';
        switch (node.kind) {
            case 'file': return '#4a9eff';
            case 'function': return '#44ff88';
            case 'struct':
            case 'class': return '#ffaa44';
            default: return '#888888';
        }
    };

    // Animate on hover/select
    useFrame(() => {
        if (meshRef.current) {
            const targetScale = hovered || isSelected ? 1.3 : 1;
            meshRef.current.scale.lerp(
                new THREE.Vector3(targetScale, targetScale, targetScale),
                0.1
            );
        }
    });

    return (
        <group position={position}>
            <mesh
                ref={meshRef}
                onClick={(e) => { e.stopPropagation(); onClick(); }}
                onPointerOver={() => setHovered(true)}
                onPointerOut={() => setHovered(false)}
            >
                <sphereGeometry args={[size, 16, 16]} />
                <meshStandardMaterial
                    color={getColor()}
                    emissive={isSelected ? getColor() : '#000000'}
                    emissiveIntensity={isSelected ? 0.5 : 0}
                    metalness={0.3}
                    roughness={0.5}
                />
            </mesh>

            {/* Label on hover */}
            {(hovered || isSelected) && (
                <Html distanceFactor={10}>
                    <div className="node-tooltip">
                        <strong>{node.name}</strong>
                        <span className="node-kind">{node.kind}</span>
                        {node.package && <span className="node-pkg">{node.package}</span>}
                    </div>
                </Html>
            )}

            {/* Cycle indicator ring */}
            {node.inCycle && (
                <mesh rotation={[Math.PI / 2, 0, 0]}>
                    <torusGeometry args={[size + 0.2, 0.05, 8, 32]} />
                    <meshBasicMaterial color="#ff0000" transparent opacity={0.8} />
                </mesh>
            )}
        </group>
    );
}

// 3D Edge component
function Edge3D({
    start,
    end,
    strength,
    relation
}: {
    start: [number, number, number];
    end: [number, number, number];
    strength: number;
    relation: string;
}) {
    // Color based on relation type
    const getColor = () => {
        switch (relation) {
            case 'calls': return '#44ff88';
            case 'imports': return '#4a9eff';
            case 'inherits': return '#ffaa44';
            default: return '#666666';
        }
    };

    const points = useMemo(() => [
        new THREE.Vector3(...start),
        new THREE.Vector3(...end)
    ], [start, end]);

    return (
        <Line
            points={points}
            color={getColor()}
            lineWidth={strength * 2}
            transparent
            opacity={0.6}
        />
    );
}

// Layout algorithm - simple force-directed in 3D
function useLayout(data: TopologyData | null) {
    return useMemo(() => {
        if (!data || Object.keys(data.nodes).length === 0) {
            return { positions: new Map<string, [number, number, number]>() };
        }

        const positions = new Map<string, [number, number, number]>();
        const nodeIds = Object.keys(data.nodes);
        const count = nodeIds.length;

        // Simple spherical layout for now
        // TODO: Implement proper force-directed layout
        nodeIds.forEach((id, i) => {
            const phi = Math.acos(-1 + (2 * i) / count);
            const theta = Math.sqrt(count * Math.PI) * phi;
            const radius = 10 + Math.random() * 5;

            positions.set(id, [
                radius * Math.cos(theta) * Math.sin(phi),
                radius * Math.sin(theta) * Math.sin(phi),
                radius * Math.cos(phi)
            ]);
        });

        return { positions };
    }, [data]);
}

// Main scene content
function SceneContent({
    data,
    onNodeSelect,
    selectedNode
}: {
    data: TopologyData | null;
    onNodeSelect: (id: string) => void;
    selectedNode: string | null;
}) {
    const layout = useLayout(data);

    if (!data) {
        return (
            <Html center>
                <div style={{
                    color: '#666666',
                    fontSize: '14px',
                    textAlign: 'center',
                    padding: '20px'
                }}>
                    No topology data.<br />
                    Run 'gr analysis rebuild' first.
                </div>
            </Html>
        );
    }

    const nodeIds = Object.keys(data.nodes);

    return (
        <>
            {/* Nodes */}
            {nodeIds.map(id => {
                const position = layout.positions.get(id) || [0, 0, 0];
                return (
                    <Node3D
                        key={id}
                        position={position}
                        node={data.nodes[id]}
                        isSelected={selectedNode === id}
                        onClick={() => onNodeSelect(id)}
                    />
                );
            })}

            {/* Edges */}
            {data.edges.map(([source, target, edge], i) => {
                const startPos = layout.positions.get(source);
                const endPos = layout.positions.get(target);
                if (!startPos || !endPos) return null;

                return (
                    <Edge3D
                        key={`${source}-${target}-${i}`}
                        start={startPos}
                        end={endPos}
                        strength={edge.strength}
                        relation={edge.relation}
                    />
                );
            })}
        </>
    );
}

// Main exported component
export function TopologyScene({ data, onNodeSelect, solidScore }: TopologySceneProps) {
    const [selectedNode, setSelectedNode] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    const handleNodeSelect = (id: string) => {
        setSelectedNode(id);
        onNodeSelect?.(id);
    };

    // Get selected node data and its neighborhood
    const selectedNodeData = selectedNode && data?.nodes[selectedNode];
    const starNeighborhood = selectedNode && data?.edges
        ? data.edges
            .filter(([src, tgt]) => src === selectedNode || tgt === selectedNode)
            .map(([src, tgt]) => src === selectedNode ? tgt : src)
            .filter((id, idx, arr) => arr.indexOf(id) === idx)
            .slice(0, 5)
        : [];

    // Copy star neighborhood for agent context
    const copyForAgent = () => {
        if (!selectedNodeData || !data) return;

        const files = [
            selectedNodeData.file_path,
            ...starNeighborhood.map(id => data.nodes[id]?.file_path).filter(Boolean)
        ].filter((v, i, a) => a.indexOf(v) === i); // unique

        const contextText = `## Context: ${selectedNodeData.name}

### Central File
- ${selectedNodeData.file_path}

### Star Neighborhood (related files)
${files.slice(1).map(f => `- ${f}`).join('\n')}

### Topology Info
- PageRank: ${(selectedNodeData.pageRank || 0).toFixed(3)}
- In Cycle: ${selectedNodeData.inCycle ? 'Yes ⚠️' : 'No'}
- Kind: ${selectedNodeData.kind}`;

        navigator.clipboard.writeText(contextText);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="topology-scene">
            {/* Vitals overlay */}
            <div className="vitals-overlay">
                <div className="solid-score">
                    <span className="score-label">Solid Score</span>
                    <span className="score-value" style={{
                        color: (solidScore || 0) > 0.7 ? '#44ff88' :
                            (solidScore || 0) > 0.4 ? '#ffaa44' : '#ff4444'
                    }}>
                        {((solidScore || 0) * 100).toFixed(0)}%
                    </span>
                </div>
            </div>

            {/* Node Inspector Panel */}
            {selectedNodeData && (
                <div className="node-inspector">
                    <div className="inspector-header">
                        <h3>Node Inspector</h3>
                        <button className="close-inspector" onClick={() => setSelectedNode(null)}>×</button>
                    </div>
                    <div className="inspector-content">
                        <div className="inspector-field">
                            <span className="field-label">Symbol</span>
                            <span className="field-value">{selectedNodeData.name}</span>
                        </div>
                        <div className="inspector-field">
                            <span className="field-label">File</span>
                            <span className="field-value file-path">{selectedNodeData.file_path?.split(/[\\/]/).pop()}</span>
                        </div>
                        <div className="inspector-field">
                            <span className="field-label">PageRank</span>
                            <span className="field-value rank-badge">{((selectedNodeData.pageRank || 0) * 100).toFixed(0)}</span>
                        </div>
                        {selectedNodeData.inCycle && (
                            <div className="inspector-warning">⚠️ In Cycle - Consider refactoring</div>
                        )}

                        <div className="inspector-section">
                            <span className="section-label">⭐ Star Neighborhood</span>
                            <ul className="neighbor-list">
                                {starNeighborhood.map(id => (
                                    <li key={id} onClick={() => handleNodeSelect(id)}>
                                        {data?.nodes[id]?.name?.split('::').pop() || id}
                                    </li>
                                ))}
                                {starNeighborhood.length === 0 && <li className="empty">No connections</li>}
                            </ul>
                        </div>

                        <div className="inspector-actions">
                            <button className="action-btn primary" onClick={copyForAgent}>
                                {copied ? '✓ Copied!' : '📋 Copy for Agent'}
                            </button>
                            <button className="action-btn" onClick={() => {
                                // TODO: Integrate with issue creation
                                console.log('Create issue for:', selectedNodeData.name);
                            }}>
                                📝 Create Issue
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Legend */}
            <div className="legend">
                <div className="legend-item">
                    <span className="dot" style={{ background: '#4a9eff' }} />
                    <span>File</span>
                </div>
                <div className="legend-item">
                    <span className="dot" style={{ background: '#44ff88' }} />
                    <span>Function</span>
                </div>
                <div className="legend-item">
                    <span className="dot" style={{ background: '#ffaa44' }} />
                    <span>Class/Struct</span>
                </div>
                <div className="legend-item">
                    <span className="dot" style={{ background: '#ff4444' }} />
                    <span>In Cycle</span>
                </div>
            </div>

            <Canvas
                camera={{ position: [0, 0, 30], fov: 60 }}
                style={{ background: 'linear-gradient(180deg, #1a1a2e 0%, #16213e 100%)' }}
            >
                <ambientLight intensity={0.5} />
                <pointLight position={[10, 10, 10]} intensity={1} />
                <pointLight position={[-10, -10, -10]} intensity={0.5} color="#4a9eff" />

                <SceneContent
                    data={data}
                    onNodeSelect={handleNodeSelect}
                    selectedNode={selectedNode}
                />

                <OrbitControls
                    enableDamping
                    dampingFactor={0.05}
                    minDistance={5}
                    maxDistance={100}
                />
            </Canvas>
        </div>
    );
}

