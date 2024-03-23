package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"regexp"
	"time"

	"realm.pub/bin/pwnboard-go/pwnboard"
	"realm.pub/bin/pwnboard-go/tavern"
)

func NewHostnameHostsFilter(re string) func([]tavern.Host) (matching []tavern.Host) {
	regex := regexp.MustCompile(re)
	return func(hosts []tavern.Host) (matching []tavern.Host) {
		// Find Matching Hosts
		for _, host := range hosts {
			if regex.Match([]byte(host.Name)) {
				matching = append(matching, host)
			}
		}
		return
	}
}

func main() {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
	defer cancel()

	var (
		tavernURL      = "https://tavern.aws-metadata.com"
		credentialPath = ".tavern-auth"

		pwnboardURL     = "https://pwnboard.aws-metadata.com"
		pwnboardAppName = "Realm"

		lookbackWindow = 3 * time.Minute
		sleepInterval  = 30 * time.Second
		httpTimeouts   = 30 * time.Second
	)

	token, err := getAuthToken(ctx, tavernURL, credentialPath)
	if err != nil {
		log.Fatalf("failed to obtain authentication credentials: %v", err)
	}

	tavern_client := &tavern.Client{
		Credential: token,
		URL:        fmt.Sprintf("%s/graphql", tavernURL),
		HTTP: &http.Client{
			Timeout: httpTimeouts,
		},
	}

	pwnboard_client := &pwnboard.Client{
		ApplicationName: pwnboardAppName,
		URL:             pwnboardURL,
		HTTP: &http.Client{
			Timeout: httpTimeouts,
		},
	}

	for {

		log.Printf("Querying Tavern for any Hosts seen in the last %s...", lookbackWindow)
		hosts, err := tavern_client.GetHostsSeenInLastDuration(lookbackWindow)
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

		log.Printf("Sleeping for %s...", sleepInterval)
		time.Sleep(sleepInterval)

	}

}
