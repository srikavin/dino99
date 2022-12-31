const loadImage = (path) => {
    const img = new Image();
    img.src = '/sprites/' + path;

    return img;
}

export const sprites = {
    bird1: loadImage("bird1.png"),
    bird2: loadImage("bird2.png"),
    duck1: loadImage("duck1.png"),
    duck2: loadImage("duck2.png"),
    ground: loadImage("ground.png"),
    large_cactus1: loadImage("large_cactus1.png"),
    stand: loadImage("stand.png"),
    cloud: loadImage("cloud.png"),
    run1: loadImage("run1.png"),
    run2: loadImage("run2.png"),
    small_cactus1: loadImage("small_cactus1.png"),
    small_cactus2: loadImage("small_cactus2.png"),
}

export interface Animation {
    frames: HTMLImageElement[],
    frameTimeMs: number
}

export const animations = {
    run: {
        frames: [sprites.run1, sprites.run2],
        frameTimeMs: 150
    },
    bird: {
        frames: [sprites.bird1, sprites.bird2],
        frameTimeMs: 250
    },
    duck: {
        frames: [sprites.duck1, sprites.duck2],
        frameTimeMs: 150
    }
} satisfies Record<string, Animation>;