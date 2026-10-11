package main

import (
	"fmt"
	"time"
)

func fibonacci(n uint64) uint64 {
	if n < 2 {
		return n
	}
	return fibonacci(n-1) + fibonacci(n-2)
}

func main() {
	n := uint64(40)
	start := time.Now()
	result := fibonacci(n)
	elapsed := time.Since(start)
	fmt.Println(result)
	fmt.Printf("%.2f\n", float64(elapsed.Microseconds())/1000.0)
}
