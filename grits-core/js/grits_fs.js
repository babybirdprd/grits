export function fs_read_to_string(path) {
    console.log(`[JS] fs_read_to_string: ${path}`);
    return "";
}
export function fs_write(path, content) {
    console.log(`[JS] fs_write: ${path}, ${content.length} bytes`);
}
export function fs_create_dir_all(path) {
    console.log(`[JS] fs_create_dir_all: ${path}`);
}
export function fs_rename(from, to) {
    console.log(`[JS] fs_rename: ${from} -> ${to}`);
}
export function fs_exists(path) {
    console.log(`[JS] fs_exists: ${path}`);
    return false;
}
