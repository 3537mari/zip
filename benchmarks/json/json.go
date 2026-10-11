package main

import (
	"encoding/json"
	"fmt"
	"time"
)

func main() {
	items := make([]map[string]int, 10000)
	for i := 0; i < 10000; i++ {
		items[i] = map[string]int{"id": i}
	}
	data := map[string]any{"данные": items}
	raw, _ := json.Marshal(data)

	start := time.Now()
	var parsed map[string]any
	_ = json.Unmarshal(raw, &parsed)
	elapsed := time.Since(start)
	fmt.Printf("%.2f\n", float64(elapsed.Microseconds())/1000.0)
}
