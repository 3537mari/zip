package main

import (
	"fmt"
	"strings"
	"time"
)

func parseGet(req string) int {
	if !strings.HasPrefix(req, "GET ") {
		return -1
	}
	p := 4
	if p >= len(req) || req[p] != '/' {
		return -1
	}
	for p < len(req) && req[p] != ' ' {
		p++
	}
	return p - 4
}

func main() {
	req := "GET /api/v1/item/42 HTTP/1.1\r\nHost: localhost\r\n\r\n"
	n := 5000000
	start := time.Now()
	sum := 0
	for i := 0; i < n; i++ {
		sum += parseGet(req)
	}
	elapsed := time.Since(start)
	ms := float64(elapsed.Microseconds()) / 1000.0
	if ms < 0.01 {
		ms = 0.01
	}
	fmt.Println(sum)
	fmt.Printf("%.2f\n", ms)
}
