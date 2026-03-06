package main

import (
	"fmt"
	"io/ioutil"
	"strings"
)

func main() {
	content, _ := ioutil.ReadFile("tavern/internal/portals/ws/ssh.go")
	s := string(content)

	s = strings.Replace(s, `	seqID          uint64
	expectedSeqOut uint64
	readBuffer     []byte
	recv           <-chan *portalpb.Mote
	cleanupSub     func()
}`, `	seqID          uint64
	seqMu          sync.Mutex
	expectedSeqOut uint64
	readBuffer     []byte
	recv           <-chan *portalpb.Mote
	cleanupSub     func()
}`, 1)

	ioutil.WriteFile("tavern/internal/portals/ws/ssh.go", []byte(s), 0644)
	fmt.Println("Patched seq races")
}
