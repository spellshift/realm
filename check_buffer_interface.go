package main

import (
	"fmt"
	"google.golang.org/grpc/mem"
)

func main() {
    var b mem.Buffer
    fmt.Printf("mem.Buffer interface exists: %T\n", b)
}
