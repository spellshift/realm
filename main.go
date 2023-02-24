package main

import (
	"fmt"

	"github.com/kcarretto/realm/tavern/namegen"
)

func main() {
	for i := 0; i < 100; i++ {
		fmt.Println(namegen.GetRandomName())
	}
}
