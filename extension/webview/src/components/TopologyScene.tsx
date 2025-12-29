import { useRef, useMemo, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Line, Html } from '@react-three/drei';
import * as THREE from 'three';

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
                    <div className="bg-vscode-sidebar/90 backdrop-blur-md border border-vscode-border p-2 rounded shadow-2xl pointer-events-none whitespace-nowrap animate-in fade-in zoom-in-95 duration-150">
                        <strong className="block text-sm">{node.name}</strong>
                        <div className="flex gap-2 items-center mt-1">
                            <span className="text-[10px] font-bold uppercase tracking-wider opacity-60 bg-vscode-bg px-1 rounded">{node.kind}</span>
                            {node.package && <span className="text-[10px] opacity-40 italic">{node.package}</span>}
                        </div>
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
                <div className="text-vscode-fg/40 text-sm text-center p-5 bg-vscode-sidebar/20 backdrop-blur rounded-xl border border-vscode-border/50">
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
        <div className="relative w-full h-full overflow-hidden bg-vscode-bg select-none">
            {/* Vitals overlay */}
            <div className="absolute top-6 left-6 z-10 p-4 bg-vscode-sidebar/40 backdrop-blur-xl border border-white/5 rounded-2xl shadow-2xl">
                <div className="flex flex-col gap-1">
                    <span className="text-[10px] font-bold uppercase tracking-[0.2em] opacity-40">Solid Score</span>
                    <span className="text-3xl font-black tracking-tighter" style={{
                        color: (solidScore || 0) > 0.7 ? '#44ff88' :
                            (solidScore || 0) > 0.4 ? '#ffaa44' : '#ff4444'
                    }}>
                        {((solidScore || 0) * 100).toFixed(0)}%
                    </span>
                </div>
            </div>

            {/* Node Inspector Panel */}
            {selectedNodeData && (
                <div className="absolute top-6 right-6 z-20 w-80 bg-vscode-sidebar/80 backdrop-blur-2xl border border-white/10 rounded-2xl shadow-[0_32px_64px_-12px_rgba(0,0,0,0.5)] animate-in slide-in-from-right-8 duration-300">
                    <div className="p-4 border-b border-white/5 flex justify-between items-center">
                        <h3 className="text-sm font-bold opacity-80 uppercase tracking-widest">Node Inspector</h3>
                        <button className="h-6 w-6 flex items-center justify-center rounded-full hover:bg-white/10 transition-colors" onClick={() => setSelectedNode(null)}>×</button>
                    </div>
                    <div className="p-5 flex flex-col gap-6">
                        <div className="flex flex-col gap-1">
                            <span className="text-[10px] uppercase font-bold tracking-wider opacity-30">Symbol</span>
                            <span className="text-lg font-bold leading-tight truncate">{selectedNodeData.name}</span>
                        </div>
                        <div className="flex flex-col gap-1">
                            <span className="text-[10px] uppercase font-bold tracking-wider opacity-30">File</span>
                            <span className="text-xs font-mono opacity-60 truncate bg-black/20 p-1.5 rounded border border-white/5">{selectedNodeData.file_path?.split(/[\\/]/).pop()}</span>
                        </div>

                        <div className="flex gap-4">
                            <div className="flex-1 flex flex-col gap-1">
                                <span className="text-[10px] uppercase font-bold tracking-wider opacity-30">PageRank</span>
                                <span className="text-xl font-bold text-vscode-accent">{((selectedNodeData.pageRank || 0) * 100).toFixed(0)}</span>
                            </div>
                            {selectedNodeData.inCycle && (
                                <div className="flex-1 flex items-center gap-2 text-xs font-bold text-red-400 bg-red-400/10 p-2 rounded-lg border border-red-400/20">
                                    <span>⚠️</span> In Cycle
                                </div>
                            )}
                        </div>

                        <div className="flex flex-col gap-2">
                            <span className="text-[10px] uppercase font-bold tracking-wider opacity-30">⭐ Star Neighborhood</span>
                            <ul className="flex flex-col gap-1">
                                {starNeighborhood.map(id => (
                                    <li
                                        key={id}
                                        onClick={() => handleNodeSelect(id)}
                                        className="text-xs p-2 rounded-lg hover:bg-white/5 border border-transparent hover:border-white/10 cursor-pointer transition-all truncate"
                                    >
                                        <span className="opacity-40 mr-2">◈</span>
                                        {data?.nodes[id]?.name?.split('::').pop() || id}
                                    </li>
                                ))}
                                {starNeighborhood.length === 0 && <li className="text-xs opacity-30 italic p-2">No connections</li>}
                            </ul>
                        </div>

                        <div className="flex flex-col gap-2 pt-2">
                            <button className="w-full py-2.5 bg-vscode-accent text-white rounded-xl text-xs font-bold shadow-lg shadow-vscode-accent/20 hover:opacity-90 transition-all active:scale-95" onClick={copyForAgent}>
                                {copied ? '✓ Copied!' : '📋 Copy for Agent'}
                            </button>
                            <button className="w-full py-2 bg-white/5 hover:bg-white/10 text-white/70 rounded-xl text-xs font-bold transition-all" onClick={() => {
                                console.log('Create issue for:', selectedNodeData.name);
                            }}>
                                📝 Create Issue
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Legend */}
            <div className="absolute bottom-6 left-6 z-10 flex flex-col gap-3 p-4 bg-vscode-sidebar/40 backdrop-blur-xl border border-white/5 rounded-2xl">
                <div className="flex items-center gap-3">
                    <span className="h-2 w-2 rounded-full shadow-[0_0_8px] shadow-[#4a9eff]" style={{ background: '#4a9eff' }} />
                    <span className="text-[10px] font-bold uppercase tracking-wider opacity-60">File</span>
                </div>
                <div className="flex items-center gap-3">
                    <span className="h-2 w-2 rounded-full shadow-[0_0_8px] shadow-[#44ff88]" style={{ background: '#44ff88' }} />
                    <span className="text-[10px] font-bold uppercase tracking-wider opacity-60">Function</span>
                </div>
                <div className="flex items-center gap-3">
                    <span className="h-2 w-2 rounded-full shadow-[0_0_8px] shadow-[#ffaa44]" style={{ background: '#ffaa44' }} />
                    <span className="text-[10px] font-bold uppercase tracking-wider opacity-60">Class/Struct</span>
                </div>
                <div className="flex items-center gap-3">
                    <span className="h-2 w-2 rounded-full shadow-[0_0_8px] shadow-[#ff4444]" style={{ background: '#ff4444' }} />
                    <span className="text-[10px] font-bold uppercase tracking-wider opacity-60">In Cycle</span>
                </div>
            </div>

            <Canvas
                camera={{ position: [0, 0, 30], fov: 60 }}
                gl={{ antialias: true, alpha: true }}
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

