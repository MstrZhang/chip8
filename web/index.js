import init, * as wasm from "./wasm.js";

const WIDTH = 64;
const HEIGHT = 32;
const SCALE = 15;
const TICKS_PER_FRAME = 10;
let animationFrame = 0;

const canvas = document.getElementById("canvas");
canvas.width = WIDTH * SCALE;
canvas.height = HEIGHT * SCALE;

const ctx = canvas.getContext("2d");
ctx.fillStyle = "black";
ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

const input = document.getElementById("fileinput");

async function run() {
  await init();
  let chip8 = new wasm.EmuWasm();

  document.addEventListener("keydown", (event) => {
    chip8.keypress(event, true);
  });

  document.addEventListener("keyup", (event) => {
    chip8.keypress(event, false);
  });

  input.addEventListener(
    "change",
    (event) => {
      // stop previous game from rendering if one exists
      if (animationFrame != 0) {
        window.cancelAnimationFrame(animationFrame);
      }

      let file = event.target.files[0];
      if (!file) {
        alert("Failed to read file");
        return;
      }

      let fr = new FileReader();
      fr.onload = (_) => {
        // load in game as Uint8Array
        let buffer = fr.result;
        const rom = new Uint8Array(buffer);
        chip8.reset();

        // send to .wasm
        chip8.load_game(rom);

        // start main loop
        mainloop(chip8);
      };
      fr.readAsArrayBuffer(file);
    },
    false
  );
}

function mainloop(chip8) {
  // only draw every few ticks
  for (let i = 0; i < TICKS_PER_FRAME; i++) {
    chip8.tick();
  }
  chip8.tick_timers();

  // clear the canvas before drawing
  ctx.fillStyle = "black";
  ctx.fillRect(0, 0, WIDTH * SCALE, HEIGHT * SCALE);

  // set the draw color back to white before drawing a frame
  ctx.fillStyle = "white";
  chip8.draw_screen(SCALE);

  animationFrame = window.requestAnimationFrame(() => mainloop(chip8));
}

run().catch(console.error);
