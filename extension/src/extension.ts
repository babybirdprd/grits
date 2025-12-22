import * as vscode from 'vscode';

/**
 * Grits Kanban Editor Provider
 * 
 * Custom editor for .jsonl issue files. Loads issues into a webview
 * with React-based UI for List, Kanban, Graph, and Agenda views.
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
        // Set up webview
        webviewPanel.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(this.context.extensionUri, 'media'),
                vscode.Uri.joinPath(this.context.extensionUri, 'dist'),
                vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist'),
            ],
        };

        // Initial content
        webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

        // Send initial document content to webview
        this.postMessage(webviewPanel.webview, 'update', {
            content: document.getText(),
        });

        // WORKSPACE MODE: Scan for other .grits/issues.jsonl files
        this.scanWorkspaceAndAppend(document, webviewPanel.webview);

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
        const nonce = this.getNonce();

        // Get the local path to main script run in the webview, then convert it to a uri we can use in the webview.
        const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist', 'assets', 'index.js'));

        // Do the same for the stylesheet.
        const styleMainUri = webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, 'webview', 'dist', 'assets', 'index.css'));

        // Get the local path to wasm file
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
                             script-src 'nonce-${nonce}';
                             img-src ${webview.cspSource} data:;
                             connect-src ${webview.cspSource};">
                <title>Grits Kanban</title>
                <link href="${styleMainUri}" rel="stylesheet">
            </head>
            <body>
                <div id="root"></div>
                <script nonce="${nonce}">
                    // Inject VS Code API and WASM URI
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

    private async scanWorkspaceAndAppend(currentDoc: vscode.TextDocument, webview: vscode.Webview) {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders) return;

        // Find all issues.jsonl files
        const files = await vscode.workspace.findFiles('**/.grits/issues.jsonl');

        for (const file of files) {
            // Skip the current document to avoid duplication
            if (file.toString() === currentDoc.uri.toString()) continue;

            try {
                const doc = await vscode.workspace.openTextDocument(file);
                const content = doc.getText();
                if (content.trim()) {
                    this.postMessage(webview, 'update', { content });
                    console.log('Appended content from:', file.toString());
                }
            } catch (e) {
                console.error('Failed to load workspace file:', file, e);
            }
        }

        if (files.length > 1) {
            // Notify UI that we are in workspace mode (optional, for read-only hint)
            // this.postMessage(webview, 'workspace-mode', { count: files.length });
        }
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
