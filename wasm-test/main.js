import init, { WasmFileSystem, WasmGit } from './pkg/beads_core.js';

async function run() {
    await init();

    console.log("Initializing WasmFileSystem...");
    const fs = new WasmFileSystem();

    console.log("Checking if file exists...");
    const exists = fs.exists("test.txt");
    console.log("Exists:", exists);

    if (!exists) {
        console.log("Writing file...");
        fs.write("test.txt", new TextEncoder().encode("Hello WASM!"));
    }

    console.log("Reading file...");
    const content = fs.read_to_string("test.txt");
    console.log("Content:", content);

    console.log("Initializing WasmGit...");
    const git = new WasmGit();

    console.log("Git Status:");
    const status = git.status();
    console.log(status);
}

run().catch(console.error);
