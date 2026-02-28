import init, { Universe } from "./pkg/conway_wasm.js"

const CELL_SIZE = 8
const TICK_INTERVAL = 5

async function main() {
    const canvas = document.getElementById("conway-bg")
    const ctx = canvas.getContext("2d")

    const w = window.innerWidth
    const h = window.innerHeight
    canvas.width = w
    canvas.height = h

    const cols = Math.floor(w / CELL_SIZE)
    const rows = Math.floor(h / CELL_SIZE)

    // pack rgba() css value into a single u32 (format: 0xRRGGBBAA)
    function packRGBA(str) {
        const m = str.match(/[\d.]+/g)
        return (
            ((+m[0]) << 24) |
            ((+m[1]) << 16) |
            ((+m[2]) << 8)  |
            (Math.round((+(m[3] ?? 1)) * 255))
        ) >>> 0
    }

    const style = getComputedStyle(document.documentElement)
    const cAlive = packRGBA(style.getPropertyValue("--conway-alive"))
    const cVisited = packRGBA(style.getPropertyValue("--conway-visited"))

    let wasm = null
    let universe = null
    let running = false
    let animId = null

    const btn = document.getElementById("conway-toggle");
    btn.addEventListener("click", async () => {
        if (running) {
            running = false
            if (animId != null) {
              cancelAnimationFrame(animId)
              animId = null
            }
            ctx.clearRect(0, 0, w, h)
        } else {
            // lazy init here instead of at page load
            if (!wasm) {
                btn.disabled = true
                wasm = await init()
                btn.disabled = false
            }

            running = true
            universe = new Universe(cols, rows)
            startLoop()
        }
    })

    function startLoop() {
        let frame = 0
    
        function loop() {
            if (!running) return
    
            frame++
            if (frame % TICK_INTERVAL !== 0) {
                animId = requestAnimationFrame(loop)
                return
            }
            universe.tick()
    
            // render pixels in wasm, returns pointer to rgba buffer
            // `render(w, h, cell_size, color_alive, color_visited)`
            const ptr = universe.render(w, h, CELL_SIZE, cAlive, cVisited)
            const pixels = new Uint8ClampedArray(wasm.memory.buffer, ptr, w * h * 4)
            const imageData = new ImageData(pixels, w, h)
            ctx.putImageData(imageData, 0, 0)

            animId = requestAnimationFrame(loop)
        }
    
        animId = requestAnimationFrame(loop)
    }
}

main()

