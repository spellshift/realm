package main

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/binary"
	"fmt"
	"io"
	"log/slog"
	mrand "math/rand"
	"net"
	"time"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/namegen"
)

// createTestData populates the DB with some test data :)
func createTestData(ctx context.Context, client *ent.Client) {
	slog.WarnContext(ctx, "test data is enabled")

	client.User.Create().
		SetName("Admin").
		SetOauthID("AdminOAuthID").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)
	client.User.Create().
		SetName("Admin2").
		SetOauthID("Admin2OAuthID").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)
	client.User.Create().
		SetName("User").
		SetOauthID("UserOAuthID").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/a/ac/Default_pfp.jpg").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)

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

		for i, svcTag := range svcTags {
			hostName := fmt.Sprintf("Group %d - %s", groupNum, svcTag.Name)
			hostID := newRandomIdentifier()
			hostIP := newRandomIP()

			var testHost *ent.Host;
			if i == 4 && groupNum == 5 {
				testHost = client.Host.Create().
					SetName(hostName).
					SetIdentifier(hostID).
					SetPrimaryIP(hostIP).
					SetPlatform(c2pb.Host_Platform(i%len(c2pb.Host_Platform_value))).
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(60*time.Second)).
					AddTags(svcTag, gTag).
					SaveX(ctx)
			} else if i == 3 {
				testHost = client.Host.Create().
					SetName(hostName).
					SetIdentifier(hostID).
					SetPrimaryIP(hostIP).
					SetPlatform(c2pb.Host_Platform(i%len(c2pb.Host_Platform_value))).
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(600*time.Second)).
					AddTags(svcTag, gTag).
					SaveX(ctx)
			} else if groupNum == 1 {
				testHost = client.Host.Create().
					SetName(hostName).
					SetIdentifier(hostID).
					SetPrimaryIP(hostIP).
					SetPlatform(c2pb.Host_Platform(i%len(c2pb.Host_Platform_value))).
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(600*time.Second)).
					AddTags(svcTag, gTag).
					SaveX(ctx)
			} else {
				testHost = client.Host.Create().
					SetName(hostName).
					SetIdentifier(hostID).
					SetPrimaryIP(hostIP).
					SetPlatform(c2pb.Host_Platform(i%len(c2pb.Host_Platform_value))).
					SetLastSeenAt(time.Now().Add(-1*time.Minute)).
					SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(600000*time.Second)).
					AddTags(svcTag, gTag).
					SaveX(ctx)
			}

			client.HostCredential.Create().
				SetHost(testHost).
				SetPrincipal("root").
				SetKind(epb.Credential_KIND_PASSWORD).
				SetSecret(newRandomCredential()).
				SaveX(ctx)

			// Cycle through transports: HTTP1, GRPC, DNS
			getTransport := func(idx int) c2pb.Transport_Type {
				transports := []c2pb.Transport_Type{
					c2pb.Transport_TRANSPORT_HTTP1,
					c2pb.Transport_TRANSPORT_GRPC,
					c2pb.Transport_TRANSPORT_DNS,
				}
				return transports[idx%3]
			}

			if i == 4 && groupNum == 5 {
				// Host with dead beacons Group 5 - Service 5
				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-1*time.Minute)).
						SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(60*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(60).
						SetPrincipal("root").
						SetTransport(getTransport(groupNum*100 + i*10 + 0)).
						SaveX(ctx),
				)
				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-2*time.Minute)).
						SetNextSeenAt(time.Now().Add(-2*time.Minute).Add(60*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(60).
						SetPrincipal("root").
						SetTransport(getTransport(groupNum*100 + i*10 + 1)).
						SaveX(ctx),
				)
				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-1*time.Minute)).
						SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(60*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(60).
						SetPrincipal("root").
						SetTransport(getTransport(groupNum*100 + i*10 + 2)).
						SaveX(ctx),
				)
			} else {
				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-1*time.Minute)).
						SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(600000*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(600000).
						SetPrincipal("root").
						SetTransport(getTransport(groupNum*100 + i*10 + 0)).
						SaveX(ctx),
				)
				if i == 3 {
					testBeacons = append(testBeacons,
						client.Beacon.Create().
							SetLastSeenAt(time.Now().Add(-1*time.Minute)).
							SetNextSeenAt(time.Now().Add(-1*time.Minute).Add(600*time.Second)).
							SetIdentifier(newRandomIdentifier()).
							SetAgentIdentifier("test-data").
							SetHost(testHost).
							SetInterval(600).
							SetPrincipal("janet").
						SetTransport(getTransport(groupNum*100 + i*10 + 1)).
						SaveX(ctx),
					)
				}
				if groupNum == 1 {
					testBeacons = append(testBeacons,
						client.Beacon.Create().
							SetLastSeenAt(time.Now().Add(-1*time.Minute)).
							SetNextSeenAt(time.Now().Add(-30*time.Second).Add(600000*time.Second)).
							SetIdentifier(newRandomIdentifier()).
							SetAgentIdentifier("test-data").
							SetHost(testHost).
							SetInterval(600000).
							SetPrincipal("jane").
						SetTransport(getTransport(groupNum*100 + i*10 + 2)).
						SaveX(ctx),
					)
				}

				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-10*time.Minute)).
						SetNextSeenAt(time.Now().Add(-10*time.Second).Add(1000*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(1000).
						SetPrincipal("admin").
						SetTransport(getTransport(groupNum*100 + i*10 + 3)).
						SaveX(ctx),
				)

				testBeacons = append(testBeacons,
					client.Beacon.Create().
						SetLastSeenAt(time.Now().Add(-1*time.Hour)).
						SetNextSeenAt(time.Now().Add(-1*time.Hour).Add(4*time.Second)).
						SetIdentifier(newRandomIdentifier()).
						SetAgentIdentifier("test-data").
						SetHost(testHost).
						SetInterval(4).
						SetPrincipal("Administrator").
						SetTransport(getTransport(groupNum*100 + i*10 + 4)).
						SaveX(ctx),
				)
			}
		}
	}

	/*
	 * Tome ParamDef Format Example
	 */
	client.Tome.Create().
		SetName("ParamDefFormatExample").
		SetDescription("This tome is an example that takes parameters and has parameter definitions defined (e.g. ParamDefs)").
		SetAuthor("kcarretto").
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
		SetAuthor("kcarretto").
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

	for i := 0; i < 5; i++ {
		createQuest(ctx, client, testBeacons...)
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

func newRandomIP() string {
	buf := make([]byte, 4)
	ip := mrand.Uint32()
	binary.LittleEndian.PutUint32(buf, ip)
	return net.IP(buf).String()
}

func newRandomCredential() string {
	buf := make([]byte, 16)
	_, err := io.ReadFull(rand.Reader, buf)
	if err != nil {
		panic(fmt.Errorf("failed to generate random credential: %w", err))
	}
	return base64.StdEncoding.EncodeToString(buf)
}

// timeAgo returns the current time minus the provided duration (e.g. 5 seconds ago)
func timeAgo(duration time.Duration) time.Time {
	return time.Now().Add(-1 * duration)
}

const loremIpsum = `
100644	root	0	2996	2022-03-04 02:54:17 UTC	File	locale.alias
100644	root	0	887	2013-04-01 16:41:40 UTC	File	rpc
100644	root	0	5217	2022-03-17 19:03:00 UTC	File	manpath.config
40755	root	0	4096	2023-05-16 02:08:52 UTC	Dir	ubuntu-advantage
40700	root	0	4096	2023-10-05 00:15:43 UTC	Dir	multipath
120777	root	0	23	2023-05-16 02:08:36 UTC		vtrgb
40755	root	0	4096	2023-05-16 02:09:12 UTC	Dir	logcheck
120777	root	0	19	2023-05-16 02:08:21 UTC		mtab
100644	root	0	12813	2021-03-27 22:32:57 UTC	File	services
100644	root	0	552	2020-08-12 00:15:04 UTC	File	pam.conf
40755	root	0	4096	2023-10-05 20:42:48 UTC	Dir	alternatives
100644	root	0	13	2021-08-22 17:00:00 UTC	File	debian_version
40755	root	0	4096	2023-05-16 02:07:49 UTC	Dir	systemd
40755	root	0	4096	2023-05-16 02:09:28 UTC	Dir	sos
40755	root	0	4096	2023-05-16 02:09:04 UTC	Dir	X11
40755	root	0	4096	2023-05-16 02:09:24 UTC	Dir	byobu
40755	root	0	4096	2023-05-16 02:09:06 UTC	Dir	pm
100644	root	0	1948	2023-10-05 00:16:04 UTC	File	passwd
40755	root	0	4096	2023-05-16 02:08:25 UTC	Dir	dbus-1
100644	root	0	106	2023-05-16 02:08:19 UTC	File	environment
100644	root	0	8	2023-05-16 02:08:49 UTC	File	timezone
40755	root	0	4096	2023-10-06 06:35:56 UTC	Dir	init.d
40755	root	0	4096	2023-05-16 02:08:28 UTC	Dir	cron.monthly
100644	root	0	11204	2022-02-09 11:30:26 UTC	File	nanorc
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	rc6.d
40755	root	0	4096	2023-05-16 02:09:36 UTC	Dir	rsyslog.d
40755	root	0	4096	2023-05-16 02:12:36 UTC	Dir	network
100440	root	0	1704	2023-10-05 00:16:04 UTC	File	sudoers
100644	root	0	19	2023-02-16 16:02:32 UTC	File	issue.net
40755	root	0	4096	2023-05-16 02:09:22 UTC	Dir	libnl-3
100644	root	0	3663	2016-06-20 00:31:45 UTC	File	screenrc
40755	root	0	4096	2022-02-21 20:05:20 UTC	Dir	gss
100644	root	0	10734	2021-11-11 15:42:38 UTC	File	login.defs
40755	root	0	4096	2023-10-06 06:36:49 UTC	Dir	apparmor.d
100644	root	0	9390	2022-02-14 11:48:05 UTC	File	sudo_logsrvd.conf
100644	root	0	891	2023-10-05 00:16:04 UTC	File	group
100644	root	0	865	2023-10-05 00:15:51 UTC	File	group-
100444	root	0	33	2023-10-05 00:15:45 UTC	File	machine-id
40755	root	0	4096	2022-04-07 19:28:15 UTC	Dir	binfmt.d
100644	root	0	54	2023-05-16 02:09:03 UTC	File	crypttab
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	chrony
100644	root	0	4436	2020-12-15 22:01:56 UTC	File	hdparm.conf
100640	root	42	1024	2023-10-05 00:16:04 UTC	File	shadow
100644	root	0	158	2023-05-16 02:09:26 UTC	File	shells
100644	root	0	2319	2022-01-06 16:23:33 UTC	File	bash.bashrc
40755	root	0	4096	2023-05-16 02:08:48 UTC	Dir	dhcp
40755	root	0	4096	2023-05-16 02:09:39 UTC	Dir	apport
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	rc3.d
100644	root	0	112	2023-05-16 02:09:48 UTC	File	overlayroot.local.conf
100644	root	0	20	2023-10-05 00:15:51 UTC	File	subgid-
100644	root	0	4942	2022-01-24 11:59:00 UTC	File	wgetrc
100644	root	0	22023	2023-10-06 06:37:21 UTC	File	ld.so.cache
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	rc0.d
40775	root	116	4096	2022-03-30 10:32:38 UTC	Dir	landscape
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	rc1.d
100644	root	0	4573	2022-02-14 11:48:05 UTC	File	sudo.conf
100644	root	0	191	2022-03-17 17:50:40 UTC	File	libaudit.conf
100644	root	0	1523	2022-03-25 09:53:05 UTC	File	usb_modeswitch.conf
40755	root	0	4096	2023-05-16 02:12:31 UTC	Dir	grub.d
40755	root	0	4096	2023-05-16 02:09:41 UTC	Dir	rcS.d
100644	root	0	685	2022-01-08 20:02:36 UTC	File	e2scrub.conf
40755	root	0	4096	2023-05-16 02:08:27 UTC	Dir	networkd-dispatcher
40755	root	0	4096	2023-02-28 02:15:02 UTC	Dir	update-notifier
100644	root	0	767	2022-03-24 16:13:48 UTC	File	netconfig
40755	root	0	4096	2023-05-16 02:09:41 UTC	Dir	update-manager
100644	root	0	1900	2023-10-05 00:16:00 UTC	File	passwd-
40755	root	0	4096	2023-05-16 02:09:36 UTC	Dir	cryptsetup-initramfs
40755	root	0	4096	2023-05-16 02:08:43 UTC	Dir	modules-load.d
40755	root	0	4096	2023-05-16 02:09:29 UTC	Dir	groff
40755	root	0	4096	2023-05-16 02:12:36 UTC	Dir	logrotate.d
40755	root	0	4096	2023-10-11 06:42:54 UTC	Dir	vim
100644	root	0	195	2023-05-16 02:08:28 UTC	File	modules
100640	root	42	748	2023-10-05 00:16:04 UTC	File	gshadow
100755	root	0	228	2022-03-23 11:28:58 UTC	File	nftables.conf
40755	root	0	4096	2023-05-16 02:08:28 UTC	Dir	cron.hourly
40755	root	0	4096	2023-05-16 02:08:28 UTC	Dir	depmod.d
100644	root	0	694	2022-03-23 13:53:14 UTC	File	fuse.conf
100644	root	0	1748	2022-01-06 16:26:54 UTC	File	inputrc
100644	root	0	3028	2023-05-16 02:08:21 UTC	File	adduser.conf
40755	root	0	4096	2023-05-16 02:09:30 UTC	Dir	initramfs-tools
40755	root	0	4096	2023-05-16 02:08:25 UTC	Dir	ca-certificates
40755	root	0	4096	2023-05-16 02:09:15 UTC	Dir	NetworkManager
100644	root	0	6920	2020-08-17 16:00:58 UTC	File	overlayroot.conf
100644	root	0	743	2018-11-12 21:16:03 UTC	File	hibinit-config.cfg
100640	root	42	994	2023-10-05 00:16:00 UTC	File	shadow-
100644	root	0	2969	2022-02-20 14:42:49 UTC	File	debconf.conf
40755	root	0	4096	2023-05-16 02:08:49 UTC	Dir	cron.d
40755	root	0	4096	2023-05-16 02:12:37 UTC	Dir	rc5.d
100644	root	0	1136	2022-03-23 13:49:13 UTC	File	crontab
100644	root	0	2355	2022-02-25 11:32:20 UTC	File	sysctl.conf
40755	root	0	4096	2023-05-16 02:09:45 UTC	Dir	console-setup
40755	root	0	4096	2023-10-06 06:35:56 UTC	Dir	pam.d
40755	root	0	4096	2023-10-05 20:42:47 UTC	Dir	fonts
100644	root	0	101	2023-05-16 02:12:14 UTC	File	fstab
100644	root	0	604	2018-09-15 22:14:19 UTC	File	deluser.conf
100644	root	0	72029	2022-03-21 09:12:23 UTC	File	mime.types
100644	root	0	582	2021-10-15 10:06:05 UTC	File	profile
100644	root	0	42	2023-10-05 00:16:04 UTC	File	subuid
100644	root	0	681	2022-03-23 09:41:49 UTC	File	xattr.conf
100644	root	0	42	2023-10-05 00:16:04 UTC	File	subgid
40755	root	0	4096	2021-09-06 10:51:02 UTC	Dir	usb_modeswitch.d
40755	root	0	4096	2023-05-16 02:08:14 UTC	Dir	opt
40755	root	0	4096	2023-10-06 06:36:24 UTC	Dir	ssl
100640	root	42	726	2023-10-05 00:15:51 UTC	File	gshadow-
100600	root	0	0	2023-05-16 02:08:20 UTC	File	.pwd.lock
100644	root	0	1382	2021-12-23 23:34:59 UTC	File	rsyslog.conf
100644	root	0	34	2020-12-16 11:04:55 UTC	File	ld.so.conf
40755	root	0	4096	2023-05-16 02:09:49 UTC	Dir	update-motd.d
100644	root	0	9456	2023-10-06 06:37:28 UTC	File	locale.gen
100644	root	0	104	2023-02-16 16:02:32 UTC	File	lsb-release
40755	root	0	4096	2023-10-06 06:34:14 UTC	Dir	ld.so.conf.d
100644	root	0	41	2022-10-28 18:43:41 UTC	File	multipath.conf
100644	root	0	92	2021-10-15 10:06:05 UTC	File	host.conf
40755	root	0	4096	2023-05-16 02:09:40 UTC	Dir	lvm
100644	root	0	592	2022-01-24 15:37:01 UTC	File	logrotate.conf
100644	root	0	5532	2023-05-16 02:08:50 UTC	File	ca-certificates.conf.dpkg-old
40755	root	0	4096	2023-05-16 02:09:26 UTC	Dir	tmpfiles.d
100644	root	0	1260	2020-06-16 05:37:53 UTC	File	ucf.conf
40755	root	0	4096	2023-05-16 02:14:23 UTC	Dir	apt
40755	root	0	4096	2023-10-06 06:34:33 UTC	Dir	python3.10
100644	root	0	111	2022-03-24 17:07:09 UTC	File	magic
40755	root	0	4096	2023-05-16 02:09:36 UTC	Dir	needrestart
100644	root	0	367	2020-12-16 11:04:55 UTC	File	bindresvport.blacklist
40755	root	0	4096	2023-05-16 02:12:36 UTC	Dir	ppp
100644	root	0	1816	2019-12-27 00:42:11 UTC	File	ethertypes
---[RESULT]----
None
---------
---[ERROR]----

--------

`

func createQuest(ctx context.Context, client *ent.Client, beacons ...*ent.Beacon) {
	// Mid-Execution
	testTome := client.Tome.Create().
		SetName(namegen.NewComplex()).
		SetDescription("Print a message for fun!").
		SetAuthor("kcarretto").
		SetEldritch(`print(input_params['msg'])`).
		SetParamDefs(`[{"name":"msg","label":"Message","type":"string","placeholder":"something to print"}]`).
		SaveX(ctx)

	q := client.Quest.Create().
		SetName(namegen.NewComplex()).
		SetParameters(`{"msg":"Hello World!"}`).
		SetTome(testTome).
		SaveX(ctx)

	for _, b := range beacons {
		client.Task.Create().
			SetBeacon(b).
			SetCreatedAt(timeAgo(5 * time.Minute)).
			SetClaimedAt(timeAgo(1 * time.Minute)).
			SetExecStartedAt(timeAgo(5 * time.Second)).
			SetOutput("Hello").
			SetQuest(q).
			SaveX(ctx)
	}
}
