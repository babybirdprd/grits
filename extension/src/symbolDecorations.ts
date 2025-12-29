import * as vscode from 'vscode';

/**
 * Symbol Decorations for Editor Gutter
 * 
 * Shows issue indicators in the editor gutter when code symbols
 * are referenced by Grits issues.
 */

interface SymbolIssue {
    issueId: string;
    status: string;
    title: string;
    line: number;
}

// Decoration types for different statuses
const decorationTypes = {
    open: vscode.window.createTextEditorDecorationType({
        gutterIconPath: getGutterIcon('open'),
        gutterIconSize: 'contain',
    }),
    in_progress: vscode.window.createTextEditorDecorationType({
        gutterIconPath: getGutterIcon('in_progress'),
        gutterIconSize: 'contain',
    }),
    blocked: vscode.window.createTextEditorDecorationType({
        gutterIconPath: getGutterIcon('blocked'),
        gutterIconSize: 'contain',
    }),
};

function getGutterIcon(status: string): vscode.Uri {
    // These would be actual SVG icons in the media folder
    // For now, returning a placeholder
    const colors: Record<string, string> = {
        open: '#4a9eff',
        in_progress: '#ffaa44',
        blocked: '#ff4444',
    };

    // Create a data URI for a simple circle SVG
    const color = colors[status] || '#888888';
    const svg = `<svg width="16" height="16" xmlns="http://www.w3.org/2000/svg">
        <circle cx="8" cy="8" r="6" fill="${color}"/>
    </svg>`;

    return vscode.Uri.parse(`data:image/svg+xml,${encodeURIComponent(svg)}`);
}

export class SymbolDecorationProvider {
    private decorations: Map<string, vscode.DecorationOptions[]> = new Map();
    private symbolIssueMap: Map<string, SymbolIssue[]> = new Map();

    constructor(private context: vscode.ExtensionContext) {
        // Listen for active editor changes
        vscode.window.onDidChangeActiveTextEditor(
            (editor) => this.updateDecorations(editor),
            null,
            context.subscriptions
        );

        // Listen for document changes
        vscode.workspace.onDidChangeTextDocument(
            (event) => {
                const editor = vscode.window.activeTextEditor;
                if (editor && event.document === editor.document) {
                    this.updateDecorations(editor);
                }
            },
            null,
            context.subscriptions
        );
    }

    /**
     * Update the symbol-issue mapping from issues list
     */
    public updateSymbolIssueMapping(issues: Array<{ id: string; status: string; title: string; affected_symbols?: string[] }>) {
        this.symbolIssueMap.clear();

        for (const issue of issues) {
            if (!issue.affected_symbols) continue;

            for (const symbol of issue.affected_symbols) {
                // Symbol format: "file/path.ts::FunctionName" or "file/path.ts::ClassName.method"
                const [filePath] = symbol.split('::');

                // We'll need to resolve symbols to line numbers
                // For now, store by file path
                const existing = this.symbolIssueMap.get(filePath) || [];
                existing.push({
                    issueId: issue.id,
                    status: issue.status,
                    title: issue.title,
                    line: 0, // Will be resolved when we have symbol info
                });
                this.symbolIssueMap.set(filePath, existing);
            }
        }

        // Refresh current editor
        this.updateDecorations(vscode.window.activeTextEditor);
    }

    /**
     * Update decorations for the given editor
     */
    private updateDecorations(editor: vscode.TextEditor | undefined) {
        if (!editor) return;

        const filePath = editor.document.uri.fsPath;
        const issues = this.symbolIssueMap.get(filePath) || [];

        // Group by status
        const byStatus: Record<string, vscode.DecorationOptions[]> = {
            open: [],
            in_progress: [],
            blocked: [],
        };

        for (const issue of issues) {
            const line = issue.line || 0;
            const decoration: vscode.DecorationOptions = {
                range: new vscode.Range(line, 0, line, 0),
                hoverMessage: new vscode.MarkdownString(`**[${issue.issueId}]** ${issue.title}`),
            };

            if (byStatus[issue.status]) {
                byStatus[issue.status].push(decoration);
            } else {
                byStatus['open'].push(decoration);
            }
        }

        // Apply decorations
        for (const [status, decorationType] of Object.entries(decorationTypes)) {
            editor.setDecorations(decorationType, byStatus[status] || []);
        }
    }

    /**
     * Dispose all decoration types
     */
    public dispose() {
        for (const dt of Object.values(decorationTypes)) {
            dt.dispose();
        }
    }
}

/**
 * Register the decoration provider
 */
export function registerSymbolDecorations(context: vscode.ExtensionContext): SymbolDecorationProvider {
    return new SymbolDecorationProvider(context);
}
