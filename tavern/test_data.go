package main

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"log"
	"time"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/tag"
)

// createTestData populates the DB with some test data :)
func createTestData(ctx context.Context, client *ent.Client) {
	log.Printf("[WARN] Test data is enabled")
	svcTags := make([]*ent.Tag, 0, 20)
	for i := 0; i < 20; i++ {
		svcTags = append(
			svcTags,
			client.Tag.Create().
				SetKind(tag.KindService).
				SetName(fmt.Sprintf("Service-%d", i+1)).
				SaveX(ctx),
		)
	}

	for groupNum := 1; groupNum <= 15; groupNum++ {
		gTag := client.Tag.Create().
			SetKind(tag.KindGroup).
			SetName(fmt.Sprintf("Group-%d", groupNum)).
			SaveX(ctx)

		for _, svcTag := range svcTags {
			hostName := fmt.Sprintf("Group %d - %s", groupNum, svcTag.Name)
			hostID := newRandomIdentifier()

			client.Session.Create().
				SetLastSeenAt(time.Now().Add(-1*time.Minute)).
				SetHostname(hostName).
				SetIdentifier(newRandomIdentifier()).
				SetHostIdentifier(hostID).
				SetAgentIdentifier("test-data").
				AddTags(svcTag, gTag).
				SaveX(ctx)

			client.Session.Create().
				SetLastSeenAt(time.Now().Add(-10*time.Minute)).
				SetHostname(hostName).
				SetIdentifier(newRandomIdentifier()).
				SetHostIdentifier(hostID).
				SetAgentIdentifier("test-data").
				AddTags(svcTag, gTag).
				SaveX(ctx)

			client.Session.Create().
				SetLastSeenAt(time.Now().Add(-1*time.Hour)).
				SetHostname(hostName).
				SetIdentifier(newRandomIdentifier()).
				SetHostIdentifier(hostID).
				SetAgentIdentifier("test-data").
				AddTags(svcTag, gTag).
				SaveX(ctx)
		}
	}
}

func newRandomIdentifier() string {
	buf := make([]byte, 64)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate random identifier: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}
