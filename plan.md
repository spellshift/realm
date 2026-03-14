1. **Create the `ScheduledTask` Ent schema**
   - Create `tavern/internal/ent/schema/scheduled_task.go` with fields `run_on_new_beacon_callback`, `run_on_first_host_callback`, and `run_on_schedule`.
   - Edges: `tome` (to Tome, Unique, Required), `scheduled_hosts` (to Host, non-unique).

2. **Remove fields from `Tome` schema**
   - In `tavern/internal/ent/schema/tome.go`, remove `run_on_new_beacon_callback`, `run_on_first_host_callback`, `run_on_schedule` fields, and `scheduled_hosts` edge.

3. **Generate Ent/GraphQL bindings**
   - Run `cd tavern && go generate ./...` to regenerate Ent and GraphQL schemas.

4. **Update `handleTomeAutomation` logic**
   - In `tavern/internal/c2/api_claim_tasks.go`, find `handleTomeAutomation`.
   - Update it to query `ScheduledTask` instead of `Tome`.
   - Logic:
     ```go
     candidateTasks, err := srv.graph.ScheduledTask.Query().
         WithTome().
         Where(scheduledtask.Or(
             scheduledtask.RunOnNewBeaconCallback(true),
             scheduledtask.RunOnFirstHostCallback(true),
             scheduledtask.RunOnScheduleNEQ(""),
         )).
         All(ctx)
     ```
   - Update `shouldRun` logic based on `ScheduledTask` fields.
   - For `selectedTomes`, it should be a map of `int` to `*ent.Tome` or `*ent.ScheduledTask`, where we use `task.Edges.Tome` to get the Tome, so we only run each Tome once if multiple tasks select the same Tome, or maybe multiple tasks could trigger the same tome. Let's keep a map of `tomeID -> *ent.Tome`.

5. **Update Tests**
   - Update `tavern/internal/c2/tome_automation_test.go` to create `ScheduledTask`s linking to Tomes, rather than setting schedule fields on the Tomes directly.
   - The test assertions and logic will remain largely the same, but the setup will involve creating a `ScheduledTask` for each `Tome` we want to automate.

6. **Add a mutation to create a ScheduledTask ent**
   - The user asked to "Add a mutation to create a scheduled task ent."
   - The new ent should already have mutations generated if we add `entgql.Mutations(entgql.MutationCreate(), entgql.MutationUpdate())` to its `Annotations()`. This automatically generates `createScheduledTask` and `updateScheduledTask` mutations.
   - We must also ensure `CreateScheduledTaskInput` matches the fields. We will ensure this in step 3.

7. **Pre-commit Checks**
   - Make sure proper testing, verifications, reviews and reflections are done.
