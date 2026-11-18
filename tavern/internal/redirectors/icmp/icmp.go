package icmp

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"os"
	"runtime"
	"strings"
	"time"

	golicmp "golang.org/x/net/icmp"
	"golang.org/x/net/ipv4"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/conversation"
	"realm.pub/tavern/internal/c2/convpb"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/redirectors"
)

var log = logging.For("redirectors.icmp")

const (
	ICMPChunkSize    = 1400
	MaxConversations = 200_000
	ConvTTL          = 5 * time.Minute
	ProtocolICMP     = 1
)

func init() {
	redirectors.Register("icmp", &Redirector{
		manager: conversation.NewManager(MaxConversations, ConvTTL),
	})
}

type Redirector struct {
	manager *conversation.Manager
}

func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, _ *tls.Config) error {
	if runtime.GOOS == "windows" {
		return fmt.Errorf("icmp redirector is not supported on windows")
	}

	// The Linux kernel auto-responds to ICMP echo requests with a standard echo reply
	// (mirroring the request payload) before user-space can act. This causes agents to
	// receive their own request payload instead of our STATUS reply.
	// Requires: echo 1 > /proc/sys/net/ipv4/icmp_echo_ignore_all on the redirector host.
	val, err := os.ReadFile("/proc/sys/net/ipv4/icmp_echo_ignore_all")
	if err != nil {
		return fmt.Errorf("icmp redirector: failed to check icmp_echo_ignore_all: %w", err)
	}
	if strings.TrimSpace(string(val)) != "1" {
		return fmt.Errorf("icmp redirector: kernel ICMP echo replies are enabled — run 'echo 1 | sudo tee /proc/sys/net/ipv4/icmp_echo_ignore_all' on this host before starting the redirector")
	}

	// listenOn is an IP address (no port for raw ICMP), e.g. "0.0.0.0"
	conn, err := golicmp.ListenPacket("ip4:icmp", listenOn)
	if err != nil {
		return err
	}
	defer conn.Close()

	log.InfoContext(ctx, "started ICMP redirector", "listen_on", listenOn)

	buf := make([]byte, 65536)
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		conn.SetReadDeadline(time.Now().Add(time.Second))
		n, src, err := conn.ReadFrom(buf)
		if err != nil {
			if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
				continue
			}
			log.ErrorContext(ctx, "read error", "error", err)
			continue
		}

		msg, err := golicmp.ParseMessage(ProtocolICMP, buf[:n])
		if err != nil || msg.Type != ipv4.ICMPTypeEcho {
			continue
		}
		echo, ok := msg.Body.(*golicmp.Echo)
		if !ok {
			continue
		}
		if !redirectors.Allowed(ctx) {
			continue
		}

		go r.handle(ctx, conn, src, echo, upstream)
	}
}

func (r *Redirector) handle(ctx context.Context, conn *golicmp.PacketConn, src net.Addr, echo *golicmp.Echo, upstream *grpc.ClientConn) {
	ctx, _, _ = logging.EnsureTrace(ctx)

	var pkt convpb.ConvPacket
	if err := proto.Unmarshal(echo.Data, &pkt); err != nil {
		log.TraceContext(ctx, "non-C2 ping, echoing reply", "src", src.String())
		reply, err := (&golicmp.Message{
			Type: ipv4.ICMPTypeEchoReply,
			Code: 0,
			Body: &golicmp.Echo{ID: echo.ID, Seq: echo.Seq, Data: echo.Data},
		}).Marshal(nil)
		if err != nil {
			log.ErrorContext(ctx, "failed to marshal ping reply", "error", err)
			return
		}
		if _, err := conn.WriteTo(reply, src); err != nil {
			log.ErrorContext(ctx, "failed to write ping reply", "error", err)
		}
		return
	}

	var responseData []byte
	var err error
	switch pkt.Type {
	case convpb.PacketType_PACKET_TYPE_INIT:
		responseData, err = r.manager.HandleInit(&pkt)
	case convpb.PacketType_PACKET_TYPE_DATA:
		responseData, err = r.manager.HandleData(ctx, upstream, &pkt, ICMPChunkSize, src.String())
	case convpb.PacketType_PACKET_TYPE_FETCH:
		responseData, err = r.manager.HandleFetch(&pkt)
	case convpb.PacketType_PACKET_TYPE_COMPLETE:
		responseData, err = r.manager.HandleComplete(&pkt)
	default:
		log.TraceContext(ctx, "unknown packet type", "type", pkt.Type)
		return
	}
	if err != nil {
		log.ErrorContext(ctx, "handle error", "type", pkt.Type, "conv_id", pkt.ConversationId, "error", err)
		return
	}

	// Log outgoing reply content at TRACE to aid wire debugging without unmarshaling on normal traffic.
	if log.Enabled(ctx, logging.LevelTrace) {
		var replyPkt convpb.ConvPacket
		if parseErr := proto.Unmarshal(responseData, &replyPkt); parseErr == nil {
			if replyPkt.Type == convpb.PacketType_PACKET_TYPE_STATUS {
				log.TraceContext(ctx, "sending STATUS reply",
					"conv_id", pkt.ConversationId,
					"req_type", pkt.Type,
					"ack_ranges", len(replyPkt.Acks),
					"nacks", replyPkt.Nacks,
					"response_size", len(responseData),
				)
			} else {
				log.TraceContext(ctx, "sending reply",
					"conv_id", pkt.ConversationId,
					"req_type", pkt.Type,
					"reply_type", replyPkt.Type,
					"response_size", len(responseData),
				)
			}
		} else {
			log.TraceContext(ctx, "sending raw reply",
				"conv_id", pkt.ConversationId,
				"req_type", pkt.Type,
				"response_size", len(responseData),
			)
		}
	}

	reply, err := (&golicmp.Message{
		Type: ipv4.ICMPTypeEchoReply,
		Code: 0,
		Body: &golicmp.Echo{ID: echo.ID, Seq: echo.Seq, Data: responseData},
	}).Marshal(nil)
	if err != nil {
		log.ErrorContext(ctx, "failed to marshal reply", "error", err)
		return
	}

	if n, err := conn.WriteTo(reply, src); err != nil {
		log.ErrorContext(ctx, "failed to write reply", "destination", src.String(), "error", err)
	} else {
		log.TraceContext(ctx, "sent reply", "conv_id", pkt.ConversationId, "dst", src.String(), "bytes", n)
	}
}
