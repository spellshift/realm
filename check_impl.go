package main

import (
	"fmt"
	"google.golang.org/grpc/mem"
)

func main() {
    b := []byte{1, 2}
    var buf mem.Buffer
    // try to assign *[]byte to mem.Buffer
    // buf = &b // This will fail compilation if not implemented

    // check SliceBuffer
    sb := mem.SliceBuffer(b)
    buf = sb
    fmt.Printf("SliceBuffer implements Buffer: %T\n", buf)
}
