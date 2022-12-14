export interface EditOptions {
    unlock_pov: boolean,
    cut?: TickRange,
}

export interface TickRange {
    from: number,
    to: number,
}

export async function edit(bytes: Uint8Array, options: EditOptions): Promise<Uint8Array> {
    let m = await import(/* webpackChunkName: "demos-tf-edit" */ "../pkg/index.js");
    return m.edit_js(bytes, options);
}

export async function count_ticks(bytes: Uint8Array): Promise<number> {
    let m = await import(/* webpackChunkName: "demos-tf-edit" */ "../pkg/index.js");
    return m.count_ticks(bytes);
}