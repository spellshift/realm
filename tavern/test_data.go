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

	// Create test beacons (with tags)
	var testBeacons []*ent.Beacon
	for groupNum := 1; groupNum <= 15; groupNum++ {
		gTag := client.Tag.Create().
			SetKind(tag.KindGroup).
			SetName(fmt.Sprintf("Group-%d", groupNum)).
			SaveX(ctx)

		for _, svcTag := range svcTags {
			hostName := fmt.Sprintf("Group %d - %s", groupNum, svcTag.Name)
			hostID := newRandomIdentifier()

			testBeacons = append(testBeacons,
				client.Beacon.Create().
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetHostname(hostName).
					SetIdentifier(newRandomIdentifier()).
					SetHostIdentifier(hostID).
					SetAgentIdentifier("test-data").
					AddTags(svcTag, gTag).
					SaveX(ctx),
			)

			testBeacons = append(testBeacons,
				client.Beacon.Create().
					SetLastSeenAt(time.Now().Add(-10*time.Minute)).
					SetHostname(hostName).
					SetIdentifier(newRandomIdentifier()).
					SetHostIdentifier(hostID).
					SetAgentIdentifier("test-data").
					AddTags(svcTag, gTag).
					SaveX(ctx),
			)

			testBeacons = append(testBeacons,
				client.Beacon.Create().
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
	 * Tome ParamDef Format Example
	 */
	client.Tome.Create().
		SetName("ParamDefFormatExample").
		SetDescription("This tome is an example that takes parameters and has parameter definitions defined (e.g. ParamDefs)").
		SetEldritch(``).
		SetParamDefs(`[
  {
    "name": "service_group",
    "label": "Service Group",
    "type": "string",
    "placeholder": "group 1"
  },
  {
    "name": "service_description",
    "label": "Service Description",
    "type": "string",
    "placeholder": "placeholder description for group 1"
  }
]`).
		SaveX(ctx)

	/*
	 * Example Quest: Hello World
	 */
	printMsgTome := client.Tome.Create().
		SetName("PrintMessage").
		SetDescription("Print a message for fun!").
		SetEldritch(`print(input_params['msg'])`).
		SetParamDefs(`[{"name":"msg","label":"Message","type":"string","placeholder":"something to print"}]`).
		SaveX(ctx)

	printQuest := client.Quest.Create().
		SetName("HelloWorld").
		SetParameters(`{"msg":"Hello World!"}`).
		SetTome(printMsgTome).
		SaveX(ctx)

	// Queued
	client.Task.Create().
		SetBeacon(testBeacons[0]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetQuest(printQuest).
		SaveX(ctx)

	// Claimed
	client.Task.Create().
		SetBeacon(testBeacons[1]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetQuest(printQuest).
		SaveX(ctx)

	// Completed
	client.Task.Create().
		SetBeacon(testBeacons[2]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetExecFinishedAt(timeAgo(5 * time.Second)).
		SetOutput("Hello World!").
		SetQuest(printQuest).
		SaveX(ctx)

	// Mid-Execution
	client.Task.Create().
		SetBeacon(testBeacons[3]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetOutput("Hello").
		SetQuest(printQuest).
		SaveX(ctx)

	// Failed
	client.Task.Create().
		SetBeacon(testBeacons[4]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetExecFinishedAt(timeAgo(5 * time.Second)).
		SetError("oops! Agent is OOM, can't print anything!").
		SetQuest(printQuest).
		SaveX(ctx)

	// Stale
	client.Task.Create().
		SetBeacon(testBeacons[5]).
		SetCreatedAt(timeAgo(1 * time.Hour)).
		SetQuest(printQuest).
		SaveX(ctx)

	// Lorem Ipsum
	client.Task.Create().
		SetBeacon(testBeacons[6]).
		SetCreatedAt(timeAgo(5 * time.Minute)).
		SetClaimedAt(timeAgo(1 * time.Minute)).
		SetExecStartedAt(timeAgo(5 * time.Second)).
		SetExecFinishedAt(timeAgo(5 * time.Second)).
		SetOutput(loremIpsum).
		SetQuest(printQuest).
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

const loremIpsum = `
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Eu tincidunt tortor aliquam nulla facilisi cras fermentum odio eu. In fermentum posuere urna nec tincidunt praesent. Elementum nisi quis eleifend quam adipiscing. Eu sem integer vitae justo. Congue quisque egestas diam in arcu cursus euismod. Posuere urna nec tincidunt praesent semper feugiat nibh sed pulvinar. Iaculis urna id volutpat lacus laoreet. Morbi enim nunc faucibus a pellentesque sit amet porttitor. Quisque id diam vel quam elementum. Nulla malesuada pellentesque elit eget gravida cum. Volutpat odio facilisis mauris sit amet massa. Neque egestas congue quisque egestas diam. Risus feugiat in ante metus. Sem et tortor consequat id porta nibh. Congue eu consequat ac felis. Nibh sit amet commodo nulla facilisi nullam vehicula ipsum a. Eget felis eget nunc lobortis. Faucibus a pellentesque sit amet porttitor eget dolor. Morbi enim nunc faucibus a pellentesque sit.

Quam nulla porttitor massa id. A arcu cursus vitae congue mauris rhoncus aenean vel elit. Amet nisl suscipit adipiscing bibendum est ultricies. Lacus luctus accumsan tortor posuere ac ut consequat. Sodales ut eu sem integer vitae justo eget magna. Odio morbi quis commodo odio aenean sed adipiscing diam. Pellentesque eu tincidunt tortor aliquam nulla facilisi cras. Diam vel quam elementum pulvinar etiam non quam lacus suspendisse. Massa id neque aliquam vestibulum morbi blandit. Justo eget magna fermentum iaculis eu non diam phasellus vestibulum. Ultricies leo integer malesuada nunc vel risus commodo viverra. Habitant morbi tristique senectus et netus et malesuada. Morbi tincidunt augue interdum velit. Convallis posuere morbi leo urna molestie. Sit amet risus nullam eget. Sit amet facilisis magna etiam tempor orci eu lobortis.

Sem viverra aliquet eget sit amet tellus cras. Mauris sit amet massa vitae tortor condimentum lacinia. Neque aliquam vestibulum morbi blandit. Sed lectus vestibulum mattis ullamcorper velit sed ullamcorper. Cras sed felis eget velit aliquet sagittis id. Tortor pretium viverra suspendisse potenti nullam ac. In ante metus dictum at tempor. Egestas integer eget aliquet nibh praesent. Sollicitudin nibh sit amet commodo nulla facilisi nullam vehicula ipsum. Morbi blandit cursus risus at. Et tortor consequat id porta nibh venenatis. Augue ut lectus arcu bibendum. Ornare arcu odio ut sem nulla pharetra diam. Eu consequat ac felis donec et odio pellentesque diam volutpat. Amet est placerat in egestas erat imperdiet sed euismod nisi. Fermentum posuere urna nec tincidunt praesent. Adipiscing elit pellentesque habitant morbi tristique senectus et. Ut eu sem integer vitae.

Vivamus arcu felis bibendum ut tristique et egestas quis ipsum. Nisi quis eleifend quam adipiscing vitae proin. Lobortis scelerisque fermentum dui faucibus in ornare. Orci eu lobortis elementum nibh tellus molestie nunc. Ac feugiat sed lectus vestibulum mattis ullamcorper velit sed ullamcorper. Sodales ut eu sem integer vitae justo eget magna. Habitasse platea dictumst vestibulum rhoncus est pellentesque. Massa id neque aliquam vestibulum morbi blandit. Aliquet risus feugiat in ante metus dictum at tempor commodo. In ante metus dictum at tempor commodo ullamcorper. In ornare quam viverra orci. Lorem ipsum dolor sit amet consectetur adipiscing elit duis. Nec dui nunc mattis enim. Ornare aenean euismod elementum nisi quis eleifend. Justo donec enim diam vulputate ut pharetra sit amet aliquam. Tempor id eu nisl nunc mi ipsum faucibus. Ipsum dolor sit amet consectetur adipiscing elit.

Blandit volutpat maecenas volutpat blandit. Donec ultrices tincidunt arcu non sodales. Phasellus egestas tellus rutrum tellus pellentesque eu. Fringilla ut morbi tincidunt augue interdum velit euismod in. Arcu odio ut sem nulla. Luctus accumsan tortor posuere ac ut consequat. Et malesuada fames ac turpis egestas integer. Volutpat consequat mauris nunc congue nisi vitae suscipit tellus. Porttitor leo a diam sollicitudin tempor id eu nisl nunc. Vitae proin sagittis nisl rhoncus mattis rhoncus urna neque viverra. Dui sapien eget mi proin sed libero. Quisque id diam vel quam elementum pulvinar. Massa id neque aliquam vestibulum morbi blandit. Tincidunt lobortis feugiat vivamus at augue eget arcu dictum varius.

Ullamcorper velit sed ullamcorper morbi tincidunt ornare massa eget. Turpis massa sed elementum tempus egestas sed sed. Commodo odio aenean sed adipiscing. Nunc sed augue lacus viverra vitae. Diam quam nulla porttitor massa id neque aliquam vestibulum. Elit sed vulputate mi sit amet mauris commodo quis. Morbi blandit cursus risus at ultrices mi tempus. Ut placerat orci nulla pellentesque dignissim enim sit amet venenatis. In egestas erat imperdiet sed euismod. Non enim praesent elementum facilisis.

Nec ultrices dui sapien eget mi proin sed libero. Ut faucibus pulvinar elementum integer enim neque volutpat ac. Amet luctus venenatis lectus magna fringilla urna porttitor. Donec adipiscing tristique risus nec. Pellentesque eu tincidunt tortor aliquam nulla facilisi cras. Dictum non consectetur a erat nam at. Erat imperdiet sed euismod nisi porta lorem mollis aliquam. Nisl rhoncus mattis rhoncus urna neque viverra justo nec. Nam libero justo laoreet sit amet. Sed pulvinar proin gravida hendrerit. Vel pretium lectus quam id. Molestie a iaculis at erat. Neque gravida in fermentum et sollicitudin ac orci. Turpis tincidunt id aliquet risus feugiat in. Est pellentesque elit ullamcorper dignissim cras tincidunt lobortis feugiat vivamus. Eget sit amet tellus cras adipiscing enim. Varius morbi enim nunc faucibus a pellentesque sit amet. Nunc sed id semper risus in hendrerit gravida.

Amet consectetur adipiscing elit ut aliquam purus sit. Gravida in fermentum et sollicitudin ac orci phasellus. Porttitor lacus luctus accumsan tortor posuere ac ut consequat semper. Odio morbi quis commodo odio. Purus sit amet volutpat consequat mauris nunc congue nisi. Tempus quam pellentesque nec nam aliquam sem et tortor consequat. A diam maecenas sed enim ut sem viverra aliquet. Nec feugiat nisl pretium fusce id. Id neque aliquam vestibulum morbi blandit cursus. Tincidunt tortor aliquam nulla facilisi cras fermentum. Eget velit aliquet sagittis id consectetur purus. Nunc faucibus a pellentesque sit amet porttitor eget dolor morbi.

Libero volutpat sed cras ornare arcu dui vivamus arcu. Non enim praesent elementum facilisis leo. Morbi tristique senectus et netus et malesuada fames ac turpis. Adipiscing commodo elit at imperdiet dui accumsan sit. Sociis natoque penatibus et magnis dis parturient montes nascetur ridiculus. Euismod quis viverra nibh cras pulvinar mattis. Nunc congue nisi vitae suscipit tellus. Morbi quis commodo odio aenean sed adipiscing. Sit amet porttitor eget dolor morbi non arcu. Integer eget aliquet nibh praesent tristique magna. Et tortor consequat id porta. Non pulvinar neque laoreet suspendisse. Nec tincidunt praesent semper feugiat. Enim diam vulputate ut pharetra sit amet aliquam. Quis lectus nulla at volutpat diam ut venenatis tellus. Et netus et malesuada fames ac turpis egestas maecenas. Vitae ultricies leo integer malesuada nunc. Habitant morbi tristique senectus et netus et malesuada fames ac.

Ut tristique et egestas quis. Viverra nibh cras pulvinar mattis nunc sed blandit libero. Urna duis convallis convallis tellus id interdum velit laoreet. Lectus magna fringilla urna porttitor rhoncus dolor purus. Elit sed vulputate mi sit amet mauris. Semper eget duis at tellus at urna condimentum. Dolor sit amet consectetur adipiscing. Ut tellus elementum sagittis vitae et leo duis. Nibh tellus molestie nunc non blandit massa enim. Dictum sit amet justo donec enim diam vulputate. In fermentum posuere urna nec. Placerat duis ultricies lacus sed turpis tincidunt id aliquet. Est lorem ipsum dolor sit amet consectetur. Sed enim ut sem viverra aliquet eget sit amet tellus. Scelerisque in dictum non consectetur a erat nam. Sit amet commodo nulla facilisi nullam vehicula. Commodo odio aenean sed adipiscing. Pulvinar mattis nunc sed blandit libero volutpat sed. Luctus accumsan tortor posuere ac ut consequat semper viverra. Arcu bibendum at varius vel pharetra vel turpis nunc.
`
