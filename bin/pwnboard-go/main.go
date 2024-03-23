package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"regexp"
	"time"
)

func NewHostnameHostsFilter(re string) func([]Host) (matching []Host) {
	regex := regexp.MustCompile(re)
	return func(hosts []Host) (matching []Host) {
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
		lookbackWindow = 3 * time.Minute
		sleepInterval  = 10 * time.Second
	)

	token, err := getAuthToken(ctx, tavernURL, credentialPath)
	if err != nil {
		log.Fatalf("failed to obtain authentication credentials: %v", err)
	}

	client := &Client{
		Credential: token,
		URL:        fmt.Sprintf("%s/graphql", tavernURL),
		HTTP: &http.Client{
			Timeout: 60 * time.Second,
		},
	}

	for {

		log.Printf("Starting new lookbackWindow...")

		hosts, err := client.GetHostsSeenInLastDuration(lookbackWindow)
		if err != nil {
			log.Fatalf("failed to query hosts: %v", err)
		}

		log.Printf("Successfully queried hosts (len=%d)", len(hosts))
		for _, host := range hosts {
			log.Printf("Found Host: id=%s\tname=%s\tip=%s", host.ID, host.Name, host.PrimaryIP)

		}

		log.Printf("Sleeping for %s...", sleepInterval)
		time.Sleep(sleepInterval)

	}

}
