import * as vscode from 'vscode';
import { registerSymbolDecorations } from './symbolDecorations';

/**
 * Grits Dashboard Panel
 * 
 * Full-featured webview panel that replaces the sidebar approach.
 * Opens in the editor area with React Three Fiber 3D topology,
 * issue management, and vitals dashboard.
 */
class GritsDashboardPanel {
    public static readonly viewType = 'grits.dashboard';
    public static currentPanel: GritsDashboardPanel | undefined;

    private readonly _panel: vscode.WebviewPanel;
    private readonly _context: vscode.ExtensionContext;
    private _disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, context: vscode.ExtensionContext) {
        this._panel = panel;
        this._context = context;

        // Set up webview
        this._panel.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(context.extensionUri, 'media'),
                vscode.Uri.joinPath(context.extensionUri, 'dist'),
                vscode.Uri.joinPath(context.extensionUri, 'webview', 'dist'),
            ],
        };

        // Set initial content
        this._panel.webview.html = this._getHtmlForWebview();

        // Handle messages from webview
        this._panel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.type) {
                    case 'ready':
                        await this._sendInitialData();
                        break;
                    case 'grCommand':
                        // Execute gr CLI command and return result
                        await this._executeGrCommand(message.command, message.args);
                        break;
                    case 'updateIssue':
                        await this._updateIssue(message.issueId, message.changes);
                        break;
                }
            },
            undefined,
            this._disposables
        );

        // Handle panel disposal
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);
    }

    public static createOrShow(context: vscode.ExtensionContext) {
        const column = vscode.window.activeTextEditor
            ? vscode.window.activeTextEditor.viewColumn
            : undefined;

        // If panel already exists, show it
        if (GritsDashboardPanel.currentPanel) {
            GritsDashboardPanel.currentPanel._panel.reveal(column);
            return;
        }

        // Create new panel
        const panel = vscode.window.createWebviewPanel(
            GritsDashboardPanel.viewType,
            'Grits Dashboard',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
            }
        );

        GritsDashboardPanel.currentPanel = new GritsDashboardPanel(panel, context);
    }

    public dispose() {
        GritsDashboardPanel.currentPanel = undefined;
        this._panel.dispose();
        while (this._disposables.length) {
            const disposable = this._disposables.pop();
            if (disposable) {
                disposable.dispose();
            }
        }
    }

    private async _sendInitialData() {
        // Find issues.jsonl and send to webview
        const files = await vscode.workspace.findFiles('**/.grits/issues.jsonl', null, 1);
        if (files.length > 0) {
            const doc = await vscode.workspace.openTextDocument(files[0]);
            this._panel.webview.postMessage({
                type: 'init',
                issues: doc.getText(),
                workspacePath: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '',
            });
        }

        // Also try to load topology.json if it exists
        const topoFiles = await vscode.workspace.findFiles('**/.grits/topology.json', null, 1);
        if (topoFiles.length > 0) {
            try {
                const topoDoc = await vscode.workspace.openTextDocument(topoFiles[0]);
                this._panel.webview.postMessage({
                    type: 'topology',
                    data: topoDoc.getText(),
                });
            } catch (e) {
                console.log('No topology cache found');
            }
        }
    }

    private async _executeGrCommand(command: string, args: string[]) {
        const { exec } = require('child_process');
        const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || '.';

        const fullCommand = `gr ${command} ${args.join(' ')}`;
        exec(fullCommand, { cwd: workspacePath }, (error: Error | null, stdout: string, stderr: string) => {
            this._panel.webview.postMessage({
                type: 'grResult',
                command,
                success: !error,
                output: stdout || stderr,
            });
        });
    }

    private async _updateIssue(issueId: string, changes: Record<string, string>) {
        // Build gr set command
        const changeArgs = Object.entries(changes).map(([k, v]) => `${k}:${v}`).join(' ');
        await this._executeGrCommand('set', [issueId, changeArgs]);

        // Refresh data
        await this._sendInitialData();
    }

    private _getHtmlForWebview(): string {
        const webview = this._panel.webview;
        const nonce = this._getNonce();

        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._context.extensionUri, 'webview', 'dist', 'assets', 'index.js')
        );
        const styleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._context.extensionUri, 'webview', 'dist', 'assets', 'index.css')
        );
        const wasmUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._context.extensionUri, 'webview', 'dist', 'assets', 'grits_core_bg.wasm')
        );

        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta http-equiv="Content-Security-Policy"
                    content="default-src 'none';
                             style-src ${webview.cspSource} 'unsafe-inline';
                             script-src 'nonce-${nonce}' 'wasm-unsafe-eval';
                             img-src ${webview.cspSource} data: https:;
                             connect-src ${webview.cspSource};
                             font-src ${webview.cspSource};">
                <title>Grits Dashboard</title>
                <link href="${styleUri}" rel="stylesheet">
            </head>
            <body>
                <div id="root"></div>
                <script nonce="${nonce}">
                    window.vscode = acquireVsCodeApi();
                    window.wasmUri = "${wasmUri.toString()}";
                    window.dashboardMode = true;
                </script>
                <script nonce="${nonce}" type="module" src="${scriptUri}"></script>
            </body>
            </html>
        `;
    }

    private _getNonce(): string {
        let text = '';
        const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        for (let i = 0; i < 32; i++) {
            text += possible.charAt(Math.floor(Math.random() * possible.length));
        }
        return text;
    }
}

/**
 * Grits Kanban Editor Provider (kept for .jsonl file editing)
 */
export class GritsEditorProvider implements vscode.CustomTextEditorProvider {
    public static readonly viewType = 'grits.issueTracker';

    constructor(private readonly context: vscode.ExtensionContext) { }

    public static register(context: vscode.ExtensionContext): vscode.Disposable {
        console.log('Registering GritsEditorProvider with viewType:', GritsEditorProvider.viewType);
        const provider = new GritsEditorProvider(context);
        return vscode.window.registerCustomEditorProvider(
            GritsEditorProvider.viewType,
            provider,
            {
                webviewOptions: {
                    retainContextWhenHidden: true,
                },
                supportsMultipleEditorsPerDocument: false,
            }
        );
    }

    public async resolveCustomTextEditor(
        document: vscode.TextDocument,
        webviewPanel: vscode.WebviewPanel,
        _token: vscode.CancellationToken
    ): Promise<void> {
        console.log('Resolving Grits Kanban editor for:', document.uri.toString());
        webviewPanel.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(this.context.extensionUri, 'media'),
                vscode.Uri.joinPath(this.context.extensionUri, 'dist'),
                vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist'),
            ],
        };

        webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

        this.postMessage(webviewPanel.webview, 'update', {
            content: document.getText(),
        });

        webviewPanel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.type) {
                    case 'save':
                        const edit = new vscode.WorkspaceEdit();
                        edit.replace(
                            document.uri,
                            new vscode.Range(0, 0, document.lineCount, 0),
                            message.content
                        );
                        await vscode.workspace.applyEdit(edit);
                        break;

                    case 'ready':
                        this.postMessage(webviewPanel.webview, 'update', {
                            content: document.getText(),
                        });
                        break;
                }
            },
            undefined,
            this.context.subscriptions
        );

        const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
            (e) => {
                if (e.document.uri.toString() === document.uri.toString()) {
                    this.postMessage(webviewPanel.webview, 'update', {
                        content: document.getText(),
                    });
                }
            }
        );

        webviewPanel.onDidDispose(() => {
            changeDocumentSubscription.dispose();
        });

        // Also load topology.json if it exists (for topology view)
        this.loadTopologyData(webviewPanel.webview);
    }

    private async loadTopologyData(webview: vscode.Webview) {
        const topoFiles = await vscode.workspace.findFiles('**/.grits/topology.json', null, 1);
        if (topoFiles.length > 0) {
            try {
                const topoDoc = await vscode.workspace.openTextDocument(topoFiles[0]);
                webview.postMessage({
                    type: 'topology',
                    data: topoDoc.getText(),
                });
            } catch (e) {
                console.log('No topology cache found');
            }
        }
    }

    private postMessage(webview: vscode.Webview, type: string, data: object) {
        webview.postMessage({ type, ...data });
    }

    private getHtmlForWebview(webview: vscode.Webview): string {
        const nonce = this.getNonce();
        const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist', 'assets', 'index.js'));
        const styleMainUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist', 'assets', 'index.css'));
        const wasmUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist', 'assets', 'grits_core_bg.wasm'));

        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta http-equiv="Content-Security-Policy"
                    content="default-src 'none';
                             style-src ${webview.cspSource} 'unsafe-inline';
                             script-src 'nonce-${nonce}' 'wasm-unsafe-eval';
                             img-src ${webview.cspSource} data:;
                             connect-src ${webview.cspSource};">
                <title>Grits Kanban</title>
                <link href="${styleMainUri}" rel="stylesheet">
            </head>
            <body>
                <div id="root"></div>
                <script nonce="${nonce}">
                    window.vscode = acquireVsCodeApi();
                    window.wasmUri = "${wasmUri.toString()}";
                </script>
                <script nonce="${nonce}" type="module" src="${scriptUri}"></script>
            </body>
            </html>
        `;
    }

    private getNonce(): string {
        let text = '';
        const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        for (let i = 0; i < 32; i++) {
            text += possible.charAt(Math.floor(Math.random() * possible.length));
        }
        return text;
    }
}


/**
 * Sidebar Webview View Provider
 * Shows a simple welcome view with a button to open the full dashboard
 */
class GritsSidebarProvider implements vscode.WebviewViewProvider {
    public static readonly viewType = 'grits.welcome';

    constructor(private readonly _context: vscode.ExtensionContext) { }

    public resolveWebviewView(
        webviewView: vscode.WebviewView,
        _context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken
    ): void {
        webviewView.webview.options = {
            enableScripts: true,
        };

        webviewView.webview.html = this._getHtml(webviewView.webview);

        webviewView.webview.onDidReceiveMessage((message) => {
            if (message.command === 'openDashboard') {
                vscode.commands.executeCommand('grits.openDashboard');
            }
        });
    }

    private _getHtml(webview: vscode.Webview): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {
            font-family: var(--vscode-font-family);
            padding: 20px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 200px;
            text-align: center;
        }
        .logo {
            width: 64px;
            height: 64px;
            background: var(--vscode-focusBorder);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 32px;
            font-weight: bold;
            color: white;
            margin-bottom: 16px;
        }
        h2 {
            margin: 0 0 8px 0;
            font-size: 16px;
            color: var(--vscode-foreground);
        }
        p {
            margin: 0 0 20px 0;
            font-size: 12px;
            color: var(--vscode-descriptionForeground);
            line-height: 1.4;
        }
        button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 10px 20px;
            border-radius: 4px;
            font-size: 13px;
            font-weight: 600;
            cursor: pointer;
            width: 100%;
        }
        button:hover {
            background: var(--vscode-button-hoverBackground);
        }
    </style>
</head>
<body>
    <div class="logo">G</div>
    <h2>Grits Studio</h2>
    <p>Agent-aware issue tracking with code topology intelligence</p>
    <button onclick="openDashboard()">Open Dashboard</button>
    <script>
        const vscode = acquireVsCodeApi();
        function openDashboard() {
            vscode.postMessage({ command: 'openDashboard' });
        }
    </script>
</body>
</html>`;
    }
}

/**
 * Extension activation
 */
export function activate(context: vscode.ExtensionContext) {
    // Register custom editor provider (for .jsonl files)
    context.subscriptions.push(GritsEditorProvider.register(context));

    // Register sidebar webview provider
    const sidebarProvider = new GritsSidebarProvider(context);
    context.subscriptions.push(
        vscode.window.registerWebviewViewProvider(GritsSidebarProvider.viewType, sidebarProvider)
    );

    // Register command to open the FULL dashboard panel
    context.subscriptions.push(
        vscode.commands.registerCommand('grits.openDashboard', () => {
            GritsDashboardPanel.createOrShow(context);
        })
    );

    // Legacy command - still opens the custom editor for .jsonl
    context.subscriptions.push(
        vscode.commands.registerCommand('grits.openKanban', () => {
            const activeEditor = vscode.window.activeTextEditor;
            if (activeEditor && activeEditor.document.fileName.endsWith('.jsonl')) {
                vscode.commands.executeCommand(
                    'vscode.openWith',
                    activeEditor.document.uri,
                    GritsEditorProvider.viewType
                );
            } else {
                vscode.window.showInformationMessage(
                    'Open a .jsonl file first to use the Grits Kanban view.'
                );
            }
        })
    );

    // Sidebar click opens dashboard
    context.subscriptions.push(
        vscode.commands.registerCommand('grits.openIssues', () => {
            GritsDashboardPanel.createOrShow(context);
        })
    );

    // Register symbol decorations for gutter icons
    const decorationProvider = registerSymbolDecorations(context);
    context.subscriptions.push({ dispose: () => decorationProvider.dispose() });

    console.log('Grits extension activated');
}

export function deactivate() { }


