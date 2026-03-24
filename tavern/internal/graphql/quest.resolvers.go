package graphql

import (
	"context"
	"sort"
	"strings"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/graphql/models"
)

// Diffs is the resolver for the diffs field.
func (r *questResolver) Diffs(ctx context.Context, obj *ent.Quest) ([]*models.TaskDiff, error) {
	tasks, err := obj.QueryTasks().Select(task.FieldID, task.FieldOutput, task.FieldError).Order(ent.Asc(task.FieldID)).All(ctx)
	if err != nil {
		return nil, err
	}

	type diffKey struct {
		output string
		error  string
	}

	diffMap := make(map[diffKey][]int)
	for _, t := range tasks {
		var outStr, errStr string
		if t.Output != "" {
			outStr = strings.TrimSpace(t.Output)
		}
		if t.Error != "" {
			errStr = strings.TrimSpace(t.Error)
		}

		key := diffKey{output: outStr, error: errStr}
		diffMap[key] = append(diffMap[key], t.ID)
	}

	var diffs []*models.TaskDiff
	for key, ids := range diffMap {
		outVal := key.output
		errVal := key.error
		diffs = append(diffs, &models.TaskDiff{
			Ids:    ids,
			Output: &outVal,
			Error:  &errVal,
		})
	}

	// Sort diffs by the first ID in the ids slice for deterministic ordering
	sort.Slice(diffs, func(i, j int) bool {
		return diffs[i].Ids[0] < diffs[j].Ids[0]
	})

	return diffs, nil
}
