package main

import (
	"math/rand"
	"strings"
	"syscall/js"
	"time"
)

type CellState uint8

const (
	cellSize = 8
	birthChance = 0.3

	dead CellState = iota
	trailFade
	trailBright
	alive

	// simulation advances once every tickInterval render frames (~60 fps / 6 = 10 ticks/s)
	tickInterval = 6

	// todo: easter eggs
)

var (
	// grid dimensions in cells, derived from the canvas element
	cols, rows int
	grid, next [][]CellState
)

func initGrid(c, r int) {
	cols, rows = c, r
	grid = make([][]CellState, rows)
	next = make([][]CellState, rows)
	for y := range grid {
		grid[y] = make([]CellState, cols)
		next[y] = make([]CellState, cols)
	}
	seed()
}

func seed() {
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	for y := range grid {
		for x := range grid[y] {
			if rng.Float32() < birthChance {
				grid[y][x] = alive
			}
		}
	}
}

func neighbors(x, y int) int {
	n := 0
	for dy := -1; dy <= 1; dy++ {
		for dx := -1; dx <= 1; dx++ {
			if dx == 0 && dy == 0 {
				continue
			}
			if grid[(y+dy+rows)%rows][(x+dx+cols)%cols] == alive {
				n++
			}
		}
	}
	return n
}

func tick() {
	for y := range grid {
		for x := range grid[y] {
			cur := grid[y][x]
			switch cur {
			case alive:
				n := neighbors(x, y)
				if n == 2 || n == 3 {
					next[y][x] = alive
				} else {
					next[y][x] = trailBright
				}
			case trailBright:
				next[y][x] = trailFade
			default: // trailFade and dead both decay to dead
				n := neighbors(x, y)
				if n == 3 {
					next[y][x] = alive
				} else {
					next[y][x] = dead
				}
			}
		}
	}
	grid, next = next, grid
}

func main() {
	doc := js.Global().Get("document")
	win := js.Global().Get("window")

	canvas := doc.Call("getElementById", "conway-bg")
	ctx := canvas.Call("getContext", "2d")

	w := win.Get("innerWidth").Int()
	h := win.Get("innerHeight").Int()
	canvas.Set("width", w)
	canvas.Set("height", h)

	// load colorscheme from css variables
	cssVars := js.Global().Call("getComputedStyle", doc.Get("documentElement"))
	cssVar := func(name string) string {
		return strings.TrimSpace(cssVars.Call("getPropertyValue", name).String())
	}
	colorAlive       := cssVar("--conway-alive")
	colorTrailBright := cssVar("--conway-trail-bright")
	colorTrailFade   := cssVar("--conway-trail-fade")

	startCh := make(chan struct{}, 1)
	stopCh := make(chan struct{}, 1)
	running := false

	btn := doc.Call("getElementById", "conway-toggle")
	toggleFn := js.FuncOf(func(this js.Value, args []js.Value) any {
		if running {
			running = false
			btn.Set("textContent", "Start")
			stopCh <- struct{}{}
		} else {
			running = true
			btn.Set("textContent", "Stop")
			startCh <- struct{}{}
		}
		return nil
	})
	btn.Call("addEventListener", "click", toggleFn)

	for {
		<-startCh

		initGrid(w/cellSize, h/cellSize)

		frame := 0
		stopped := false
		var loop js.Func
		loop = js.FuncOf(func(this js.Value, args []js.Value) any {
			if stopped {
				loop.Release()
				return nil
			}
			frame++
			if frame%tickInterval == 0 {
				tick()
			}
			ctx.Call("clearRect", 0, 0, w, h)

			for _, pass := range []struct {
				state CellState
				color string
			}{
				{trailFade, colorTrailFade},
				{trailBright, colorTrailBright},
				{alive, colorAlive},
			} {
				ctx.Set("fillStyle", pass.color)
				for y := range grid {
					for x := range grid[y] {
						if grid[y][x] == pass.state {
							ctx.Call("fillRect", x*cellSize, y*cellSize, cellSize-1, cellSize-1)
						}
					}
				}
			}

			js.Global().Call("requestAnimationFrame", loop)
			return nil
		})
		js.Global().Call("requestAnimationFrame", loop)

		<-stopCh
		stopped = true
		ctx.Call("clearRect", 0, 0, w, h)
		// loop releases itself on its next (final) RAF invocation
	}
}
