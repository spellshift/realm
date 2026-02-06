package main

import (
	"fmt"
	"google.golang.org/grpc/mem"
)

func main() {
    pool := mem.DefaultBufferPool()
    if pool == nil {
        fmt.Println("No default pool")
        return
    }
    buf := pool.Get(1024)
    fmt.Printf("Buffer type: %T\n", buf)

    // Check if we can write to it
    // buf has ReadOnlyData() []byte
    data := buf.ReadOnlyData()
    // Is it mutable?
    data[0] = 'a'
    fmt.Printf("Data[0]: %c\n", data[0])

    // Check if Free works (no output expected, just compilation)
    buf.Free()

    // Key check
    key := [32]byte{}
    fmt.Printf("Key: %v\n", key)
}
