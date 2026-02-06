package main

import (
	"fmt"
	"reflect"
	"google.golang.org/grpc/mem"
)

func main() {
    pool := mem.DefaultBufferPool()
    fmt.Printf("Pool type: %T\n", pool)

    // We can't call methods if we don't know the interface, but we can reflect on the return value of Get
    method, ok := reflect.TypeOf(pool).MethodByName("Get")
    if ok {
        fmt.Printf("Get returns: %v\n", method.Type.Out(0))
    }
}
