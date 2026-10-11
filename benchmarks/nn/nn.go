package main

import (
	"fmt"
	"math"
	"time"
)

type Layer struct {
	nIn, nOut int
	act       int
	w, b      []float64
	lastIn    []float64
	lastOut   []float64
}

func xavier(rng *uint32, nIn, nOut int) float64 {
	*rng = *rng*1103515245 + 12345
	r := float64(int(*rng%2000000)-1000000) / 1000000.0
	limit := math.Sqrt(6.0 / float64(nIn+nOut))
	return r * limit
}

func newLayer(nIn, nOut, act int, rng *uint32) *Layer {
	w := make([]float64, nIn*nOut)
	for i := range w {
		w[i] = xavier(rng, nIn, nOut)
	}
	return &Layer{nIn: nIn, nOut: nOut, act: act, w: w, b: make([]float64, nOut), lastIn: make([]float64, nIn), lastOut: make([]float64, nOut)}
}

func (l *Layer) forward(x, y []float64) {
	copy(l.lastIn, x)
	for j := 0; j < l.nOut; j++ {
		z := l.b[j]
		for i := 0; i < l.nIn; i++ {
			z += x[i] * l.w[i*l.nOut+j]
		}
		if l.act == 1 {
			if z > 0 {
				y[j] = z
			} else {
				y[j] = 0
			}
		} else {
			y[j] = 1.0 / (1.0 + math.Exp(-z))
		}
		l.lastOut[j] = y[j]
	}
}

func (l *Layer) backward(gout, gin []float64, lr float64) {
	gz := make([]float64, l.nOut)
	for j := 0; j < l.nOut; j++ {
		y := l.lastOut[j]
		da := 1.0
		if l.act == 1 {
			if y > 0 {
				da = 1
			} else {
				da = 0
			}
		} else {
			da = y * (1 - y)
		}
		gz[j] = gout[j] * da
	}
	if gin != nil {
		for i := 0; i < l.nIn; i++ {
			s := 0.0
			for j := 0; j < l.nOut; j++ {
				s += gz[j] * l.w[i*l.nOut+j]
			}
			gin[i] = s
		}
	}
	for i := 0; i < l.nIn; i++ {
		for j := 0; j < l.nOut; j++ {
			l.w[i*l.nOut+j] -= lr * l.lastIn[i] * gz[j]
		}
	}
	for j := 0; j < l.nOut; j++ {
		l.b[j] -= lr * gz[j]
	}
}

func main() {
	rng := uint32(12345)
	l1 := newLayer(2, 4, 1, &rng)
	l2 := newLayer(4, 1, 2, &rng)
	X := [][]float64{{0, 0}, {0, 1}, {1, 0}, {1, 1}}
	Y := []float64{0, 1, 1, 0}
	lr := 0.5
	start := time.Now()
	for epoch := 0; epoch < 10000; epoch++ {
		for k := 0; k < 4; k++ {
			h := make([]float64, 4)
			y := make([]float64, 1)
			l1.forward(X[k], h)
			l2.forward(h, y)
			err := y[0] - Y[k]
			gin := make([]float64, 4)
			l2.backward([]float64{err}, gin, lr)
			l1.backward(gin, nil, lr)
		}
	}
	fmt.Printf("%.2f\n", float64(time.Since(start).Microseconds())/1000.0)
}
