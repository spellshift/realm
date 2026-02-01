package mux

import (
	"context"
	"fmt"
	"log/slog"

	gcppubsub "cloud.google.com/go/pubsub"
	"gocloud.dev/pubsub"
	"realm.pub/tavern/portals/portalpb"
)

// OpenPortal opens an existing portal for viewing (Client side).
func (m *Mux) OpenPortal(ctx context.Context, portalID int) (func(), error) {
	topicOut := m.TopicOut(portalID)
	subName := m.SubName(topicOut)

	m.subMgr.Lock()
	// Check Cache
	if m.useInMem {
		if _, ok := m.subMgr.active[subName]; ok {
			m.subMgr.refs[subName]++
			m.subMgr.Unlock()
			return m.makeTeardown(subName), nil
		}
	} else {
		if _, ok := m.subMgr.activeGCP[subName]; ok {
			m.subMgr.refs[subName]++
			m.subMgr.Unlock()
			return m.makeTeardown(subName), nil
		}
	}
	m.subMgr.Unlock()

	// Provisioning
	// Ensure subscription exists for the OUT topic
	if err := m.ensureSub(ctx, topicOut, subName); err != nil {
		return nil, fmt.Errorf("failed to ensure subscription: %w", err)
	}

	var subInMem *pubsub.Subscription
	var subGCP *gcppubsub.Subscription

	if m.useInMem {
		// Connect InMem
		subURL := m.SubURL(topicOut, subName)
		var err error
		subInMem, err = pubsub.OpenSubscription(ctx, subURL)
		if err != nil {
			return nil, fmt.Errorf("failed to open subscription %s: %w", subURL, err)
		}
	} else {
		// Connect GCP
		if m.gcpClient == nil {
			return nil, fmt.Errorf("gcp client is nil")
		}
		subGCP = m.gcpClient.Subscription(subName)
		// Configure ReceiveSettings for low latency
		subGCP.ReceiveSettings.Synchronous = false
		subGCP.ReceiveSettings.MaxOutstandingMessages = -1
		subGCP.ReceiveSettings.MaxOutstandingBytes = -1
	}

	m.subMgr.Lock()
	// RACE CONDITION CHECK:
	// Re-check cache in case another goroutine created it while we were provisioning/connecting
	if m.useInMem {
		if _, ok := m.subMgr.active[subName]; ok {
			m.subMgr.refs[subName]++
			m.subMgr.Unlock()
			subInMem.Shutdown(context.Background())
			return m.makeTeardown(subName), nil
		}
	} else {
		if _, ok := m.subMgr.activeGCP[subName]; ok {
			m.subMgr.refs[subName]++
			m.subMgr.Unlock()
			// GCP Subscription object is just a handle, no need to "close" it if we didn't start receive.
			return m.makeTeardown(subName), nil
		}
	}

	// We won the race (or are the first).
	if m.useInMem {
		m.subMgr.active[subName] = subInMem
	} else {
		m.subMgr.activeGCP[subName] = subGCP
	}
	m.subMgr.refs[subName] = 1

	// Prepare Loop Context
	ctxLoop, cancelLoop := context.WithCancel(context.Background())
	m.subMgr.cancelFuncs[subName] = cancelLoop

	m.subMgr.Unlock()

	// Spawn
	if m.useInMem {
		go func() {
			m.receiveLoop(ctxLoop, topicOut, subInMem)
		}()
	} else {
		go func() {
			err := subGCP.Receive(ctxLoop, func(ctx context.Context, msg *gcppubsub.Message) {
				m.dispatchGCPMsg(ctx, topicOut, msg)
			})
			if err != nil {
				// Receive returns error if context is canceled, but we should log distinct errors
				if ctxLoop.Err() == nil {
					slog.Error("GCP Receive loop failed", "sub", subName, "error", err)
					// Inject CLOSE mote to terminate gRPC streams
					closeMote := &portalpb.Mote{
						Payload: &portalpb.Mote_Bytes{
							Bytes: &portalpb.BytesPayload{
								Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
								Data: []byte(fmt.Sprintf("subscription error: %v", err)),
							},
						},
					}
					m.dispatch(topicOut, closeMote)
				}
			}
		}()
	}

	return m.makeTeardown(subName), nil
}

func (m *Mux) makeTeardown(subName string) func() {
	return func() {
		m.subMgr.Lock()
		m.subMgr.refs[subName]--
		shouldShutdown := false
		var sInMem *pubsub.Subscription
		var cancel context.CancelFunc

		if m.subMgr.refs[subName] <= 0 {
			if m.useInMem {
				if sub, ok := m.subMgr.active[subName]; ok {
					sInMem = sub
					delete(m.subMgr.active, subName)
					shouldShutdown = true
				}
			} else {
				if _, ok := m.subMgr.activeGCP[subName]; ok {
					// GCP sub object is just a handle
					delete(m.subMgr.activeGCP, subName)
					shouldShutdown = true
				}
			}

			if shouldShutdown {
				cancel = m.subMgr.cancelFuncs[subName]
				delete(m.subMgr.refs, subName)
				delete(m.subMgr.cancelFuncs, subName)
			}
		}
		m.subMgr.Unlock()

		if shouldShutdown {
			if cancel != nil {
				cancel()
			}
			if sInMem != nil {
				sInMem.Shutdown(context.Background())
			}
		}
	}
}
