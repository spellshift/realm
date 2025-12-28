package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"strconv"
	"time"
)

func main() {
	if len(os.Args) < 2 {
		log.Fatal("Usage: traffic_test <socks_addr>")
	}
	socksAddr := os.Args[1]

	// Start Echo Servers
	go startTCPEchoServer(":9001")
	go startUDPEchoServer(":9002")

	// Wait for servers to start
	time.Sleep(1 * time.Second)

	// Test TCP
	if err := testTCP(socksAddr, "127.0.0.1:9001"); err != nil {
		log.Fatalf("TCP Test Failed: %v", err)
	}
	fmt.Println("TCP Test Passed")

	// Test UDP
	if err := testUDP(socksAddr, "127.0.0.1:9002"); err != nil {
		log.Fatalf("UDP Test Failed: %v", err)
	}
	fmt.Println("UDP Test Passed")
}

func startTCPEchoServer(addr string) {
	l, err := net.Listen("tcp", addr)
	if err != nil {
		log.Fatal(err)
	}
	for {
		c, err := l.Accept()
		if err != nil {
			return
		}
		go io.Copy(c, c)
	}
}

func startUDPEchoServer(addr string) {
	c, err := net.ListenPacket("udp", addr)
	if err != nil {
		log.Fatal(err)
	}
	buf := make([]byte, 1024)
	for {
		n, client, err := c.ReadFrom(buf)
		if err != nil {
			continue
		}
		c.WriteTo(buf[:n], client)
	}
}

func testTCP(socksAddr, target string) error {
    // Basic SOCKS5 TCP client
    conn, err := net.Dial("tcp", socksAddr)
    if err != nil {
        return err
    }
    defer conn.Close()

    // 1. Handshake
    conn.Write([]byte{0x05, 0x01, 0x00})
    buf := make([]byte, 2)
    io.ReadFull(conn, buf)
    if buf[0] != 0x05 || buf[1] != 0x00 {
        return fmt.Errorf("socks handshake failed")
    }

    // 2. Request (CONNECT)
    // Target is 127.0.0.1:9001
    targetHost, targetPortStr, _ := net.SplitHostPort(target)
    targetPort, _ := strconv.Atoi(targetPortStr)
    targetIP := net.ParseIP(targetHost).To4()

    req := []byte{0x05, 0x01, 0x00, 0x01}
    req = append(req, targetIP...)
    req = append(req, byte(targetPort>>8), byte(targetPort))
    conn.Write(req)

    // 3. Read Reply
    respBuf := make([]byte, 10)
    io.ReadFull(conn, respBuf)
    if respBuf[1] != 0x00 {
        return fmt.Errorf("socks connect failed: %x", respBuf[1])
    }

    // 4. Send/Recv
    msg := "hello tcp"
    conn.Write([]byte(msg))
    readBuf := make([]byte, len(msg))
    if _, err := io.ReadFull(conn, readBuf); err != nil {
        return err
    }
    if string(readBuf) != msg {
        return fmt.Errorf("expected %s, got %s", msg, string(readBuf))
    }
    return nil
}

func testUDP(socksAddr, target string) error {
    // 1. Connect TCP to SOCKS server
    conn, err := net.Dial("tcp", socksAddr)
    if err != nil {
        return err
    }
    defer conn.Close()

    // 2. Handshake
    conn.Write([]byte{0x05, 0x01, 0x00}) // VER 5, 1 Method, NO AUTH
    buf := make([]byte, 2)
    io.ReadFull(conn, buf)
    if buf[0] != 0x05 || buf[1] != 0x00 {
        return fmt.Errorf("socks handshake failed")
    }

    // 3. UDP ASSOCIATE
    conn.Write([]byte{0x05, 0x03, 0x00, 0x01, 0,0,0,0, 0,0})

    // 4. Read Reply
    header := make([]byte, 10) // Min size for IPv4
    io.ReadFull(conn, header)
    if header[1] != 0x00 {
        return fmt.Errorf("UDP ASSOCIATE failed: %x", header[1])
    }

    // Parse bind addr/port
    var relayIP net.IP
    switch header[3] {
    case 0x01: // IPv4
        relayIP = net.IP(header[4:8])
    default:
        // We only support IPv4 for this test
        return fmt.Errorf("unsupported bind address type: %d", header[3])
    }
    relayPort := int(header[8])<<8 | int(header[9])

    // If relayIP is 0.0.0.0, use the address we connected to
    if relayIP.IsUnspecified() {
        tcpRemoteAddr := conn.RemoteAddr().(*net.TCPAddr)
        relayIP = tcpRemoteAddr.IP
    }

    relayAddr := &net.UDPAddr{IP: relayIP, Port: relayPort}

    // 5. Send UDP Packet
    udpConn, err := net.ListenUDP("udp", nil)
    if err != nil {
        return err
    }
    defer udpConn.Close()

    // Header: RSV(2) FRAG(1) ATYP(1) DST.ADDR DST.PORT DATA
    // Target is 127.0.0.1:9002
    targetHost, targetPortStr, _ := net.SplitHostPort(target)
    targetPort, _ := strconv.Atoi(targetPortStr)
    targetIP := net.ParseIP(targetHost).To4()

    req := []byte{0x00, 0x00, 0x00, 0x01}
    req = append(req, targetIP...)
    req = append(req, byte(targetPort>>8), byte(targetPort))
    msg := "hello udp"
    req = append(req, []byte(msg)...)

    udpConn.WriteTo(req, relayAddr)

    // 6. Read Reply
    udpConn.SetReadDeadline(time.Now().Add(5 * time.Second))
    respBuf := make([]byte, 1024)
    n, _, err := udpConn.ReadFrom(respBuf)
    if err != nil {
        return err
    }

    // Unwrap header
    // Skip 10 bytes (IPv4 header)
    if n < 10 {
        return fmt.Errorf("response too short")
    }
    data := respBuf[10:n]
    if string(data) != msg {
        return fmt.Errorf("expected %s, got %s", msg, string(data))
    }

    return nil
}
