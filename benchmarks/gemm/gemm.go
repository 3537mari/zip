package main

import (
	"fmt"
	"time"
)

func gemm(a, b, c []float64, m, k, n int) {
	for i := 0; i < m; i++ {
		for j := 0; j < n; j++ {
			sum := 0.0
			for l := 0; l < k; l++ {
				sum += a[i*k+l] * b[l*n+j]
			}
			c[i*n+j] = sum
		}
	}
}

func main() {
	size := 512
	a := make([]float64, size*size)
	b := make([]float64, size*size)
	c := make([]float64, size*size)
	for i := range a {
		a[i] = 1.0
		b[i] = 2.0
	}
	start := time.Now()
	gemm(a, b, c, size, size, size)
	elapsed := time.Since(start)
	fmt.Println(c[0])
	fmt.Printf("%.2f\n", float64(elapsed.Microseconds())/1000.0)
}
