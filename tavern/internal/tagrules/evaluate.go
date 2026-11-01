package tagrules

import (
	"context"
	"sort"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/tagrule"
)

// RuleWithTag is a TagRule with its target tag preloaded.
type RuleWithTag struct {
	Rule *ent.TagRule
	Tag  *ent.Tag
}

// SnapshotOf converts an ent Host to a HostSnapshot.
// Platform uses the GraphQL enum string (e.g. PLATFORM_LINUX).
func SnapshotOf(h *ent.Host) HostSnapshot {
	platform := h.Platform.String()
	return HostSnapshot{
		Platform:   platform,
		PrimaryIP:  h.PrimaryIP,
		ExternalIP: h.ExternalIP,
		Name:       h.Name,
	}
}

// LoadRules loads all tag rules ordered by priority ASC, ID ASC,
// with their target tags.
func LoadRules(ctx context.Context, client *ent.Client) ([]RuleWithTag, error) {
	rules, err := client.TagRule.Query().
		WithTag().
		Order(tagrule.ByPriority(), tagrule.ByID()).
		All(ctx)
	if err != nil {
		return nil, err
	}
	out := make([]RuleWithTag, 0, len(rules))
	for _, r := range rules {
		t, err := r.Edges.TagOrErr()
		if err != nil || t == nil {
			continue
		}
		out = append(out, RuleWithTag{Rule: r, Tag: t})
	}
	return out, nil
}

// Winners walks rules in order and returns the first matching rule
// per tag-kind slot (group vs service).
func Winners(snapshot HostSnapshot, rules []RuleWithTag) map[tag.Kind]*RuleWithTag {
	winners := map[tag.Kind]*RuleWithTag{}
	for i := range rules {
		r := &rules[i]
		conds, err := ParseConditions(r.Rule.Conditions)
		if err != nil {
			continue
		}
		if !Matches(snapshot, conds) {
			continue
		}
		kind := r.Tag.Kind
		if _, taken := winners[kind]; !taken {
			c := *r
			winners[kind] = &c
		}
		if len(winners) >= 2 {
			break
		}
	}
	return winners
}

// CurrentSlots maps each tag kind present on the host to one tag ID.
func CurrentSlots(tags []*ent.Tag) map[tag.Kind]int {
	slots := map[tag.Kind]int{}
	for _, t := range tags {
		if _, ok := slots[t.Kind]; !ok {
			slots[t.Kind] = t.ID
		}
	}
	return slots
}

// PlanForHost computes the tag IDs to attach for one host.
// In fill-empty mode only empty kind slots are filled; in overwrite mode
// the winning rule's tag replaces the current tag in its slot.
// Slots with no winning rule are always left untouched (never deleted).
func PlanForHost(snapshot HostSnapshot, current map[tag.Kind]int, rules []RuleWithTag, mode SweepMode) []int {
	winners := Winners(snapshot, rules)
	var toAdd []int
	for kind, winner := range winners {
		cur, occupied := current[kind]
		switch mode {
		case SweepOverwrite:
			if !occupied || cur != winner.Tag.ID {
				toAdd = append(toAdd, winner.Tag.ID)
			}
		default: // fill-empty
			if !occupied {
				toAdd = append(toAdd, winner.Tag.ID)
			}
		}
	}
	sort.Ints(toAdd)
	return toAdd
}

// ApplyToHost attaches tags to a host. In overwrite mode the current tag
// in each winning kind slot is removed first; unmatched slots are untouched.
func ApplyToHost(ctx context.Context, client *ent.Client, hostID int, tagIDs []int, current map[tag.Kind]int, rules []RuleWithTag, mode SweepMode) error {
	if len(tagIDs) == 0 {
		return nil
	}
	if mode == SweepOverwrite {
		// Remove the current tag in each slot that has a winner so the
		// winner replaces it instead of accumulating alongside it.
		winnerKinds := map[tag.Kind]bool{}
		for _, id := range tagIDs {
			for _, r := range rules {
				if r.Tag.ID == id {
					winnerKinds[r.Tag.Kind] = true
				}
			}
		}
		var toRemove []int
		for kind := range winnerKinds {
			if cur, ok := current[kind]; ok {
				already := false
				for _, id := range tagIDs {
					if id == cur {
						already = true
						break
					}
				}
				if !already {
					toRemove = append(toRemove, cur)
				}
			}
		}
		if len(toRemove) > 0 {
			if err := client.Host.UpdateOneID(hostID).RemoveTagIDs(toRemove...).Exec(ctx); err != nil {
				return err
			}
		}
	}
	return client.Host.UpdateOneID(hostID).AddTagIDs(tagIDs...).Exec(ctx)
}

// EvaluateHostForCallback is the per-callback path: fill-empty only,
// never displaces an existing tag.
func EvaluateHostForCallback(ctx context.Context, client *ent.Client, hostID int) error {
	h, err := client.Host.Query().
		Where(host.ID(hostID)).
		WithTags().
		Only(ctx)
	if err != nil {
		return err
	}
	rules, err := LoadRules(ctx, client)
	if err != nil {
		return err
	}
	if len(rules) == 0 {
		return nil
	}
	snapshot := SnapshotOf(h)
	current := CurrentSlots(h.Edges.Tags)
	toAdd := PlanForHost(snapshot, current, rules, SweepFillEmpty)
	if len(toAdd) == 0 {
		return nil
	}
	// Re-check: attaching an already-present tag is a no-op in ent,
	// but skip the write entirely when there is nothing new.
	return client.Host.UpdateOneID(hostID).AddTagIDs(toAdd...).Exec(ctx)
}

// SweepResult attributes one rule's backfill outcome.
type SweepResult struct {
	TagRuleID    int
	TagID        int
	MatchedHosts int
	TaggedHosts  int
}

// SweepAll evaluates every rule against every host at once.
// Mode is fill-empty (same as callback) or overwrite (replace the current
// tag in each winning slot). Slots with no winner are always untouched.
func SweepAll(ctx context.Context, client *ent.Client, mode SweepMode) ([]SweepResult, error) {
	rules, err := LoadRules(ctx, client)
	if err != nil {
		return nil, err
	}
	hosts, err := client.Host.Query().WithTags().All(ctx)
	if err != nil {
		return nil, err
	}
	matched := map[int]int{}
	tagged := map[int]int{}
	totalTagged := 0
	for _, h := range hosts {
		snapshot := SnapshotOf(h)
		current := CurrentSlots(h.Edges.Tags)
		winners := Winners(snapshot, rules)
		if len(winners) == 0 {
			continue
		}
		toAdd := PlanForHost(snapshot, current, rules, mode)
		for kind, winner := range winners {
			_ = kind
			matched[winner.Rule.ID]++
		}
		if len(toAdd) == 0 {
			continue
		}
		if err := ApplyToHost(ctx, client, h.ID, toAdd, current, rules, mode); err != nil {
			return nil, err
		}
		totalTagged++
		// Attribute tagged hosts to the winning rules that contributed tags.
		for _, id := range toAdd {
			for _, r := range rules {
				if r.Tag.ID == id {
					tagged[r.Rule.ID]++
					break
				}
			}
		}
	}
	_ = totalTagged
	results := make([]SweepResult, 0, len(rules))
	for _, r := range rules {
		results = append(results, SweepResult{
			TagRuleID:    r.Rule.ID,
			TagID:        r.Tag.ID,
			MatchedHosts: matched[r.Rule.ID],
			TaggedHosts:  tagged[r.Rule.ID],
		})
	}
	return results, nil
}
