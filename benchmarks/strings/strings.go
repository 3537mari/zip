package main

import (
	"fmt"
	"time"
)

func main() {
	const N = 1000000
	start := time.Now()
	h := uint32(2166136261)
	buf := make([]byte, 0, 8192)
	for i := 0; i < N; i++ {
		tmp := fmt.Sprintf("%d,", i%1000)
		buf = append(buf, tmp...)
		for _, b := range []byte(tmp) {
			h ^= uint32(b)
			h *= 16777619
		}
		if len(buf) > 4096 {
			buf = buf[:0]
		}
	}
	elapsed := time.Since(start)
	fmt.Println(h)
	fmt.Printf("%.2f\n", float64(elapsed.Microseconds())/1000.0)
}
