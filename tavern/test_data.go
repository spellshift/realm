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

	// Create test sessions (with tags)
	var testSessions []*ent.Session
	for groupNum := 1; groupNum <= 15; groupNum++ {
		gTag := client.Tag.Create().
			SetKind(tag.KindGroup).
			SetName(fmt.Sprintf("Group-%d", groupNum)).
			SaveX(ctx)

		for _, svcTag := range svcTags {
			hostName := fmt.Sprintf("Group %d - %s", groupNum, svcTag.Name)
			hostID := newRandomIdentifier()

			testSessions = append(testSessions,
				client.Session.Create().
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetHostname(hostName).
					SetIdentifier(newRandomIdentifier()).
					SetHostIdentifier(hostID).
					SetAgentIdentifier("test-data").
					AddTags(svcTag, gTag).
					SaveX(ctx),
			)

			testSessions = append(testSessions,
				client.Session.Create().
					SetLastSeenAt(time.Now().Add(-10*time.Minute)).
					SetHostname(hostName).
					SetIdentifier(newRandomIdentifier()).
					SetHostIdentifier(hostID).
					SetAgentIdentifier("test-data").
					AddTags(svcTag, gTag).
					SaveX(ctx),
			)

			testSessions = append(testSessions,
				client.Session.Create().
					SetLastSeenAt(time.Now().Add(-1*time.Hour)).
					SetHostname(hostName).
					SetIdentifier(newRandomIdentifier()).
					SetHostIdentifier(hostID).
					SetAgentIdentifier("test-data").
					AddTags(svcTag, gTag).
					SaveX(ctx),
			)
		}
	}

	/*
	 * Example Job: Hello World
	 */
	printMsgTome := client.Tome.Create().
		SetName("PrintMessage").
		SetDescription("Print a message for fun!").
		SetEldritch(`print(input_params['msg'])`).
		SetParamDefs(`{"msg":"string"}`).
		SaveX(ctx)

	printJob := client.Job.Create().
		SetName("HelloWorld").
		SetParameters(`{"msg":"Hello World!"}`).
		SetTome(printMsgTome).
		SaveX(ctx)

	// Queued
	client.Task.Create().
		SetSession(testSessions[0]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetJob(printJob).
		SaveX(ctx)

	// Claimed
	client.Task.Create().
		SetSession(testSessions[1]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetJob(printJob).
		SaveX(ctx)

	// Completed
	client.Task.Create().
		SetSession(testSessions[2]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetExecFinishedAt(timeAgo(5 * time.Second)).
		SetOutput("Hello World!").
		SetJob(printJob).
		SaveX(ctx)

	// Mid-Execution
	client.Task.Create().
		SetSession(testSessions[3]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetOutput("Hello").
		SetJob(printJob).
		SaveX(ctx)

	// Failed
	client.Task.Create().
		SetSession(testSessions[4]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetExecFinishedAt(timeAgo(5 * time.Second)).
		SetError("oops! Agent is OOM, can't print anything!").
		SetJob(printJob).
		SaveX(ctx)

	// Stale
	client.Task.Create().
		SetSession(testSessions[5]).
		SetCreatedAt(timeAgo(1 * time.Hour)).
		SetJob(printJob).
		SaveX(ctx)
}

func newRandomIdentifier() string {
	buf := make([]byte, 64)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate random identifier: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}

// timeAgo returns the current time minus the provided duration (e.g. 5 seconds ago)
func timeAgo(duration time.Duration) time.Time {
	return time.Now().Add(-1 * duration)
}
