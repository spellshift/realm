package agent

import (
	"context"
	"log"
	"math/rand"
	"regexp"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/testdata/processlist"
)

var (
	reportFileRegex        = regexp.MustCompile(`report\.file\("([^"]+)"\)`)
	reverseShellRegex      = regexp.MustCompile(`pivot\.reverse_shell_pty\(\)`)
	reportProcessListRegex = regexp.MustCompile(`report\.process_list\(\)`)
	processListRegex       = regexp.MustCompile(`process\.list\(\)`)
	reportCredentialRegex  = regexp.MustCompile(`report\.credential\("([^"]+)",\s*"([^"]+)"\)`)
	fetchAssetRegex        = regexp.MustCompile(`fetch\.asset\("([^"]+)"\)`)
)

// Option is a functional option for configuring an Agent.
type Option func(*Agent)

// WithCallbackInterval sets the callback interval for the agent.
func WithCallbackInterval(d time.Duration) Option {
	return func(a *Agent) {
		a.callbackInterval = d
	}
}

// WithHostIdentifier sets the host identifier for the agent.
func WithHostIdentifier(id string) Option {
	return func(a *Agent) {
		a.hostIdentifier = id
	}
}

// WithAgentIdentifier sets the agent identifier for the agent.
func WithAgentIdentifier(id string) Option {
	return func(a *Agent) {
		a.agentIdentifier = id
	}
}

// WithPlatform sets the platform for the agent.
func WithPlatform(p c2pb.Host_Platform) Option {
	return func(a *Agent) {
		a.platform = p
	}
}

// WithErrorRate sets the error rate for the agent.
func WithErrorRate(rate int) Option {
	return func(a *Agent) {
		a.errorRate = rate
	}
}

// WithKillChance sets the kill chance for the agent.
func WithKillChance(chance int) Option {
	return func(a *Agent) {
		a.killChance = chance
	}
}

// WithDialOptions sets the gRPC dial options for the agent.
func WithDialOptions(opts ...grpc.DialOption) Option {
	return func(a *Agent) {
		a.dialOpts = append(a.dialOpts, opts...)
	}
}

// Agent is a test agent that connects to the gRPC server.
type Agent struct {
	callbackURL     string
	callbackInterval time.Duration
	hostIdentifier  string
	agentIdentifier string
	platform        c2pb.Host_Platform
	errorRate       int
	killChance      int
	client          c2pb.C2Client
	dialOpts        []grpc.DialOption
}

// New creates a new agent.
func New(callbackURL string, opts ...Option) *Agent {
	a := &Agent{
		callbackURL:     callbackURL,
		callbackInterval: 5 * time.Second,
		hostIdentifier:  "default-host",
		agentIdentifier: "default-agent",
		platform:        c2pb.Host_PLATFORM_LINUX,
		errorRate:       0,
		killChance:      0,
		dialOpts:        []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())},
	}
	for _, opt := range opts {
		opt(a)
	}
	return a
}

// Run starts the agent's main loop.
func (a *Agent) Run(ctx context.Context) error {
	conn, err := grpc.Dial(a.callbackURL, a.dialOpts...)
	if err != nil {
		return err
	}
	defer conn.Close()

	a.client = c2pb.NewC2Client(conn)

	ticker := time.NewTicker(a.callbackInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			if a.shouldKill() {
				log.Println("Agent is being killed by KillChance")
				return nil
			}
			a.claimTasks(ctx)
		}
	}
}

func (a *Agent) shouldKill() bool {
	return rand.Intn(100) < a.killChance
}

func (a *Agent) claimTasks(ctx context.Context) {
	req := &c2pb.ClaimTasksRequest{
		Beacon: &c2pb.Beacon{
			Host: &c2pb.Host{
				Identifier: a.hostIdentifier,
				Name:       a.hostIdentifier,
				Platform:   a.platform,
			},
			Agent: &c2pb.Agent{
				Identifier: a.agentIdentifier,
			},
			Interval: uint64(a.callbackInterval.Seconds()),
		},
	}
	resp, err := a.client.ClaimTasks(ctx, req)
	if err != nil {
		log.Printf("Failed to claim tasks: %v", err)
		return
	}

	for _, task := range resp.Tasks {
		go a.handleTask(ctx, task)
	}
}

func (a *Agent) handleTask(ctx context.Context, task *c2pb.Task) {
	log.Printf("Handling task %d: %s", task.Id, task.Tome.Eldritch)
	if a.shouldError() {
		a.reportTaskError(ctx, task.Id, "a goblin stole the output")
		return
	}

	tome := task.Tome.Eldritch
	handled := false

	// report.file("path/to/mock/file")
	matches := reportFileRegex.FindAllStringSubmatch(tome, -1)
	for _, match := range matches {
		if len(match) > 1 {
			a.reportFile(ctx, task.Id, match[1])
			handled = true
		}
	}

	// pivot.reverse_shell_pty()
	if reverseShellRegex.MatchString(tome) {
		a.reverseShell(ctx, task.Id)
		handled = true
	}

	// report.process_list() or process.list()
	if reportProcessListRegex.MatchString(tome) || processListRegex.MatchString(tome) {
		a.reportProcessList(ctx, task.Id)
		handled = true
	}

	// report.credential("user", "password")
	matches = reportCredentialRegex.FindAllStringSubmatch(tome, -1)
	for _, match := range matches {
		if len(match) > 2 {
			a.reportCredential(ctx, task.Id, match[1], match[2])
			handled = true
		}
	}

	// fetch.asset("asset-name")
	matches = fetchAssetRegex.FindAllStringSubmatch(tome, -1)
	for _, match := range matches {
		if len(match) > 1 {
			a.fetchAsset(ctx, task.Id, match[1])
			handled = true
		}
	}

	if !handled {
		a.reportTaskOutput(ctx, task.Id, a.mockTaskOutput())
	}
}
func (a *Agent) reportTaskError(ctx context.Context, taskID int64, errMsg string) {
	_, err := a.client.ReportTaskOutput(ctx, &c2pb.ReportTaskOutputRequest{
		Output: &c2pb.TaskOutput{
			Id:    taskID,
			Error: &c2pb.TaskError{Msg: errMsg},
		},
	})
	if err != nil {
		log.Printf("Failed to report task error: %v", err)
	}
}

func (a *Agent) reportTaskOutput(ctx context.Context, taskID int64, output string) {
	_, err := a.client.ReportTaskOutput(ctx, &c2pb.ReportTaskOutputRequest{
		Output: &c2pb.TaskOutput{
			Id:     taskID,
			Output: output,
		},
	})
	if err != nil {
		log.Printf("Failed to report task output: %v", err)
	}
}
func (a *Agent) reportFile(ctx context.Context, taskID int64, filePath string) {
	stream, err := a.client.ReportFile(ctx)
	if err != nil {
		log.Printf("Failed to open stream for ReportFile: %v", err)
		return
	}

	fileContent := a.mockFile()
	chunkSize := 1024
	for i := 0; i < len(fileContent); i += chunkSize {
		end := i + chunkSize
		if end > len(fileContent) {
			end = len(fileContent)
		}

		req := &c2pb.ReportFileRequest{
			TaskId: taskID,
			Chunk: &epb.File{
				Metadata: &epb.FileMetadata{
					Path: filePath,
				},
				Chunk: fileContent[i:end],
			},
		}

		if err := stream.Send(req); err != nil {
			log.Printf("Failed to send file chunk: %v", err)
			return
		}
	}

	_, err = stream.CloseAndRecv()
	if err != nil {
		log.Printf("Failed to close stream for ReportFile: %v", err)
	}
}
func (a *Agent) mockFile() []byte {
	return []byte("Never trust an elf!")
}
func (a *Agent) reverseShell(ctx context.Context, taskID int64) {
	stream, err := a.client.ReverseShell(ctx)
	if err != nil {
		log.Printf("Failed to open stream for ReverseShell: %v", err)
		return
	}
	defer stream.CloseSend()

	// Goroutine to read from the stream (commands from server) and write back (output)
	go func() {
		for {
			// Recv blocks until it receives a message from the server or the stream is closed.
			resp, err := stream.Recv()
			if err != nil {
				return
			}

			// We received a command, now process it.
			output := a.handleShellCommand(string(resp.Data))

			// Send the output back to the server.
			req := &c2pb.ReverseShellRequest{
				TaskId: taskID,
				Kind:   c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_DATA,
				Data:   []byte(output),
			}
			if err := stream.Send(req); err != nil {
				return
			}
		}
	}()

	initialReq := &c2pb.ReverseShellRequest{
		TaskId: taskID,
		Kind:   c2pb.ReverseShellMessageKind_REVERSE_SHELL_MESSAGE_KIND_PING,
	}
	if err := stream.Send(initialReq); err != nil {
		log.Printf("Failed to send initial reverse shell message: %v", err)
		return
	}
}

func (a *Agent) handleShellCommand(command string) string {
	command = strings.TrimSpace(command)
	switch {
	case command == "ls":
		return "there is a wizard and a warrior here"
	case command == "pwd":
		return "/the/dungeon"
	case command == "whoami":
		return "a humble adventurer"
	default:
		return "Unknown command. Try 'ls', 'pwd', or 'whoami'."
	}
}

func (a *Agent) mockTaskOutput() string {
	outputs := []string{
		"The dragonborn breathes fire, but the spreadsheet survives.",
		"A mimic disguised as a treasure chest contains only boring TPS reports.",
		"The rogue attempts to backstab the final boss, but trips on a network cable.",
		"The wizard casts a powerful spell, but it only manages to clear the browser cache.",
	}
	return outputs[rand.Intn(len(outputs))]
}

func (a *Agent) shouldError() bool {
	return rand.Intn(100) < a.errorRate
}

func (a *Agent) reportProcessList(ctx context.Context, taskID int64) {
	_, err := a.client.ReportProcessList(ctx, &c2pb.ReportProcessListRequest{
		TaskId: taskID,
		List:   a.mockProcessList(),
	})
	if err != nil {
		log.Printf("Failed to report process list: %v", err)
	}
}

func (a *Agent) mockProcessList() *epb.ProcessList {
	return processlist.New()
}

func (a *Agent) reportCredential(ctx context.Context, taskID int64, principal, secret string) {
	_, err := a.client.ReportCredential(ctx, &c2pb.ReportCredentialRequest{
		TaskId:     taskID,
		Credential: a.mockCredential(principal, secret),
	})
	if err != nil {
		log.Printf("Failed to report credential: %v", err)
	}
}

func (a *Agent) mockCredential(principal, secret string) *epb.Credential {
	return &epb.Credential{
		Principal: principal,
		Secret:    secret,
		Kind:      epb.Credential_KIND_PASSWORD,
	}
}

func (a *Agent) fetchAsset(ctx context.Context, taskID int64, assetName string) {
	stream, err := a.client.FetchAsset(ctx, &c2pb.FetchAssetRequest{Name: assetName})
	if err != nil {
		log.Printf("Failed to fetch asset: %v", err)
		return
	}

	for {
		_, err := stream.Recv()
		if err != nil {
			break
		}
	}
}
