export type Region = {
    x: number,
    y: number,
    w: number,
    h: number
}

export type Config = {
    region?: Region,
    monitor: number,
    text_color: string,
    text_align: string,
    background_color: string,
}
