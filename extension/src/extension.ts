import * as vscode from 'vscode';

/**
 * Grits Kanban Editor Provider
 * 
 * Custom editor for .jsonl issue files. Loads issues into a webview
 * with React-based UI for List, Kanban, Graph, and Agenda views.
 */
export class GritsEditorProvider implements vscode.CustomTextEditorProvider {
    public static readonly viewType = 'grits.kanban';

    constructor(private readonly context: vscode.ExtensionContext) { }

    public static register(context: vscode.ExtensionContext): vscode.Disposable {
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
        // Set up webview
        webviewPanel.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(this.context.extensionUri, 'media'),
                vscode.Uri.joinPath(this.context.extensionUri, 'dist'),
            ],
        };

        // Initial content
        webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

        // Send initial document content to webview
        this.postMessage(webviewPanel.webview, 'update', {
            content: document.getText(),
        });

        // Handle messages from webview
        webviewPanel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.type) {
                    case 'save':
                        // Apply edit from webview to document
                        const edit = new vscode.WorkspaceEdit();
                        edit.replace(
                            document.uri,
                            new vscode.Range(0, 0, document.lineCount, 0),
                            message.content
                        );
                        await vscode.workspace.applyEdit(edit);
                        break;

                    case 'ready':
                        // Webview is ready, send current content
                        this.postMessage(webviewPanel.webview, 'update', {
                            content: document.getText(),
                        });
                        break;
                }
            },
            undefined,
            this.context.subscriptions
        );

        // Handle document changes (external edits)
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
    }

    private postMessage(webview: vscode.Webview, type: string, data: object) {
        webview.postMessage({ type, ...data });
    }

    private getHtmlForWebview(webview: vscode.Webview): string {
        // TODO: Replace with actual React build output
        const nonce = this.getNonce();

        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta http-equiv="Content-Security-Policy"
                    content="default-src 'none';
                             style-src ${webview.cspSource} 'unsafe-inline';
                             script-src 'nonce-${nonce}';
                             img-src ${webview.cspSource} data:;">
                <title>Grits Kanban</title>
                <style>
                    body {
                        padding: 20px;
                        font-family: var(--vscode-font-family);
                        color: var(--vscode-foreground);
                        background: var(--vscode-editor-background);
                    }
                    .loading {
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        height: 100vh;
                        font-size: 1.2em;
                        color: var(--vscode-descriptionForeground);
                    }
                    #root {
                        height: 100%;
                    }
                </style>
            </head>
            <body>
                <div id="root">
                    <div class="loading">Loading Grits UI...</div>
                </div>
                <script nonce="${nonce}">
                    const vscode = acquireVsCodeApi();
                    
                    // Handle messages from extension
                    window.addEventListener('message', event => {
                        const message = event.data;
                        if (message.type === 'update') {
                            // TODO: Pass to React app when loaded
                            console.log('Received issues:', message.content);
                        }
                    });

                    // Signal that webview is ready
                    vscode.postMessage({ type: 'ready' });
                </script>
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
 * Extension activation
 */
export function activate(context: vscode.ExtensionContext) {
    // Register custom editor provider
    context.subscriptions.push(GritsEditorProvider.register(context));

    // Register command to open kanban view
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

    console.log('Grits Kanban extension activated');
}

export function deactivate() { }
