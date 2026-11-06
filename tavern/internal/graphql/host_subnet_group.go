package graphql

import (
	"context"
	"fmt"
	"net/netip"
	"sort"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/tag"
)

// groupHostsBySubnet creates or refreshes group tags for reported primary
// subnets shared by at least two hosts. A host is skipped when its primary IP
// is missing or does not belong to its reported subnet.
func groupHostsBySubnet(ctx context.Context, client *ent.Client) ([]*ent.Tag, error) {
	hosts, err := client.Host.Query().All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query hosts: %w", err)
	}

	bySubnet := make(map[string][]int)
	for _, h := range hosts {
		ip, ipErr := netip.ParseAddr(h.PrimaryIP)
		prefix, prefixErr := netip.ParsePrefix(h.Subnet)
		if ipErr != nil || prefixErr != nil || ip.IsUnspecified() || ip.IsLoopback() || ip.IsLinkLocalUnicast() || !prefix.Contains(ip) {
			continue
		}
		cidr := prefix.Masked().String()
		bySubnet[cidr] = append(bySubnet[cidr], h.ID)
	}

	subnets := make([]string, 0, len(bySubnet))
	for cidr, ids := range bySubnet {
		if len(ids) >= 2 {
			subnets = append(subnets, cidr)
		}
	}
	sort.Strings(subnets)
	if len(subnets) == 0 {
		return []*ent.Tag{}, nil
	}

	tx, err := client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	createdOrUpdatedIDs := make([]int, 0, len(subnets))
	for _, cidr := range subnets {
		name := "Subnet: " + cidr
		existing, err := tx.Tag.Query().Where(tag.Name(name)).Only(ctx)
		if err != nil && !ent.IsNotFound(err) {
			return nil, rollback(tx, fmt.Errorf("failed to query tag %q: %w", name, err))
		}
		if existing != nil {
			if existing.Kind != tag.KindGroup {
				return nil, rollback(tx, fmt.Errorf("tag %q already exists with kind %q", name, existing.Kind))
			}
			if _, err := existing.Update().ClearHosts().AddHostIDs(bySubnet[cidr]...).Save(ctx); err != nil {
				return nil, rollback(tx, fmt.Errorf("failed to update tag %q: %w", name, err))
			}
			createdOrUpdatedIDs = append(createdOrUpdatedIDs, existing.ID)
			continue
		}
		created, err := tx.Tag.Create().
			SetName(name).
			SetKind(tag.KindGroup).
			AddHostIDs(bySubnet[cidr]...).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to create tag %q: %w", name, err))
		}
		createdOrUpdatedIDs = append(createdOrUpdatedIDs, created.ID)
	}
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}
	return client.Tag.Query().Where(tag.IDIn(createdOrUpdatedIDs...)).Order(ent.Asc(tag.FieldName)).All(ctx)
}
