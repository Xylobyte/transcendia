export type Region = {
    x: number,
    y: number,
    w: number,
    h: number
}

export type Config = {
    region?: Region,
    monitor: string,
    text_color: string,
    text_align:
        | 'T:L'
        | 'T:C'
        | 'T:R'
        | 'C:L'
        | 'C:C'
        | 'C:R'
        | 'B:L'
        | 'B:C'
        | 'B:R',
    background_color: string,
    blur_background: boolean,
    interval: number
}
