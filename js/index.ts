export interface EditOptions {
    unlock_pov: boolean,
    cut?: TickRange,
}

export interface TickRange {
    from: number,
    to: number,
}

export async function edit(bytes: Uint8Array, options: EditOptions): Promise<Uint8Array> {
    let m = await import("../pkg/index.js");
    return m.edit(bytes, options);
}