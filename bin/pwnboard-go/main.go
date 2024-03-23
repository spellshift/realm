package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"realm.pub/bin/pwnboard-go/pwnboard"
	"realm.pub/bin/pwnboard-go/tavern"
)

const (
	TAVERN_URL      = "https://tavern.aws-metadata.com"
	CREDENTIAL_PATH = ".tavern-auth"

	PWNBOARD_URL     = "https://pwnboard.aws-metadata.com"
	PWNBOARD_APP_NAME = "Realm"

	LOOKBACK_WINDOW = 3 * time.Minute
	SLEEP_INTERVAL  = 30 * time.Second
	HTTP_TIMEOUT   = 30 * time.Second
)

func main() {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
	defer cancel()


	token, err := getAuthToken(ctx, TAVERN_URL, CREDENTIAL_PATH)
	if err != nil {
		log.Fatalf("failed to obtain authentication credentials: %v", err)
	}

	tavern_client := &tavern.Client{
		Credential: token,
		URL:        fmt.Sprintf("%s/graphql", TAVERN_URL),
		HTTP: &http.Client{
			Timeout: HTTP_TIMEOUT,
		},
	}

	pwnboard_client := &pwnboard.Client{
		ApplicationName: PWNBOARD_APP_NAME,
		URL:             PWNBOARD_URL,
		HTTP: &http.Client{
			Timeout: HTTP_TIMEOUT,
		},
	}

	for {

		log.Printf("Querying Tavern for any Hosts seen in the last %s...", LOOKBACK_WINDOW)
		hosts, err := tavern_client.GetHostsSeenInLastDuration(LOOKBACK_WINDOW)
		if err != nil {
			log.Fatalf("failed to query hosts: %v", err)
		}

		log.Printf("Found %d host(s)!", len(hosts))
		var ips []string
		for _, host := range hosts {
			ips = append(ips, host.PrimaryIP)
		}

		log.Printf("Sending %d IP(s) to PWNboard...", len(ips))
		err = pwnboard_client.ReportIPs(ips)
		if err != nil {
			log.Fatalf("failed to send ips to PWNboard: %v", err)
		}

		log.Printf("Sleeping for %s...", SLEEP_INTERVAL)
		time.Sleep(SLEEP_INTERVAL)

	}

}
