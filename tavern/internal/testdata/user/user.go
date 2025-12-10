package user

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"math/rand"
	"net/http"
	"net/http/cookiejar"
	"time"
)

type User struct {
	graphqlURL    string
	actionInterval time.Duration
	actionWeights  map[string]int
	client        *http.Client
}

func New(graphqlURL string, actionInterval time.Duration, actionWeights map[string]int) (*User, error) {
	jar, err := cookiejar.New(nil)
	if err != nil {
		return nil, err
	}

	return &User{
		graphqlURL:    graphqlURL,
		actionInterval: actionInterval,
		actionWeights:  actionWeights,
		client:        &http.Client{Jar: jar},
	}, nil
}

func (u *User) Run(ctx context.Context) error {
	ticker := time.NewTicker(u.actionInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			u.performRandomAction(ctx)
		}
	}
}

func (u *User) performRandomAction(ctx context.Context) {
	// Get total weight
	totalWeight := 0
	for _, weight := range u.actionWeights {
		totalWeight += weight
	}

	// Get random number
	randNum := rand.Intn(totalWeight)

	// Find action
	for action, weight := range u.actionWeights {
		if randNum < weight {
			u.executeAction(ctx, action)
			return
		}
		randNum -= weight
	}
}

func (u *User) executeAction(ctx context.Context, action string) {
	switch action {
	case "createQuest":
		hosts, err := u.getHosts(ctx)
		if err != nil || len(hosts) == 0 {
			return
		}
		host := hosts[rand.Intn(len(hosts))]
		if len(host.Beacons) == 0 {
			return
		}
		beacon := host.Beacons[rand.Intn(len(host.Beacons))]

		tomes, err := u.getTomes(ctx)
		if err != nil || len(tomes) == 0 {
			return
		}
		tome := tomes[rand.Intn(len(tomes))]

		u.createQuest(ctx, tome.ID, beacon.ID)
	}
}

type Host struct {
	ID      string   `json:"id"`
	Beacons []Beacon `json:"beacons"`
}

type Beacon struct {
	ID string `json:"id"`
}

type Tome struct {
	ID string `json:"id"`
}

func (u *User) query(ctx context.Context, query string, result interface{}) error {
	reqBody, err := json.Marshal(map[string]string{"query": query})
	if err != nil {
		return err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", u.graphqlURL, bytes.NewBuffer(reqBody))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := u.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	return json.NewDecoder(resp.Body).Decode(result)
}

func (u *User) getHosts(ctx context.Context) ([]Host, error) {
	query := `query { hosts { edges { node { id beacons { id } } } } }`
	var result struct {
		Data struct {
			Hosts struct {
				Edges []struct {
					Node Host `json:"node"`
				} `json:"edges"`
			} `json:"hosts"`
		} `json:"data"`
	}
	if err := u.query(ctx, query, &result); err != nil {
		return nil, err
	}
	var hosts []Host
	for _, edge := range result.Data.Hosts.Edges {
		hosts = append(hosts, edge.Node)
	}
	return hosts, nil
}

func (u *User) getTomes(ctx context.Context) ([]Tome, error) {
	query := `query { tomes { edges { node { id } } } }`
	var result struct {
		Data struct {
			Tomes struct {
				Edges []struct {
					Node Tome `json:"node"`
				} `json:"edges"`
			} `json:"tomes"`
		} `json:"data"`
	}
	if err := u.query(ctx, query, &result); err != nil {
		return nil, err
	}
	var tomes []Tome
	for _, edge := range result.Data.Tomes.Edges {
		tomes = append(tomes, edge.Node)
	}
	return tomes, nil
}

func (u *User) createQuest(ctx context.Context, tomeID, beaconID string) error {
	query := `mutation { createQuest(input: { tomeID: "%s", beaconIDs: ["%s"], parameters: "%s" }) { id } }`
	var result struct {
		Data struct {
			CreateQuest struct {
				ID string `json:"id"`
			} `json:"createQuest"`
		} `json:"data"`
	}
	return u.query(ctx, fmt.Sprintf(query, tomeID, beaconID, u.mockParameters()), &result)
}

func (u *User) mockParameters() string {
	return "{}"
}
