import init, {GameClient, Input} from "../../client/pkg/client.js";
import {sendMessage, setAllMessageHandler, ws} from "./websocket";
import {Animation, animations, sprites} from './sprites'

let client: GameClient | undefined;
init().then(() => {
    client = new GameClient("test_user", "lobby");
    setAllMessageHandler(client.on_message.bind(client));
});

type GameState = {
    score: number,
    tick: number,
    player: {
        y: number,
        is_ducked: boolean
    },
    obstacles: {
        category: 'Bird' | 'Cactus',
        position: {
            x: number,
            y: number
        }
    }[]
};

type GameStateReturn = [
    string,
    Map<string, GameState>
]

const GAME_HEIGHT = 120;
const GAME_WIDTH = 600;
const DINO_X = 50;

const setupCanvas = (drawFn: (dt: number, h: number, w: number, ctx: CanvasRenderingContext2D, canvas: HTMLCanvasElement, totalTime: number) => void) => {
    const canvas = document.getElementById("canvas") as HTMLCanvasElement;
    const ctx = canvas.getContext("2d");


    let lastInput = Input.None;
    canvas.addEventListener('keydown', (e) => {
        if (e.code === 'Space' || e.code === 'ArrowUp') {
            lastInput = Input.Jump;
        } else if (e.code === 'ArrowDown') {
            lastInput = Input.Duck;
        }
    });
    canvas.addEventListener('keyup', (e) => {
        if ((e.code === 'Space' || e.code === 'ArrowUp') && lastInput == Input.Jump) {
            lastInput = Input.None;
        } else if (e.code === 'ArrowDown') {
            lastInput = Input.Unduck;
        }
    });

    let lastTick = 0;
    setInterval(() => {
        if (!client) {
            return;
        }
        let timestamp = Date.now();

        console.log('tickDelta', timestamp - lastTick)
        client.tick(lastInput);
        lastTick = timestamp;

        lastInput = Input.None;
    }, 50)

    let lastRender = 0;
    let totalTime = 0;
    const callback = timestamp => {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;

        let dt = lastRender == 0 ? 0 : (timestamp - lastRender);
        totalTime += dt;
        lastRender = timestamp;

        if (client) {
            drawFn(dt, canvas.height, canvas.width, ctx, canvas, totalTime);
        }
        requestAnimationFrame(callback);
    }

    requestAnimationFrame(callback);
}

type State = 'title' | 'playing';
let state: State = 'title';

const uiRoot = document.getElementById("root");
const play_button = document.getElementById("play-button") as HTMLButtonElement;
play_button.onclick = () => {
    state = 'playing';
}

setupCanvas((dt, h, w, ctx, canvas, totalTime) => {
    const drawText = (text: string, x, y, options: { xalign?: 'left' | 'right' | 'center', style?: string }) => {
        ctx.save();
        ctx.font = '20px arial'
        if (options.style) {
            ctx.font = options.style;
        }
        const split = text.split("\n");

        let renderX = x;
        let renderY = y;
        for (const line of split) {
            const metrics = ctx.measureText(line);
            if (options.xalign === 'right') {
                renderX -= metrics.width;
            }
            if (options.xalign === 'center') {
                renderX -= metrics.width / 2;
                renderY -= metrics.actualBoundingBoxAscent / 2;
            }
            ctx.fillText(line, renderX, renderY + metrics.actualBoundingBoxAscent)

            renderY += metrics.actualBoundingBoxAscent + 10;
            renderX = x;
        }
        ctx.restore();
    }

    const renderImage = (image: HTMLImageElement, x, y) => {
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(image, x, y - image.height);
    }

    const renderAnimation = (animation: Animation, x, y) => {
        const curFrameIdx = Math.floor((totalTime / animation.frameTimeMs)) % animation.frames.length;
        const curFrame = animation.frames[curFrameIdx];
        renderImage(curFrame, x, y);
    }

    ctx.clearRect(0, 0, w, h);

    if (state === 'title') {
        uiRoot.style.display = 'default';
        canvas.style.display = 'none';
        return;
    } else {
        canvas.style.display = 'block';
        uiRoot.style.display = 'none';
        canvas.focus();
    }

    const fps = Math.round(1000 / dt);

    const renderState = client.game_state() as GameStateReturn;
    console.log(renderState)

    if (renderState === null) {
        drawText(`Connecting to Lobby Server` + '.'.repeat((totalTime / 300) % 4), w / 2, h / 2, {
            xalign: "center",
            style: '45px arial'
        });
        return;
    }

    let [uuid, states] = renderState;
    let localState = states.get(uuid);

    if (!localState) return;

    drawText(`ws: ${ws.readyState === ws.OPEN ? "connected" : "disconnected"}\n` +
        `fps: ${fps}\n` +
        `tick: ${localState.tick}`, w - 10, 10, {xalign: 'right'});


    const renderGameArea = (render_state: GameState) => {
        const {y: gameY, is_ducked} = render_state.player;

        let realY = GAME_HEIGHT - gameY;

        ctx.strokeStyle = 'gray';
        ctx.strokeRect(0, 0, GAME_WIDTH, GAME_HEIGHT + 5);
        ctx.fillRect(0, 0, GAME_WIDTH, GAME_HEIGHT + 5);

        ctx.drawImage(sprites.ground, (localState.tick * 10) % (sprites.ground.width - GAME_WIDTH), 0, GAME_WIDTH, sprites.ground.height, 0, GAME_HEIGHT - 15, GAME_WIDTH, sprites.ground.height);

        if (localState.tick == 0) {
            renderImage(sprites.stand, DINO_X, realY);
        } else if (is_ducked) {
            renderAnimation(animations.duck, DINO_X, realY);
        } else {
            renderAnimation(animations.run, DINO_X, realY);
        }

        for (let i = 0; i < 5; ++i) {
            let offset = (i * 71) % GAME_WIDTH;
            let x = (offset + totalTime / (-100 - i * 5));
            if (x < 0) {
                x += GAME_WIDTH - sprites.cloud.width;
            }
            renderImage(sprites.cloud, x, 15 + 10 * (i * 31 % 7))
        }

        for (let obstacle of render_state.obstacles) {
            if (obstacle.position.x > GAME_WIDTH) continue;
            if (obstacle.position.y > GAME_HEIGHT) continue;
            if (obstacle.category === 'Bird') {
                renderAnimation(animations.bird, obstacle.position.x, obstacle.position.y);
            }
            if (obstacle.category === 'Cactus') {
                renderImage(sprites.small_cactus1, obstacle.position.x, obstacle.position.y);
            }
        }
    }

    let i = 0;
    let j = 0;
    let scale = 0.4;
    for (let [id, gameState] of states.entries()) {
        if (id === uuid) continue;

        ctx.save();
        ctx.fillStyle = 'transparent';
        ctx.scale(scale, scale);
        ctx.translate(100 + i * (GAME_WIDTH + 100), 100 + j * (GAME_HEIGHT + 100));
        renderGameArea(gameState);
        ctx.restore();

        if (scale * (100 + (i + 2) * (GAME_WIDTH + 100)) > w) {
            i = 0;
            j += 1;
        } else {
            i += 1;
        }
    }


    ctx.save();
    ctx.translate((w - GAME_WIDTH) / 2, (h - GAME_HEIGHT) / 2);
    ctx.fillStyle = 'white';
    renderGameArea(localState);
    ctx.restore();
});
